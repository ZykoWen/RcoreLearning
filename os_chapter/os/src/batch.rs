//该模块功能：实现能找到并加载应用程序二进制码的应用管理器 AppManager
//1.保存应用数量和各自的位置信息，以及当前执行到第几个应用了。
//2.根据应用程序位置信息，初始化好应用所需内存空间，并加载应用执行。

use crate::sbi::shutdown;
use crate::sync::UPSafeCell; //使用绝对路径来引入模块
use core::arch::asm;
use lazy_static::*;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

//应用管理器 AppManager 结构体
struct AppManager {
  num_app: usize,
  current_app: usize, //表示当前执行的是第几个应用--在运行过程中要可修改（内部可变性）
  app_start: [usize; MAX_APP_NUM + 1],
}
impl AppManager{
    pub fn print_app_info(&self){
        println!("[kernel num_app = {}",self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} ({:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i+1] 
            );
        }
    }
    pub fn get_current_app(&self) -> usize{
        self.current_app
    }
    pub fn move_to_next_app(&mut self){
        self.current_app += 1;
    }
    unsafe fn load_app(&self, app_id: usize) {
        if app_id > self.num_app {
            println!("All application completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}",app_id);
        //清除将一块内存清空--创建一个可变切片，用于操作原始指针指向的内存区域
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        //创建一个不可变切片，用于读取应用程序的数据。
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id]
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
        //保证在它之后的取指过程必须能够看到在它之前的所有对于取指内存区域的修改--有时间再好好理解一下
        asm!("fence.i");
    }
}



//初始化 AppManager 的全局实例 APP_MANAGER
//lazy_static! 宏提供了全局变量的运行时初始化功能,保证APP_MANAGER的全局实例只有在它第一次被使用到的时候，才会进行实际的初始化工作
lazy_static!{
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {UPSafeCell::new({
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
    })};
}