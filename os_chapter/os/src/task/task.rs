//!与任务管理相关的数据结构
use super::TaskContext;
use crate::println;
use crate::{config::{kernel_stack_position, TRAP_CONTEXT}, mm::{MapPermission, MemorySet, PhysPageNum,VirtAddr, KERNEL_SPACE}, trap::{trap_handler, TrapContext}};

///代表任务状态的枚举类型
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus {
  Ready, //准备运行
  Running, //正在运行
  Exited, //已经退出
}

///任务控制块TCB：内核保存一个应用的更多信息的数据结构
pub struct TaskControlBlock {
  pub task_status: TaskStatus, //任务当前状态
  pub task_cx: TaskContext, //任务上下文
  pub memory_set: MemorySet, //应用的地址空间
  pub trap_cx_ppn: PhysPageNum, //Trap 上下文被实际存放在物理页帧的物理页号 
  pub base_size: usize, //应用数据大小--应用地址空间中从开始到用户栈结束一共包含多少字节
}

impl TaskControlBlock {
  pub fn get_trap_cx(&self) -> &'static mut TrapContext {
    //用get_mut获取物理页中的内容
    self.trap_cx_ppn.get_mut()
  }
  pub fn get_user_token(&self) -> usize {
    self.memory_set.token()
  }
  pub fn new(elf_data: &[u8], app_id: usize) -> Self {
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    //找到应用地址空间的Trap上下文被放置的物理页号
    let trap_cx_ppn = memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
    let task_status = TaskStatus::Ready;
    //找到该应用在内核空间对应的内核栈
    let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
    //将逻辑段插入到内核地址空间
    KERNEL_SPACE
        .exclusive_access()
        .insert_frames_area(
          kernel_stack_bottom.into(),
          kernel_stack_top.into(),
          MapPermission::R | MapPermission::W,
        );
    let task_control_block = Self {
      task_status,
      task_cx: TaskContext::goto_trap_return(kernel_stack_top),
      memory_set,
      trap_cx_ppn,
      base_size: user_sp,
    };
    //在用户地址空间准备TrapContext
    let trap_cx = task_control_block.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point,
      user_sp,
      KERNEL_SPACE.exclusive_access().token(),
      kernel_stack_top,
      trap_handler as usize,
    );
    task_control_block
  }
}