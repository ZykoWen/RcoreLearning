//! Trap handling functionality
mod context;

use crate::syscall::syscall;
use core::arch::global_asm;
use riscv::register::{
  mtvec::TrapMode,
  scause::{self,Exception,Trap},
  stval,stvec,
};

global_asm!(include_str!("trap.S"));

///初始化CSR'stvec'为__alltraps的入口
pub fn init(){
  extern "C" {
    fn __alltraps();
  }
  unsafe {
    stvec::write(__alltraps as usize, TrapMode::Direct);//将Trap处理代码的入口地址写进stvec
  }
}

#[no_mangle]
///处理一个来自用户态的中断请求、系统调用
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext{
  let scause = scause::read(); //得到trap原因
  let stval = stval::read(); //保存trap的附加信息
  match scause.cause(){
    Trap::Exception(Exception::UserEnvCall) => {
      cx.sepc += 4;
      //为什么要用x[10]
      cx.x[10] = syscall(cx.x[17], [cx.x[10],cx.x[11],cx.x[12]]) as usize;
    }
    Trap::Exception(Exception::StoreFault)|
    Trap::Exception(Exception::StorePageFault) => {
      println!("[kernel] PageFault in application, kernel killed it.");
      panic!("[kernel] Cannot continue!");
      // run_next_app();
    }
    Trap::Exception(Exception::IllegalInstruction) => {
      println!("[kernel] IllegalInstruction in application, kernel killed it.");
      panic!("[kernel] Cannot continue!");
      // run_next_app();
    }
    _ => {
       panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
    }
  }
  cx
}
pub use context::TrapContext;