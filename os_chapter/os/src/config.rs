//!在rCore中使用的常量

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
//单个页面的大小为4KB,12位字节地址
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;


///返回一个应用的内核栈在内核空间的起始位置
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
  //每个应用程序的栈在内存中有一个页面分隔
  let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
  let bottom = top - KERNEL_STACK_SIZE;
  (bottom, top)
}

pub use crate::board::CLOCK_FREQ;