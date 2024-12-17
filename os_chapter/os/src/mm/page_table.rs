//! 实现多级页表
use super::{frame_alloc, FrameTracker, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::vec::Vec;
use alloc::vec;
use bitflags::*;
use alloc::string::String;


bitflags! {
  ///将u8封装成页表项的标志位的集合类型
  pub struct PTEFlags: u8 {
    const V = 1 << 0;
    const R = 1 << 1;
    const W = 1 << 2;
    const X = 1 << 3;
    const U = 1 << 4;
    const G = 1 << 5;
    const A = 1 << 6;
    const D = 1 << 7;
  }
}

#[derive(Copy,Clone)]
#[repr(C)]
///页表项数据结构
pub struct PageTableEntry {
  pub bits: usize,
}

impl PageTableEntry {
  ///初始化一个页表项
  pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
    PageTableEntry{
      bits: ppn.0 << 10 | flags.bits as usize, //flags.bits是标志的原始位表示(u8类型)
    }
  }
  ///生成一个全零的页表项，隐含着该页表项的 V 标志位为 0 ，因此它是不合法的
  pub fn empty() -> Self {
    PageTableEntry {
      bits: 0,
    }
  }
  ///返回一个页表项包含的物理地址
  pub fn ppn(&self) -> PhysPageNum {
    (self.bits >> 10 & ((1usize << 44) - 1)).into() //44位代表物理页
  }
  ///返回一个页表项包含的10个标志位
  pub fn flags(&self) -> PTEFlags {
    PTEFlags::from_bits(self.bits as u8).unwrap()
  }
  ///判断一个页表项是否合法
  pub fn is_valid(&self) -> bool {
    (self.flags() & PTEFlags::V) != PTEFlags::empty() //仅仅保留PTEFlags中的V标志位
  }
  ///判断一个页表项是否可读
  pub fn readable(&self) -> bool {
    (self.flags() & PTEFlags::R) != PTEFlags::empty()
  }
  ///判断一个页表项是否可写
  pub fn writable(&self) -> bool {
    (self.flags() & PTEFlags::W) != PTEFlags::empty()
  }
  ///判断一个页表项是否可执行
  pub fn executable(&self) -> bool {
    (self.flags() & PTEFlags::X) != PTEFlags::empty()
  }
}

///页表数据结构
pub struct PageTable {
  //页表根节点的物理页号
  root_ppn: PhysPageNum,
  //页表所有节点所在的物理页帧
  frames: Vec<FrameTracker>,
}

impl PageTable {

  ///新建页表--只需要有一个根节点
  pub fn new() -> Self {
    let frame = frame_alloc().unwrap();
    PageTable {
      root_ppn: frame.ppn,
      frames: vec![frame]
    }
  }

  ///在多级页表找到一个虚拟页号对应的页表项的可变引用。如果在遍历的过程中发现有节点尚未创建则会新建一个节点
  fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes(); //获取虚拟页号的三级页索引
    let mut ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for i in 0..3 {
      let pte = &mut ppn.get_pte_array()[idxs[i]];
      if i == 2 {
        result = Some(pte); //遍历到叶子节点--找到对应的虚拟页号
        break;
      }
      if !pte.is_valid() { //页表项不合法
        let frame = frame_alloc().unwrap();
        *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
        self.frames.push(frame);
      }
      ppn = pte.ppn(); //更新ppn为下一级页表的首地址的物理页号
    }
    result
  }

  ///寻找页表项--不会新建节点
  fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes(); //获取虚拟页号的三级页索引
    let mut ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for i in 0..3 {
      let pte = &mut ppn.get_pte_array()[idxs[i]];
      if i == 2 {
        result = Some(pte); //遍历到叶子节点--找到对应的虚拟页号
        break;
      }
      if !pte.is_valid() { //页表项不合法
        return None;
      }
      ppn = pte.ppn(); //更新ppn为下一级页表的首地址的物理页号
    }
    result
  }
  ///在多级页表中插入一个键值对
  pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags){
    let pte = self.find_pte_create(vpn).unwrap();
    assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
    *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
  }

  ///在多级页表中删除一个键值对
  pub fn unmap(&mut self, vpn:VirtPageNum){
    let pte = self.find_pte(vpn).unwrap();
    assert!(pte.is_valid(),"vpn {:?} is invalid before unmapping", vpn);
    *pte = PageTableEntry::empty();
  }

  ///临时创建一个专用来手动查页表的 PageTable
  pub fn from_token(satp: usize) -> Self {
    Self {
      //satp中的低44位表示页表根的物理页号
      root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
      frames: Vec::new(),
    }
  }

  ///将给定虚拟地址对应的页表项拷贝一份并返回
  pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.find_pte(vpn)
        .map(|pte| *pte)
  }
  ///给定虚拟地址，返回物理地址
  pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
    self.find_pte(va.clone().floor()).map(|pte|{
      let aligned_pa: PhysAddr = pte.ppn().into();
      let offset = va.page_offset();
      let aligned_pa_usize: usize = aligned_pa.into();
      (aligned_pa_usize + offset).into()
    })
  }

  ///构造一个无符号64位整数，使得分页模式为SV39
  pub fn token(&self) -> usize {
    8usize << 60 | self.root_ppn.0
  }
}

///将应用地址空间中一个缓冲区转化为在内核空间中能够直接访问的形式的辅助函数
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
  let page_table = PageTable::from_token(token);
  let mut start = ptr as usize;
  let end = start + len;
  let mut v = Vec::new();
  while start < end {
    let start_va = VirtAddr::from(start);
    let mut vpn = start_va.floor();
    let ppn = page_table
        .translate(vpn)
        .unwrap()
        .ppn();
    vpn.step();
    let mut end_va: VirtAddr = vpn.into();
    //确保 end_va 不超过数据的实际结束位置
    end_va = end_va.min(VirtAddr::from(end));
    if end_va.page_offset() == 0 {
      v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);//获取物理页面一整页的内容
    } else {
      v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()])
    }
    start = end_va.into();
  }
  v
}

///构造字符串，直到发现一个 \0 为止
pub fn translated_str(token: usize, ptr: *const u8) -> String {
  let page_table = PageTable::from_token(token);
  let mut string = String::new();
  let mut va = ptr as usize;
  loop {
    let ch: u8 = *(page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut());
    if ch == 0 {
      break;
    } else {
      string.push(ch as char);
      va += 1;
    }
  }
  string
}

///通过页表转换虚拟地址（VA）为物理地址（PA），并返回一个可变的引用
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
  let page_table = PageTable::from_token(token);
  let va = ptr as usize;
  page_table
    .translate_va(VirtAddr::from(va))
    .unwrap()
    .get_mut()
}