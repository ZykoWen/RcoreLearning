//!文件系统的块缓存层
use super::BlockDevice;
use alloc::sync::Arc;

///块缓存
pub struct BlockCache {
  ///内存中的缓冲区512bytes
  cache: [u8; BLOCK_SZ],
  ///块缓存来自于磁盘中的块的编号
  clock_id: usize,
  ///底层块设备的引用，可通过它进行块读写
  block_device: Arc<dyn BlockDevice>,
  ///记录这个块从磁盘载入内存缓存之后，有没有被修改过
  modified: bool,
}

impl BlockCache {
  ///块上的数据从磁盘读到缓冲区 cache
  pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
    let mut cache: [0u8; BLOCK_SZ];
    block_device.read_block(block_id, &mut cache);
    Self {
      cache,
      block_id,
      block_device,
      modified: false,
    }
  }
  ///得到一个 BlockCache 内部的缓冲区中指定偏移量 offset 的字节地址
  fn addr_of_offset(&self, offset: usize) -> usize {
    &self.cache[offset] as *const _ as usize
  }
  ///获取缓冲区中的位于偏移量 offset 的一个类型为 T 的磁盘上数据结构的不可变引用--T必须是已知大小的类型
  pub fn get_ref<T>(&self, offset: usize) -> &T where T: Sized {
    let type_size = core::mem::size_of::<T>();
    assert!(offset + type_size <= BLOCK_SZ);
    let addr = self.addr_of_offset(offset);
    unsafe{ &*(addr as *const T) }
  }
  ///会获取磁盘上数据结构的可变引用
  pub fn get_mut<T>(&mut self, offset: usize) -> &mut T where T: Sized {
    let type_size = core::mem::size_of::<T>();
    assert!(offset + type_size <= BLOCK_SZ);
    self.modified = true;
    let addr = self.addr_of_offset(offset);
    //*符号在rust中代表解引用或解指针
    unsafe{ &mut *(addr as *mut T) }
  }
  ///判断缓存块是否被修改过，确实被修改过的话才会将缓冲区的内容写回磁盘
  pub fn sync(&mut self) {
    if self.modified {
      self.modified = false;
      self.block_device.write_block(self.block_id, &self.cache);
    }
  }
  ///在 BlockCache 缓冲区偏移量为 offset 的位置获取一个类型为 T 的磁盘上数据结构的不可变引用
  pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
    f(self.get_ref(offset))
  }
  ///在 BlockCache 缓冲区偏移量为 offset 的位置获取一个类型为 T 的磁盘上数据结构的可变引用
  pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V { //为什么T前面不加mut呢？？？
    f(self.get_mut(offset))
  }

}

impl Drop for BlockCache {
  fn drop(&mut self) {
    self.sync()
  }
}
