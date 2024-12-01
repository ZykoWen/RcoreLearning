#![no_std]
#![feature(linkage)]

#[macro_use]  //别忘了引入模块
pub mod console;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"] //将 _start 这段代码编译后的汇编代码中放在一个名为 .text.entry 的代码段
pub extern "C" fn _start() ->! {
  clear_bss();//手动清空需要零初始化的 .bss 段
  exit(main());
  panic!("unreachable after sys_exit!")
}
//在最后链接的时候，虽然在 lib.rs 和 bin 目录下的某个应用程序都有 main 符号，但由于 lib.rs 中的 main 符号是弱链接，链接器会使用 bin 目录下的应用主逻辑作为 main 。
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
  panic!("Cannot find main!");
}
fn clear_bss() {
    //找到全局符号 sbss 和 ebss--指出需要被清零的 .bss 段的起始和终止地址
    extern "C" {
        fn sbss();
        fn ebss();
    }
    //遍历该地址区间并逐字节进行清零
    (sbss as usize..ebss as usize).for_each(|a|{
        unsafe {(a as *mut u8).write_volatile(0)}
    });
}
use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize{sys_write(fd,buf)}
pub fn exit(exit_code: i32) -> isize{sys_exit(exit_code)}
//因为yield是rust的关键字--所以接口名不能为yield
pub fn yield_() -> isize {sys_yield()}