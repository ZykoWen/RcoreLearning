//!实现物理页帧管理器

use crate::sync::UPSafeCell;
use crate::config::MEMORY_END;

///描述一个物理页帧管理器需要提供哪些功能
trait FrameAllocator {
  ///创建一个物理页帧管理器实例
  fn new() -> Self;
  ///以页号为单位进行物理页帧的分配和回收
  fn alloc(&mut self) -> Option<PhysPageNum>;
  fn dealloc(&mut self, ppn: PhysPageNum);
}

///实现栈式物理页帧管理器
pub struct StackFrameAllocator {
  // [ current , end ) 的物理页号此前均从未被分配出去过
  current: usize,
  end: usize, 
  //recycled保存了被回收的物理页号
  recycled: Vec<usize>, 
}

impl FrameAllocator for StackFrameAllocator {
  ///创建栈式物理页帧管理器实例
  fn new() -> Self {
    Self{
      current: 0,
      end: 0,
      recycled: Vec::new(),
    }
  }
  ///物理页帧分配
  fn alloc(&mut self) -> Option<PhysPageNum> {
    //检查栈 recycled 内有没有之前回收的物理页号
    if let Some(ppn) = self.recycled.pop() {
      Some(ppn.into())
    } else {
      if self.current == self.end {
        None
      } else {
        self.current += 1;
        Some((self.current - 1).into())
      }
    }
  }
  ///物理页帧回收
  fn dealloc(&mut self, ppn: PhysPageNum) {
    let ppn = ppn.0;
    //检查回收页面的合法性--该页面之前一定被分配出去过或没有正处在回收状态
    if ppn >= self.current || self.recycled.iter().find(|&v| {*v == ppn}).is_some() {
      panic!("Frame ppn = {:#x} has not been allocated!", ppn);
    }
    self.recycled.push(ppn);
  }
}

impl StackFrameAllocator {
  pub fn init(&mut self, l:PhysPageNum, r:PhysPageNum) {
    self.current = l.0;
    self.end = r.0;
  }
}

//定义类型别名--可以支持多种内存分配策略（换用不同的物理页帧管理器）
type FrameAllocatorImpl = StackFrameAllocator; 

///创建 StackFrameAllocator 的全局实例 FRAME_ALLOCATOR
lazy_static! {
  //ref关键字表明FRAME_ALLOCATOR 是对 UPSafeCell<FrameAllocatorImpl> 的一个引用
  pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
    UPSafeCell::new(FrameAllocatorImpl::new())
  };
}

///初始化物理页帧全局管理器 FRAME_ALLOCATOR
pub fn init_frame_allocator() {
  extern "C" {
    fn ekernel();
  }
  FRAME_ALLOCATOR
    .exclusive_access()
    .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

pub struct FrameTracker {
  pub ppn: PhysPageNum,
}

impl FrameTracker {
  ///新建一个FrameTracker
  pub fn new(ppn: PhysPageNum) -> Self {
    //物理页帧之前可能被分配过--清空所有字节
    let bytes_array = ppn.get_bytes_array();
    for i in bytes_array {
      *i = 0;
    }
    Self { ppn }
  }
}

///FrameTracker 生命周期结束被编译器回收时候自动执行--回收物理页帧
impl Drop for FrameTracker {
  fn drop(&mut self) {
    frame_dealloc(self.ppn)
  }

}

///分配物理页帧的接口
pub fn frame_alloc -> Option<FrameTracker> {
  FRAME_ALLOCATOR
    .exclusive_access()
    .alloc()
    .map(|ppn| FrameTracker::new(ppn))
}

///回收物理页帧的接口
pub fn frame_dealloc(ppn: PhysPageNum){
  FRAME_ALLOCATOR
    .exclusive_access()
    .dealloc(ppn);
}