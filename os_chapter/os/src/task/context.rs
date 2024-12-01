//!实现任务上下文的结构体
#[derive(Copy,Clone)]
#[repr(C)]
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

  ///实现switch过后，进入__restore
  pub fn goto_restore(kstack_ptr: usize) -> Self {
    extern "C" { fn __restore(); }
    Self {
      ra: __restore as usize,
      sp: kstack_ptr,
      s: [0; 12],
    }
  }
}