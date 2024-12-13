//! Trap handling functionality
mod context;

use crate::{config::{TRAMPOLINE, TRAP_CONTEXT}, syscall::syscall, task::{current_user_token, suspend_current_and_run_next}, timer::set_next_trigger};
use core::arch::global_asm;
use core::arch::asm;
use riscv::register::{
  scause::Interrupt, mtvec::TrapMode, scause::{self,Exception,Trap}, stval, stvec
};
use crate::task::current_trap_cx;
use riscv::register::sie;

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
pub fn trap_handler() -> ! {
  //进入内核后再次触发到 S态 Trap后的处理
  set_kernel_trap_entry();
  //获取当前应用的 Trap 上下文的可变引用
  let cx = current_trap_cx();
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
    Trap::Interrupt(Interrupt::SupervisorTimer) => { //时钟中断处理
      set_next_trigger();
      suspend_current_and_run_next();
    }
    _ => {
       panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
    }
  }
  trap_return();
}

///使 S 特权级时钟中断不会被屏蔽
pub fn enable_timer_interrupt() {
  unsafe { sie::set_stimer(); }
}

fn set_kernel_trap_entry() {
  unsafe {
    stvec::write(trap_from_kernel as usize, TrapMode::Direct)
  }
}

#[no_mangle]
pub fn trap_from_kernel() -> !{
  panic!("a trap from kernel!");
}

fn set_user_trap_entry() {
  unsafe {
    stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
  }
}

#[no_mangle] 
pub fn trap_return() -> ! {
  //让应用 Trap 到 S 的时候可以跳转到 __alltraps 
  set_user_trap_entry();
  //Trap 上下文在应用地址空间中的虚拟地址
  let trap_cx_ptr = TRAP_CONTEXT;
  //要继续执行的应用地址空间的 token
  let user_satp = current_user_token();
  extern "C" {
    fn __alltraps();
    fn __restore();
  }
  // __restore 虚地址
  let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
  unsafe {
    asm!(
      "fence.i", //清空指令缓存i-cache
      "jr {restore_va}", //跳转到 __restore 
      restore_va = in(reg) restore_va,
      in("a0") trap_cx_ptr,
      in("a1") user_satp,
      options(noreturn) //告诉编译器此函数永远不会返回
    );
  }
}


pub use context::TrapContext;