//!实现管理各个任务的任务管理器

mod switch;
mod context;

//对于task模块，Clippy将不会发出关于模块名与其父模块名相同的警告
#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app,init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
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
  tasks: [TaskControlBlock; MAX_APP_NUM],
  ///正在运行的任务的id
  current_task: usize,
}

//lazy_static! 保证在它第一次被使用到的时候，才会进行实际的初始化工作
lazy_static! {
  ///全局变量TASK_MANAGER
  pub static ref TASK_MANAGER: TaskManager = {
    let num_app = get_num_app();
    //创建一个初始化的tasks数组
    let mut tasks = [
      TaskControlBlock {
        task_cx: TaskContext::zero_init(),
        task_status: TaskStatus::UnInit //初始化任务控制块的运行状态为“尚未初始化”
      };
      MAX_APP_NUM
    ];
    //依次对每一个任务控制块进行初始化
    for i in 0..num_app {
      tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
      tasks[i].task_status = TaskStatus::Ready;
    }
    //创建一个TaskManager实例，并返回
    TaskManager {
      num_app,
      inner: unsafe {
        UPSafeCell::new(TaskManagerInner{
          tasks,
          current_task: 0,
        })
      }
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
        &mut _unused as *mut TaskContext,
        next_task_cx_ptr,
      );
    }
    panic!("unreachable in run_first_task!");
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
