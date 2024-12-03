use core::panic::PanicInfo;//在核心库中得以保留

#[panic_handler]
fn panic(info:&PanicInfo) -> !{
  if let Some(location) = info.location() {
    println!(
              "Panicked at {}:{} {}",
              location.file(),
              location.line(),
              info.message()  
              //message函数的返回值为PanicMessage，PanicMessage 实现了 Debug 或 Display，所以不再直接提供解包方法，例如 unwrap
            );      
  }else{
    println!("Panicked: {}",info.message());
  }
  loop{}
}