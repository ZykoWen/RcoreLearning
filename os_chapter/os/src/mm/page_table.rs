//! 实现多级页表

use bitflags::*;

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
  fn find_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
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

  ///将页表项拷贝一份并返回
  pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.find_pte(vpn)
        .map(|pte| {pte.clone()})
  }
}