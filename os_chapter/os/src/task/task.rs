//!与任务管理相关的数据结构
use core::cell::RefMut;

use super::{pid::{KernelStack, PidHandle}, TaskContext};
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
  ///获取进程控制块的Trap上下文所在的物理页面
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
  ///创建一个新进程
  pub fn new(elf_data: &[u8]) -> Self {

  }
  ///实现exec的系统调用--当前进程加载并执行另一个 ELF 格式可执行文件
  pub fn exec(&self, elf_data: &[u8]){

  }
  ///实现 fork 系统调用--创建子进程
  pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {

  }
  // ///新建一个任务控制块TCB
  // pub fn new(elf_data: &[u8], app_id: usize) -> Self {
  //   let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
  //   //找到应用地址空间的Trap上下文被放置的物理页号
  //   let trap_cx_ppn = memory_set
  //       .translate(VirtAddr::from(TRAP_CONTEXT).into())
  //       .unwrap()
  //       .ppn();
  //   let task_status = TaskStatus::Ready;
  //   //找到该应用在内核空间对应的内核栈
  //   let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
  //   //将逻辑段插入到内核地址空间
  //   KERNEL_SPACE
  //       .exclusive_access()
  //       .insert_frames_area(
  //         kernel_stack_bottom.into(),
  //         kernel_stack_top.into(),
  //         MapPermission::R | MapPermission::W,
  //       );
  //   let task_control_block = Self {
  //     task_status,
  //     task_cx: TaskContext::goto_trap_return(kernel_stack_top),
  //     memory_set,
  //     trap_cx_ppn,
  //     base_size: user_sp,
  //   };
  //   //在用户地址空间准备TrapContext
  //   let trap_cx = task_control_block.get_trap_cx();
  //   *trap_cx = TrapContext::app_init_context(
  //     entry_point,
  //     user_sp,
  //     KERNEL_SPACE.exclusive_access().token(),
  //     kernel_stack_top,
  //     trap_handler as usize,
  //   );
  //   task_control_block
  // }
}