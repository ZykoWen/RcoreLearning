//!与任务管理相关的数据结构

use super::TaskContext;

///代表任务状态的枚举类型
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus {
  UnInit, //未初始化
  Ready, //准备运行
  Running, //正在运行
  Exited, //已经退出
}

///任务控制块TCB：内核保存一个应用的更多信息的数据结构
#[derive(Copy,Clone)]
pub struct TaskControlBlock {
  pub task_status: TaskStatus, //任务当前状态
  pub task_cx: TaskContext, //任务上下文
}