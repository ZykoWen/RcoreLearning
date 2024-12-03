//!将内核与 RustSBI 通信的相关功能实现在子模块 sbi 中

///输出字符串功能
pub fn console_putchar(c: usize){
  #[allow(deprecated)]
  sbi_rt::legacy::console_putchar(c);
}
///关机功能
pub fn shutdown(failure: bool) -> ! { //参数 failure 表示系统是否正常退出
  use sbi_rt::{system_reset,NoReason,Shutdown,SystemFailure};
  if !failure {
    system_reset(Shutdown,NoReason);
  }else{
    system_reset(Shutdown, SystemFailure);
  }
  unreachable!();
}

///设置时间
pub fn set_timer(timer: usize) {
  sbi_rt::set_timer(timer as _);
}
