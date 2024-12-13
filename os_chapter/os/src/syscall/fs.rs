//!文件和文件系统相关的系统调用
//! 实现与IO相关的系统调用
use crate::mm::translated_byte_buffer;
use crate::task::current_user_token;

const FD_STDOUT: usize = 1; //文件描述符（file descriptor）1，即标准输出。

///将len长度的buf写入一个文件，通过标识符fd
pub fn sys_write(fd: usize, buf: *const u8, len:usize) -> isize{
  match fd {
    FD_STDOUT => {
      // let slice = unsafe { 
      // //将原始指针 buf 和长度 len 转换为一个切片  
      // core::slice::from_raw_parts(buf,len)};
      // //字节切片解释为 UTF-8 编码的字符串
      // let str = core::str::from_utf8(slice).unwrap();
      // print!("{}",str);
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