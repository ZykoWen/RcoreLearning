//!实现管理各个任务的任务管理器

mod switch;
mod context;
mod pid;
mod manager;
mod processor;

//对于task模块，Clippy将不会发出关于模块名与其父模块名相同的警告
#[allow(clippy::module_inception)]
mod task;


use alloc::sync::Arc;
use lazy_static::*;
use crate::sbi::shutdown;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::get_app_data_by_name;
pub use manager::add_task;
pub use processor::{take_current_task, current_user_token,current_task, schedule, current_trap_cx, run_tasks};

pub use context::TaskContext;


lazy_static!{
  ///初始进程的进程控制块
  pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
    TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
  );
}
///将初始进程添加到任务控制器
pub fn add_initproc() {
  add_task(INITPROC.clone());
}


///暂停当前任务并运行下一个任务
pub fn suspend_current_and_run_next() {
  let task = take_current_task().unwrap();
  //获取当前TCB的权限
  let mut task_inner = task.inner_exclusive_access();
  let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
  task_inner.task_status = TaskStatus::Ready;
  drop(task_inner);
  //重新放回就绪队列
  add_task(task);
  //切换任务
  schedule(task_cx_ptr);
}

///空闲进程的pid
pub const IDLE_PID: usize = 0;

///终止当前任务并运行下一个任务
pub fn exit_current_and_run_next(exit_code: i32) {
  //从处理器监控 PROCESSOR取出当前任务
  let task = take_current_task().unwrap();
  let pid = task.getpid();
  //空闲进程退出是一种特殊情况，通常意味着系统关机或出现异常
  if pid == IDLE_PID {
    println!("[kernel] Idle process exit with exit_code {} ...", exit_code);
    if exit_code != 0 {
      shutdown(true); //非正常退出
    } else {
      shutdown(false); //正常退出
    }
  }

  //获取可变部分
  let mut inner = task.inner_exclusive_access();
  //转换当前进程为僵尸进程
  inner.task_status = TaskStatus::Zombie;
  //将传入的退出码写入TCB
  inner.exit_code = exit_code;
  //将当前进程的所有子进程都挂在初始进程initproc下
  {
    let mut initproc_inner = INITPROC.inner_exclusive_access();
    for child in inner.children.iter() {
      child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
      initproc_inner.children.push(child.clone());
    }
  }
  //清空当前进程的所有子进程
  inner.children.clear();
  //对当前进程占用的用户空间进行释放
  inner.memory_set.recycled_data_pages();
  drop(inner);
  drop(task); //存放页表的那些物理页帧会由父进程回收
  //无需关心任务上下文的保存
  let mut _unused = TaskContext::zero_init();
  //触发调度及任务切换
  schedule(&mut _unused as *mut _);
}
