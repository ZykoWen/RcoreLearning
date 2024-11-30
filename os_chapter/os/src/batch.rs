//! 实现一个简单的批处理系统
//！该模块功能：实现能找到并加载应用程序二进制码的应用管理器 AppManager
//！1.保存应用数量和各自的位置信息，以及当前执行到第几个应用了。
//！2.根据应用程序位置信息，初始化好应用所需内存空间，并加载应用执行。

use crate::sbi::shutdown;
use crate::sync::UPSafeCell; //使用绝对路径来引入模块
use crate::trap::TrapContext;
use core::arch::asm;
use lazy_static::*;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

///实现应用管理器 AppManager 结构体
struct AppManager {
  num_app: usize,
  current_app: usize, //表示当前执行的是第几个应用--在运行过程中要可修改（内部可变性）
  app_start: [usize; MAX_APP_NUM + 1],
}
impl AppManager{
    ///打印应用信息
    pub fn print_app_info(&self){
        println!("[kernel] num_app = {}",self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} ({:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i+1] 
            );
        }
    }
    ///获取当前应用
    pub fn get_current_app(&self) -> usize{
        self.current_app
    }
    ///移动到下一个应用
    pub fn move_to_next_app(&mut self){
        self.current_app += 1;
    }
    ///加载应用
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}", app_id);
        // clear app area
        unsafe{
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
            let app_src = core::slice::from_raw_parts(
                self.app_start[app_id] as *const u8,
                self.app_start[app_id + 1] - self.app_start[app_id],
            );
            let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src); //将应用加载到对应的内核位置
        }
        // Memory fence about fetching the instruction memory
        // It is guaranteed that a subsequent instruction fetch must
        // observes all previous writes to the instruction memory.
        // Therefore, fence.i must be executed after we have loaded
        // the code of the next app into the instruction memory.
        // See also: riscv non-priv spec chapter 3, 'Zifencei' extension.
        asm!("fence.i");
    }
}



//初始化 AppManager 的全局实例 APP_MANAGER
//lazy_static! 宏提供了全局变量的运行时初始化功能,保证APP_MANAGER的全局实例只有在它第一次被使用到的时候，才会进行实际的初始化工作
lazy_static!{
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
        extern "C" { fn _num_app();} //找到 link_app.S 中提供的符号 _num_app ，并从这里开始解析出应用数量以及各个应用的起始地址
        //将 _num_app 的地址（函数指针）转为一个 usize 类型，然后再转为一个指向 usize 的裸指针 *const usize。
        let num_app_ptr = _num_app as usize as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start : [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1]; //初始化每个app的起始地址
        let app_start_raw: &[usize] = core::slice::from_raw_parts(
            num_app_ptr.add(1),num_app + 1
        );
        app_start[..=num_app].copy_from_slice(app_start_raw);
        AppManager{
            num_app,
            current_app: 0,
            app_start,
        }
    })
    };
}
//创建用户栈和内核栈
const USER_STACK_SIZE: usize = 4096*2;
const KERNEL_STACK_SIZE: usize = 4096*2;
//repr属性用于指定结构体或枚举的内存布局
//确保结构体的实例会按照 4096 字节对齐存储。
#[repr(align(4096))]
struct KernelStack{
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack{
    data: [u8; USER_STACK_SIZE],
}
//创建全局变量--存储在批处理操作系统的 .bss 段中的
static KERNEL_STACK: KernelStack = KernelStack {data:[0; KERNEL_STACK_SIZE]};
static USER_STACK: UserStack = UserStack{ data: [0; KERNEL_STACK_SIZE]};

//有点不太懂这里？？？
impl UserStack {
    //计算和获取栈的初始栈顶位置
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>())as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe{ cx_ptr.as_mut().unwrap()}
    }
}

///初始化batch subsystem
pub fn init() {
    print_app_info();
}

///打印应用程序的信息
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn run_next_app() -> !{
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe{
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(APP_BASE_ADDRESS,USER_STACK.get_sp()))as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}