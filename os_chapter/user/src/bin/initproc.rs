//!用户初始化程序

#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{
  fork,
  wait,
  exec,
  yield_,
};

#[no_mangle]
fn main() -> i32 {
  if fork() == 0 {
    exec("user_shell\0"); //rust不会主动加入\0
  } else {
    loop {
      let mut exit_code: i32 = 0;
      let pid = wait(&mut exit_code);
      if pid == -1 {
        yield_();
        continue;
      }
      println!(
        "[initproc] Released a aombie process, pid={}, exit_code={}",
        pid,
        exit_code,
      );
    }
  }
  0
}

