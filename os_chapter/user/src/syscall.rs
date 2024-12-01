//！实现之间的应用程序和批处理系统系统调用
use core::arch::asm;

///定义通用系统调用函数
fn syscall(id:usize,args:[usize;3])->isize{
  let mut ret: isize;//保存系统调用返回值
  //asm! 宏可以获取上下文中的变量信息并允许嵌入的汇编代码对这些变量进行操作
  unsafe{
    asm!(
      "ecall",//触发系统调用
      //编译器自动插入相关指令并保证在 ecall 指令执行之前，以下寄存器被赋值
      inlateout("x10") args[0] => ret, //a0 寄存器，它同时作为输入和输出
      in("x11") args[1],
      in("x12") args[2],
      in("x17") id //用来传递 syscall ID，这是因为所有的 syscall 都是通过 ecall 指令触发的，除了各输入参数之外我们还额外需要一个寄存器来保存要请求哪个系统调用
    );
  }
  ret
}

//系统调用号，用于标识写操作的系统调用
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;

/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的起始地址；
///      `len` 表示内存中缓冲区的长度。
/// 返回值：返回成功写入的长度。
/// syscall ID：64
pub fn sys_write(fd: usize,buffer: &[u8]) -> isize{
  syscall(SYSCALL_WRITE,[fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`exit_code` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(xstate: i32) -> isize {
  syscall(SYSCALL_EXIT,[xstate as usize,0,0])
}

/// 功能：应用主动交出 CPU 所有权并切换到其他应用。
/// 返回值：总是返回 0。
/// syscall ID：124
pub fn sys_yield() -> isize {
  syscall(SYSCALL_YIELD, [0,0,0])
}