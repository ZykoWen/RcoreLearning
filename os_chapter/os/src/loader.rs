use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn load_apps() {
  extern "C" { fn _num_app();}
  let num_app_ptr = _num_app as usize as *const usize;
  let num_app = get_num_app();
  //存储每个app的起始地址
  let app_start = unsafe{
    core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
  };
  //加载app
  for i in 0..num_app {
    let base_i = get_base_i(i);
    //清除应用要存入的区域
    (base_i..base_i + APP_SIZE_LIMIT).for_each(|addr| unsafe {
      (addr as *mut u8).write_volatile(0)
    });
    //将数据段上的app加载到内存
    let src = unsafe {
      core::slice::from_raw_parts(
        app_start[i] as *const u8,
        app_start[i + 1] - app_start[i]
      )
    };
    let dst = unsafe {
      core::slice::from_raw_parts_mut(base_i as *mut u8, src.len())
    };
    dst.copy_from_slice(src);
  }
  unsafe{
    asm!("fence.i");
  }
}

///得到appi的基地址
fn get_base_i(app_id: usize) -> usize{
  APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

///得到应用的个数
pub fn get_num_app() -> usize {
  extern "C" {
    fn _num_app();
  }
  unsafe{ (_num_app as usize as *const usize).read_volatile()}
}

///通过应用程序入口地址和用户栈指针来获取app，并在kernel stack上保存TrapContext
pub fn init_app_cx(app_id: usize) -> usize {
  println!("[kernel] spec:{:X}", get_base_i(app_id));
  KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(get_base_i(app_id), USER_STACK[app_id].get_sp()))
}
