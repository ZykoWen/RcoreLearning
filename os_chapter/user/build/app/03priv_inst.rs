//尝试在用户态执行内核态的特权指令 sret 
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::arch::asm;

#[no_mangle]
fn main() -> i32{
  println!("[user]Try to execute privileges instruction in U mode");
  println!("[user]Kernel should kill this application");
  unsafe{
    asm!("sret");
  }
  0
}