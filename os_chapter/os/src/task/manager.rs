use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::sync::UPSafeCell;

use super::TaskControlBlock;
use lazy_static::*;

///任务管理器--仅仅管理所有任务
pub struct TaskManager {
  ///进程就绪队列
  ready_queue: VecDeque<Arc<TaskControlBlock>>, //任务控制块实际上被放置在内核堆上，任务管理器中仅存放他们的引用计数智能指针
}

impl TaskManager {
  ///初始化一个任务管理器
  pub fn new() -> Self {
    Self { ready_queue: VecDeque::new(), }
  }
  ///将一个任务加入队尾
  pub fn add(&mut self, task: Arc<TaskControlBlock>) {
    self.ready_queue.push_back(task);
  }
  ///从队头中取出一个任务
  pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
    self.ready_queue.pop_front()
  }
}

lazy_static! {
  ///创建一个全局任务管理器
  pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = unsafe {
    UPSafeCell::new(TaskManager::new())
  };
}

///添加进程到进程管理器
pub fn add_task(task: Arc<TaskControlBlock>) {
  TASK_MANAGER.exclusive_access().add(task);
}

///从任务管理器中取出一个进程
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
  TASK_MANAGER.exclusive_access().fetch()
}