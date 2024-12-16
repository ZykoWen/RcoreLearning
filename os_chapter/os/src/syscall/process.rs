use crate::loader::get_app_data_by_name;
//！实现任务处理相关的 syscall
use crate::task::{add_task, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;
use crate::mm::translated_str;


pub fn sys_exit(exit_code: i32) -> ! {
  println!("[kernel] Application exited with code {}", exit_code);
  exit_current_and_run_next(exit_code);
  panic!("Unreachable in sys_exit!");
}
pub fn sys_yield() -> isize {
  suspend_current_and_run_next();
  0
}
pub fn sys_get_time() -> isize {
  get_time_ms() as isize
}
pub fn sys_fork() -> isize {
  let current_task = current_task().unwrap();
  let new_task = current_task.fork();
  let new_pid = new_task.pid.0;
  let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
  //a0是通用的函数调用参数寄存器，
  //以下语句表示子进程在执行 fork 后会返回 0
  trap_cx.x[10] = 0;
  //将新任务添加到任务管理器中
  add_task(new_task);
  new_pid as isize
}
///exec函数参数：要执行的应用名字符串在当前应用地址空间中的起始地址
pub fn sys_exec(path: *const u8) -> isize {
  let token = current_user_token();
  let path = translated_str(token, path);
  if let Some(data) = get_app_data_by_name(path.as_str()) {
    let task = current_task().unwrap();
    task.exec(data);
    0
  } else {
    -1
  }
}
///父进程通过 sys_waitpid 系统调用来回收子进程的资源并收集它的一些信息
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
  let task = current_task().unwrap();
  //找进程号为pid的子进程
  let mut inner = task.inner_exclusive_access();
  //当传入的 pid 为 -1 的时候，任何一个子进程都算是符合要求；
  //但 pid 不为 -1 的时候，则只有 PID 恰好与 pid 相同的子进程才算符合条件
  if inner.children
    .iter()
    .find(|p| {pid == -1 || pid as usize == p.getpid()})
    .is_none() {
      return -1;
    }
    //判断符合要求的子进程中是否有僵尸进程
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)|{
          p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });
    
}