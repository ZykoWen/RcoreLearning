#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
// asm 和 panic_info_message 特性自 Rust 的某些版本之后已经稳定，不再需要使用 #[feature(...)] 属性来启用。

extern crate alloc; //引入 alloc 库的依赖

#[macro_use]
extern crate bitflags;

use core::arch::global_asm;

#[path = "boards/qemu.rs"] //自定义模块文件的路径
mod board;

#[macro_use]
mod console;
mod mm;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;
mod loader;
mod config;
pub mod task;
mod timer;


//通过 include_str! 宏将同目录下的汇编代码 entry.asm 转化为字符串并通过 global_asm! 宏嵌入到代码中。
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

///os的rust入口
#[no_mangle] //避免编译器对它的名字进行混淆
pub fn rust_main() -> !{
    clear_bss();//内核初始化中，需要先完成对 .bss 段的清零
    println!("[kernel]hello,world!");
    mm::init();
    println!("[kernel]back to world!");
    mm::remap_test();
    //将初始进程添加到任务控制器
    task::add_initproc();
    println!("after initproc");
    trap::init();
    trap::enable_timer_interrupt();//避免S特权级时钟中断被屏蔽
    timer::set_next_trigger();//设置下一个中断
    loader::list_apps();//打印所有应用的名字
    task::run_tasks(); //开始执行
    panic!("Unreachable in rust main!");
}

///完成对 .bss 段的清零
fn clear_bss() {
    extern "C" {
        fn sbss(); //sbss 和 ebss指出需要被清零的 .bss 段的起始和终止地址
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a|{
        unsafe {(a as *mut u8).write_volatile(0)}
    });//遍历该地址区间并逐字节进行清零
}
