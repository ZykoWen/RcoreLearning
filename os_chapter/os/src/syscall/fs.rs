//!文件和文件系统相关的系统调用
//! 实现与IO相关的系统调用
use crate::mm::translated_byte_buffer;
use crate::task::{current_user_token, suspend_current_and_run_next};
use crate::sbi::console_getchar;

const FD_STDOUT: usize = 1; //文件描述符（file descriptor）1，即标准输出。
const FD_STDIN: usize = 0; //标准输入 FD_STDIN

///将len长度的buf写入一个文件，通过标识符fd
pub fn sys_write(fd: usize, buf: *const u8, len:usize) -> isize{
  match fd {
    FD_STDOUT => {
      let buffers = translated_byte_buffer(current_user_token(), buf, len);
      for buffer in buffers {
        print!("{}", core::str::from_utf8(buffer).unwrap())
      }
      len as isize
    }
    _ => {
      panic!("Unsupported fd in sys_write!")
    }
  }
}
///通过标识符fd，从缓冲区buf中读取len长度的信息
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
  match fd {
    FD_STDIN => {
      assert_eq!(len, 1, "Only support len=1 in sys_read!");
      let mut c: usize;
      loop {
        c = console_getchar();
        if c == 0 { //表示还没有输入，先切换到别的进程
          suspend_current_and_run_next();
          continue;
        } else {
          break;
        }
      }
      let ch = c as u8;
      //手动查页表将输入的字符正确的写入到应用地址空间
      let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
      unsafe {
        buffers[0].as_mut_ptr().write_volatile(ch);
      }
      1
    }
    _ => {
      panic!("Unsupported fd in sys_read!");
    }
  }
}
