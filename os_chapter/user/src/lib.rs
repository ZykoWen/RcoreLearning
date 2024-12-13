#![no_std]
#![feature(linkage)]

#[macro_use]  //别忘了这个标志
pub mod console;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"] //将 _start 这段代码编译后的汇编代码中放在一个名为 .text.entry 的代码段
pub extern "C" fn _start() ->! {
  exit(main());
  panic!("unreachable after sys_exit!")
}
//在最后链接的时候，虽然在 lib.rs 和 bin 目录下的某个应用程序都有 main 符号，但由于 lib.rs 中的 main 符号是弱链接，链接器会使用 bin 目录下的应用主逻辑作为 main 。
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
  panic!("Cannot find main!");
}
use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize{
  sys_write(fd,buf)
}
pub fn exit(exit_code: i32) -> isize{
  sys_exit(exit_code)
}
//因为yield是rust的关键字--所以接口名不能为yield
pub fn yield_() -> isize {
  sys_yield()
}
pub fn get_time() -> isize {
  sys_get_time()
}