// os/src/main.rs
#![no_std]
#![no_main]
// asm 和 panic_info_message 特性自 Rust 的某些版本之后已经稳定，不再需要使用 #[feature(...)] 属性来启用。
mod batch;
use core::arch::global_asm;

// use sbi::shutdown;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
//通过 include_str! 宏将同目录下的汇编代码 entry.asm 转化为字符串并通过 global_asm! 宏嵌入到代码中。
global_asm!(include_str!("entry.asm"));

#[no_mangle] //避免编译器对它的名字进行混淆
pub fn rust_main() -> !{
    clear_bss();//内核初始化中，需要先完成对 .bss 段的清零
    println!("hello,zyko");
    panic!("Shutdown machine!")
    // shutdown(false)
}
//完成对 .bss 段的清零
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