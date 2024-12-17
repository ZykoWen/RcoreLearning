#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]


#[macro_use]  //别忘了这个标志
pub mod console;
mod lang_items;
mod syscall;

use syscall::*;

use buddy_system_allocator::LockedHeap;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handler_alloc_error(layout: core::alloc::Layout) -> ! {
  panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"] //将 _start 这段代码编译后的汇编代码中放在一个名为 .text.entry 的代码段
pub extern "C" fn _start() -> ! {
  unsafe {
    HEAP.lock()
        .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
  }
  exit(main());
}
//在最后链接的时候，虽然在 lib.rs 和 bin 目录下的某个应用程序都有 main 符号，但由于 lib.rs 中的 main 符号是弱链接，链接器会使用 bin 目录下的应用主逻辑作为 main 。
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
  panic!("Cannot find main!");
}

pub fn write(fd: usize, buf: &[u8]) -> isize{
  sys_write(fd,buf)
}
pub fn exit(exit_code: i32) -> ! {
  sys_exit(exit_code)
}
//因为yield是rust的关键字--所以接口名不能为yield
pub fn yield_() -> isize {
  sys_yield()
}
///获得系统时间
pub fn get_time() -> isize {
  sys_get_time()
}
///得到进程的标识符
pub fn getpid() -> isize {
    sys_getpid()
}
///创建一个子进程
pub fn fork() -> isize {
    sys_fork()
}
///将当前进程的地址空间清空并加载一个特定的可执行文件，返回用户态后开始它的执行
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
///等待任意一个子进程结束
pub fn wait(exit_code: &mut i32) -> isize {
  loop {
    match sys_waitpid(-1, exit_code as *mut _) {
      -2 => {
        yield_();
      }
      exit_pid => return exit_pid,
    }
  }
}
///等待一个进程标识符的值为pid 的子进程结束
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
  loop {
    match sys_waitpid( pid as isize, exit_code as *mut _) {
      -2 => {yield_();}
      exit_pid=> return exit_pid,
    }
  }
}
///从文件中读取一段内容到缓冲区
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
  sys_read(fd, buf)
}
///让进程休眠
pub fn sleep(period_ms: usize) {
  let start = sys_get_time();
  while sys_get_time() < start + period_ms as isize {
    sys_yield();
  }
}