//!创建子进程并让子进程执行一些操作后退出，父进程进行检查

#![no_std]
#![no_main]

#![macro_use]
extern crate user_lib;
use user_lib::{exit, fork, println, wait, waitpid, yield_};

const MAGIC: i32 = -0x10384;

#[no_mangle]
pub fn main() -> i32 {
  println!("I am the parent. Forking the child...");
  let pid = fork();
  if pid == 0 {
    println!("I am the child.");
    for _ in 0..7 {
      yield_();
    }
    exit(MAGIC);
  } else {
    println!("I am parent, fork a child pid {}", pid);
  }
  println!("I am parent, waiting now..");
  let mut xstate: i32 = 0;
  //等待子进程结束
  assert!(waitpid(pid as usize, &mut xstate) == pid && xstate == MAGIC);
  //判断还有没有其他子进程
  assert!(waitpid(pid as usize, &mut xstate) < 0 && wait(&mut xstate) <= 0);
  println!("waitpid {} ok.", pid);
  println!("exit pass.");
  0
} 