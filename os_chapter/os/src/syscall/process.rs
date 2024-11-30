//！实现任务处理相关的 syscall
use crate::batch::run_next_app;

pub fn sys_exit(xstate: i32) -> !{
  //打印退出的应用程序的返回值
  println!("[kernel] Application exited with code {}", xstate);
  run_next_app()
}