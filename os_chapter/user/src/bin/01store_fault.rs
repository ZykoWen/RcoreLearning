#![no_std]
#![no_main]
#[macro_use]
extern crate user_lib;
#[no_mangle]
fn main() -> i32 {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe {
        core::ptr::null_mut::<u8>().write_volatile(0);
        //core::ptr::null_mut::<u8>() 创建了一个指向 u8 类型的空指针（即 0x00 地址）。write_volatile 方法将值 0 写入这个地址。这是一个非法操作，因为对空指针的解引用是未定义行为，而且 write_volatile 通常用于硬件寄存器的写入，而不是用于常规内存操作。
    }
    0
}