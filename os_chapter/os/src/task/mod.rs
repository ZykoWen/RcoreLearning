//!实现管理各个任务的任务管理器

mod switch;
mod context;

//对于task模块，Clippy将不会发出关于模块名与其父模块名相同的警告
#[allow(clippy::module_inception)]
mod task;


use crate::loader::{get_app_data, get_num_app};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

///定义任务管理器结构体
pub struct TaskManager {
  ///任务的数量
  num_app: usize,
  ///用inner value
  inner: UPSafeCell<TaskManagerInner>, 
}

///Inner of Task Manager
struct TaskManagerInner {
  ///任务列表
  tasks: Vec<TaskControlBlock>,
  ///正在运行的任务的id
  current_task: usize,
}

//lazy_static! 保证在它第一次被使用到的时候，才会进行实际的初始化工作
lazy_static! {
  ///全局变量TASK_MANAGER
  pub static ref TASK_MANAGER: TaskManager = {
    println!("init TASK_MANAGER");
    let num_app = get_num_app();
    println!("num_app = {}", num_app);
    let mut tasks: Vec<TaskControlBlock> = Vec::new();
    for i in 0..num_app {
      tasks.push(TaskControlBlock::new(
        get_app_data(i),
        i
      ));
    }
    //创建一个TaskManager实例，并返回
    TaskManager {
      num_app,
      inner: unsafe{
        UPSafeCell::new(TaskManagerInner{
          tasks,
          current_task: 0,
        })
      },
    }
  };
}

impl TaskManager {
  ///改变当前运行的任务状态为Ready状态
  fn mark_current_suspended(&self) {
    let mut inner = self.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].task_status = TaskStatus::Ready;
  }

  ///改变当前运行的任务状态为Exited
  fn mark_current_exited(&self) {
    let mut inner = self.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].task_status = TaskStatus::Exited;
  }

  ///执行下一个任务
  fn run_next_task(&self) {
    //找到了下一个Ready的任务
    if let Some(next) = self.find_next_task() {
      let mut inner = self.inner.exclusive_access();
      let current = inner.current_task;
      inner.tasks[next].task_status = TaskStatus::Running;
      inner.current_task = next;
      let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
      let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
      drop(inner);
      unsafe {
        __switch(
          current_task_cx_ptr,
          next_task_cx_ptr
        );
        //回到用户态
      }
    } else { //没有找到Ready的任务
      println!("All applications completed!");
      shutdown(false);
    }
  }

  ///找到下一个准备好的Task的任务号
  fn find_next_task(&self) -> Option<usize> {
    let inner = self.inner.exclusive_access();
    let current = inner.current_task;
    (current + 1..current + self.num_app + 1).map(|id| id % self.num_app).find(|id|{
      inner.tasks[*id].task_status == TaskStatus::Ready})
  }

  ///运行第一个任务
  fn run_first_task(&self) -> ! {
    let mut inner = self.inner.exclusive_access();
    let task0 = &mut inner.tasks[0];
    task0.task_status = TaskStatus::Running;
    let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
    drop(inner);
    //在启动栈上保存了一些之后不会用到的数据
    let mut _unused = TaskContext::zero_init();
    unsafe {
      __switch(
        &mut _unused as *mut _,
        next_task_cx_ptr,
      );
    }
    panic!("unreachable in run_first_task!");
  }
  fn get_current_token(&self) -> usize {
    let inner = self.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].get_user_token()
  }
  fn get_current_trap_cx(&self) -> &'static mut TrapContext {
    let inner = self.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].get_trap_cx()
  }
}




///暂停当前任务并运行下一个任务
pub fn suspend_current_and_run_next() {
  mark_current_suspended();
  run_next_task();
}

///终止当前任务并运行下一个任务
pub fn exit_current_and_run_next() {
  mark_current_exited();
  run_next_task();
}

///运行第一个任务
pub fn run_first_task() {
  TASK_MANAGER.run_first_task();
}

fn run_next_task() {
  TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
  TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
  TASK_MANAGER.mark_current_exited();
}

pub fn current_user_token() -> usize {
  TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
  TASK_MANAGER.get_current_trap_cx()
}
