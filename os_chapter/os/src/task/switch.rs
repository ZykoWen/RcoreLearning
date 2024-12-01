//! 将汇编代码中的全局符号 __switch 解释为一个 Rust 函数
use core::arch::asm;

global_asm!(include_str!("switch.S"));

use super::TaskContext;

//extern "C"确保Rust编译器不会对C函数的名称进行修饰，从而可以与C代码兼容
extern "C" {
  pub fn __switch(
    current_task_cx_ptr: *mut TaskContext,
    next_task_cx_ptr: *const TaskContext
  );
}