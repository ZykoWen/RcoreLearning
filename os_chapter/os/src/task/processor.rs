use crate::{sync::UPSafeCell, trap::TrapContext};

use super::{switch::__switch, task::{TaskControlBlock, TaskStatus}, TaskContext};
use alloc::sync::Arc;
use lazy_static::*;
use super::manager::fetch_task;

///描述CPU执行状态的数据结构
pub struct Processor {
  ///在当前处理器上正在执行的任务
  current: Option<Arc<TaskControlBlock>>,
  ///当前处理器上的 idle 空闲控制流的任务上下文
  //idle控制流的功能：尝试从任务管理器中选出一个任务来在当前 CPU 核上执行
  idle_task_cx: TaskContext,
}

impl Processor {
  pub fn new() -> Self {
    Self {
      current: None,
      idle_task_cx: TaskContext::zero_init(),
    }
  }
  ///获取指向idle控制流的指针
  fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
    &mut self.idle_task_cx as *mut _
  }
}

lazy_static! {
  ///全局CPU执行状态的数据结构--因为单核cpu，所以只有一个
  pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
    UPSafeCell::new(Processor::new())
  };
}

impl Processor {
  ///获取正在运行的进程
  pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
    self.current.take()
  }
  ///返回当前执行的进程的一份拷贝
  pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
    self.current.as_ref().map(|task| Arc::clone(task))
  }
}

///获取正在运行的进程
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
  PROCESSOR.exclusive_access().take_current()
}

///获取当前执行的进程的一份拷贝
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
  PROCESSOR.exclusive_access().current()
}

///获取当前应用地址空间的token
pub fn current_user_token() -> usize {
  let task = current_task().unwrap();
  let token = task.inner_exclusive_access().get_user_token();
  token
}

///获取当前正在执行任务的Trap上下文
pub fn current_trap_cx() -> &'static mut TrapContext {
  current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

///内核初始化完毕之后，会通过调用该函数来进入 idle 控制流
pub fn run_tasks() {
  loop {
    let mut processor = PROCESSOR.exclusive_access();
    //直到顺利从任务管理器中取出一个任务，随后便准备通过任务切换的方式来执行
    if let Some(task) = fetch_task() {
      //获取当前 idle 控制流的 task_cx_ptr
      let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
      let mut task_inner = task.inner_exclusive_access();
      //从任务管理器中取出对应的任务控制块
      let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
      task_inner.task_status = TaskStatus::Running;
      drop(task_inner);
      //修改当前 Processor 正在执行的任务为取出的任务
      processor.current = Some(task);
      drop(processor);
      unsafe {
        //从当前的 idle 控制流切换到接下来要执行的任务
        __switch(
          idle_task_cx_ptr, 
          next_task_cx_ptr,
        );
      }
    }
  }
}

///内核切换到idle控制流的函数（当一个应用主动或被动交出cpu使用权时）
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
  let mut processor = PROCESSOR.exclusive_access();
  let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
  drop(processor);
  unsafe {
    __switch(
      switched_task_cx_ptr, 
      idle_task_cx_ptr,
    );
  };
}