use core::fmt::{self, Write};
use super::write;
use super::read;

struct Stdout;
const STDOUT : usize = 1; //1代表标准输出
const STDIN: usize = 0; //0代表标准输入

impl Write for Stdout {
  //在 Write trait 中， write_str 方法必须实现，因此我们需要为 Stdout 实现这一方法
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
///从 标准输入 中获取一个字符
pub fn getchar() -> u8 {
    //声明一个长度为1的缓冲区
    let mut c = [0u8; 1];
    read(STDIN, &mut c);
    c[0]
}