//!与任务管理相关的数据结构
use core::cell::RefMut;

use super::{pid::{pid_alloc, KernelStack, PidHandle}, TaskContext};
use crate::{config::{kernel_stack_position, TRAP_CONTEXT}, mm::{MapPermission, MemorySet, PhysPageNum,VirtAddr, KERNEL_SPACE}, trap::{trap_handler, TrapContext}};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use alloc::sync::{Arc, Weak};

///代表任务状态的枚举类型
#[derive(Copy,Clone,PartialEq)]
///任务状态
pub enum TaskStatus {
  ///准备运行
  Ready, 
  ///正在运行
  Running,
  ///已经退出 
  Exited, 
  ///僵尸状态
  Zombie,
}

///进程控制块：内核保存一个进程信息的数据结构
pub struct TaskControlBlock {
  pub pid: PidHandle,
  ///内核栈--初始化后就不可变
  pub kernel_stack: KernelStack,
  ///可变数据的封装
  inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
  ///Trap 上下文被实际存放在物理页帧的物理页号
  pub trap_cx_ppn: PhysPageNum,
  ///应用数据仅有可能出现在应用地址空间低于 base_size 字节的区域中
  pub base_size: usize,
  ///暂停的任务的任务上下文
  pub task_cx: TaskContext,
  ///当前进程的执行状态
  pub task_status: TaskStatus,
  ///应用地址空间
  pub memory_set: MemorySet,
  ///当前进程的父进程
  pub parent: Option<Weak<TaskControlBlock>>,
  ///当前进程的所有子进程的任务控制块
  pub children: Vec<Arc<TaskControlBlock>>,
  ///当进程主动退出时，退出码会保存在该位置
  pub exit_code: i32,
}

impl TaskControlBlockInner {
  ///获取进程控制块的Trap上下文的可变引用
  pub fn get_trap_cx(&self) -> &'static mut TrapContext {
    self.trap_cx_ppn.get_mut()
  }
  ///获取应用地址空间的token
  pub fn get_user_token(&self) -> usize {
    self.memory_set.token()
  }
  ///获取进程的状态
  fn get_status(&self) -> TaskStatus {
    self.task_status
  }
  ///判断一个进程是否为僵尸进程
  pub fn is_zombie(&self) -> bool {
    self.get_status() == TaskStatus::Zombie
  }
}

impl TaskControlBlock {
  ///获取内层 TaskControlBlockInner 的可变引用并可以对它指向的内容进行修改
  pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
    self.inner.exclusive_access()
  }
  ///获得进程标识符
  pub fn getpid(&self) -> usize {
    self.pid.0
  }
  ///实现exec的系统调用--当前进程加载并执行另一个 ELF 格式可执行文件
  pub fn exec(&self, elf_data: &[u8]){
    // memory_set包含elf program headers/trampoline/trap context/user stack
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    let trap_cx_ppn = memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
    let mut inner = self.inner_exclusive_access();
    //修改地址空间
    inner.memory_set = memory_set;
    //修改地址空间的Trap上下文所在物理页面
    inner.trap_cx_ppn = trap_cx_ppn;
    //初始化trap context
    let trap_cx = inner.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point, 
      user_sp, 
      KERNEL_SPACE.exclusive_access().token(), 
      self.kernel_stack.get_top(), 
      trap_handler as usize
    );
  }
  ///创建一个新进程
  pub fn new(elf_data: &[u8]) -> Self {
    //memory_set中包含elf program headers/trampoline/trap context/user stack
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    //找到应用地址空间的Trap上下文被放置的物理页号
    let trap_cx_ppn = memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
    //在内核空间获取一个pid和内核栈
    let pid_handle = pid_alloc();
    let kernel_stack = KernelStack::new(&pid_handle);
    //内核栈在内核的地址空间
    let kernel_stack_top = kernel_stack.get_top();
    //在该进程的内核栈上压入初始化的任务上下文
    let task_control_block = Self {
      pid: pid_handle,
      kernel_stack,
      inner: unsafe { UPSafeCell::new(TaskControlBlockInner{
        trap_cx_ppn,
        base_size: user_sp,
        task_cx: TaskContext::goto_trap_return(kernel_stack_top),
        task_status: TaskStatus::Ready,
        memory_set,
        parent: None,
        children: Vec::new(),
        exit_code: 0,
      })},
    };
    //在用户地址空间准备TrapContext
    let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point,
      user_sp,
      KERNEL_SPACE.exclusive_access().token(),
      kernel_stack_top,
      trap_handler as usize,
    );
    task_control_block
  }
  ///实现 fork 系统调用--创建子进程
  pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock>{
    let mut parent_inner = self.inner_exclusive_access();
    //复制用户地址空间
    let memory_set = MemorySet::from_exited_user(
      &parent_inner.memory_set
    );
    let trap_cx_ppn = memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
    //在内核空间中分配一个pid和内核栈
    let pid_handle = pid_alloc();
    let kernel_stack = KernelStack::new(&pid_handle);
    let kernel_stack_top = kernel_stack.get_top();
    let task_control_block = Arc::new(
      TaskControlBlock{
        pid: pid_handle,
        kernel_stack,
        inner: unsafe { UPSafeCell::new(TaskControlBlockInner{
            trap_cx_ppn,
            base_size: parent_inner.base_size,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            task_status: TaskStatus::Ready,
            memory_set,
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            exit_code: 0,
          })},
    });
    //将孩子进程的PCB加入到其父进程的children中
    parent_inner.children.push(task_control_block.clone());
    let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
    trap_cx.kernel_sp = kernel_stack_top;
    task_control_block
  }
}