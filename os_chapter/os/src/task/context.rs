//!实现任务上下文的结构体

use crate::trap::trap_return;
#[derive(Copy,Clone)]
#[repr(C)]
///任务上下文
pub struct TaskContext {
  ///switch返回后的执行位置e.g. __restore
  ra: usize, 
  ///应用的内核栈指针
  sp: usize, 
  ///保存寄存器
  s: [usize; 12], 

}

impl TaskContext {
  ///初始化task context 
  pub fn zero_init() -> Self {
    Self {
      ra: 0,
      sp: 0,
      s: [0; 12],
    }
  }

  ///根据应用的在内核空间的内核栈栈顶指针返回任务上下文
  pub fn goto_trap_return(kstack_ptr: usize) -> Self {
    Self { 
      ra: trap_return as usize,
      sp: kstack_ptr,
      s: [0; 12],
    }
  }
}