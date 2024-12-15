use alloc::vec::Vec;
use lazy_static::*;
use crate::sync::UPSafeCell;
use crate::mm::{MapPermission, VirtAddr, KERNEL_SPACE};
use crate::config::{PAGE_SIZE, TRAMPOLINE, KERNEL_STACK_SIZE};
///对进程标识符的封装
pub struct PidHandle(pub usize);

///进程标识符分配器数据结构
struct PidAllocator {
  current: usize,
  recycled: Vec<usize>,
}

impl PidAllocator {
  ///初始化进程标识符分配器
  pub fn new() -> Self {
    PidAllocator {
      current: 0,
      recycled: Vec::new(),
    }
  }
  ///分配进程标识符并返回
  pub fn alloc(&mut self) -> PidHandle {
    if let Some(pid) = self.recycled.pop() {
      PidHandle(pid)
    } else {
      self.current += 1;
      PidHandle(self.current - 1)
    }
  }
  ///回收进程标识符
  pub fn dealloc(&mut self, pid: usize) {
    assert!(pid < self.current);
    assert!(
      self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
      "pid {} has been deallocated!", pid
    );
    self.recycled.push(pid);
  }
}

lazy_static! {
  ///创建全局进程标识符分配器
  static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = unsafe {
    UPSafeCell::new(PidAllocator::new())
  };
}

///分配进程标识符的接口
pub fn pid_alloc() -> PidHandle {
  PID_ALLOCATOR.exclusive_access().alloc()
}

impl Drop for PidHandle {
  fn drop(&mut self) {
    PID_ALLOCATOR.exclusive_access().dealloc(self.0);
  }
}

//进程的内核栈
pub struct KernelStack {
  pid: usize,
}

impl KernelStack {
  ///根据进程标识符创建一个内核栈
  pub fn new(pid_handle: &PidHandle) -> Self {
    let pid = pid_handle.0;
    let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
    //将内核栈逻辑段加入到内核地址空间
    KERNEL_SPACE
        .exclusive_access()
        .insert_frames_area(
          kernel_stack_bottom.into(),
          kernel_stack_top.into(),
          MapPermission::R |MapPermission::W,
        );
    KernelStack{
      pid: pid_handle.0,
    }
  }
  ///将一个类型为 T 的变量压入内核栈顶并返回其裸指针
  pub fn push_on_top<T>(&self, value: T) -> *mut T where T: Sized, {
    let kernle_stack_top = self.get_top();
    let ptr_mut = (kernle_stack_top - core::mem::size_of::<T>()) as *mut T;
    unsafe { *ptr_mut = value; }
    ptr_mut
  }
  ///获取当前内核栈顶在内核地址空间中的地址
  pub fn get_top(&self) -> usize {
    let (_, kernel_stack_top) = kernel_stack_position(self.pid);
    kernel_stack_top
  }
}

impl Drop for KernelStack {
  ///内核栈的生命周期结束则在内核地址空间中将对应的逻辑段删除
  fn drop(&mut self) {
      let (kernel_stack_bottom,_) = kernel_stack_position(self.pid);
      let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
      KERNEL_SPACE
        .exclusive_access()
        .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
  }
}
///根据进程标识符计算内核栈在内核地址空间中的位置
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
  let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
  let bottom = top - KERNEL_STACK_SIZE;
  (bottom, top)
}
