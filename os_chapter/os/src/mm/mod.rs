mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission,MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, PageTableEntry};
use page_table::{PTEFlags, PageTable};


pub fn init() {
  //初始化动态内存分配器
  heap_allocator::init_heap();
  //初始化物理页帧管理器
  frame_allocator::init_frame_allocator();
  //创建内核地址空间--使cpu开启分页模式
  KERNEL_SPACE.exclusive_access().activate();
}