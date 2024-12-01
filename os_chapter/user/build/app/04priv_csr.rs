//试图在用户态修改内核态 CSR sstatus
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use riscv::register::sstatus::{self, SPP};

#[no_mangle]
fn main() -> i32 {
  println!("[user]Try to access privileged CSR in U Mode");
  println!("[user]Kernel should kill this application");
  unsafe{
    sstatus::set_spp(SPP::User);
  }
  0
}