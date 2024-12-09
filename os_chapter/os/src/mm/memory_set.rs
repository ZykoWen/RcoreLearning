//!内存空间的实现
use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{MEMORY_END, MMIO, PAGE_SIZE, TRAMPOLINE};
use crate::sync::UPIntrFreeCell;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::*;
use riscv::register::satp;

extern "C" {
  fn stext();
  fn etext();
  fn srodata();
  fn erodate();
  fn sdata();
  fn edata();
  fn sbss_with_stack();
  fn ebss();
  fn ekernel();
  fn strampoline();
}
///逻辑段数据结构的实现：描述一段连续地址的虚拟内存
pub struct MapArea {
  //描述一段虚拟页号的连续空间
  vpn_range: VPNRange,
  //该逻辑段中的虚拟页号和物理页号的对应关系
  data_frames: BTreeMap<VirtPageNum, FrameTracker>,
  //该逻辑段内的所有虚拟页面映射到物理页帧的同一种方式
  map_type: MapType,
  //该逻辑段的访问方式
  map_perm: MapPermission,
}

///虚拟页面映射到物理页帧的方式
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
  Identical, //恒等映射
  Framed, //每个虚拟页面都有一个新分配的物理页帧与之对应
}

///逻辑段的访问方式--U/R/W/X 四个标志位
bitflags! {
  pub struct MapPermission: u8 {
    const R = 1 << 1;
    const W = 1 << 2;
    const X = 1 << 3;
    const U = 1 << 4;
  }
}

impl MapArea {
  ///创建一个新的逻辑段
  pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, map_perm: MapPermission) -> Self {
    let start_vpn: VirtPageNum = start_va.floor();
    let end_vpn: VirtPageNum = end_va.ceil();
    Self {
      vpn_range: VPNRange::new(start_vpn, end_vpn),
      data_frames: BTreeMap::new(),
      map_type,
      map_perm,
    }
  }
  ///将当前逻辑段到物理内存的映射从传入的该逻辑段所属的地址空间的多级页表中加入
  pub fn map(&mut self, page_table: &mut PageTable) {
    for vpn in self.vpn_range {
      self.map_one(page_table,vpn);
    }
  }
  ///将当前逻辑段到物理内存的映射从传入的该逻辑段所属的地址空间的多级页表中删除
  pub fn unmap(&mut self, page_table: &mut PageTable) {
    for vpn in self.vpn_range {
      self.unmap_one(page_table, vpn);
    }
  }
  ///将切片 data 中的数据拷贝到当前逻辑段实际被内核放置在的各物理页帧上--以后再看看
  pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
    assert_eq!(self.map_type, MapType::Framed);
    let mut start: usize = 0;
    let mut current_vpn = self.vpn_range.get_start();
    let len = data.len();
    loop {
      let src = &data[start..len.min(start + PAGE_SIZE)]; //尝试取一页的数据如果len <= start + PAGE_SIZE，则取到数据末len
      let dst = &mut page_table
        .translate(current_vpn)
        .unwrap()
        .ppn()
        .get_bytes_array()[..src.len()];
      //将数据复制到对应的物理页帧上
      dst.copy_from_slice(src);
      start += PAGE_SIZE;
      if start >= len {
        break;
      }
      current_vpn.step();
    }
  }
  ///逻辑段中的单个虚拟页面进行映射
  pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
    let ppn: PhysPageNum;
    match self.map_type {
      MapType::Identical => {
        ppn = PhysPageNum(vpn.0);
      }
      MapType::Framed => {
        let frame = frame_alloc().unwrap();
        ppn = frame.ppn;
        self.data_frames.insert(vpn, frame);
      }
    }
    let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
    page_table.map(vpn, ppn, pte_flags);
  }
  ///逻辑段中的单个虚拟页面进行解映射
  pub fn unmap_one(&mut self, pagetable: &mut PageTable, vpn: VirtPageNum) {
    match self.map_type {
      MapType::Framed => {
        self.data_frames.remove(&vpn);
      }
      _ => {}
    }
    page_table.unmap(vpn);
  }
}

///地址空间数据结构：表示一系列有关联的逻辑段
pub struct MemorySet {
  //该地址空间的多级页表--包含多级页表中所有节点所在的物理页帧
  page_table: PageTable,
  //该地址空间的所有逻辑段--包含对应逻辑段中的数据所在的物理页帧
  areas: Vec<MapArea>,
}

impl MemorySet {
  ///新建一个空的地址空间
  pub fn new_bare() -> Self { 
    Self {
      page_table: PageTable::new(),
      areas: Vec::new(),
    }
  }
  ///向 MemorySet 中添加一个虚拟地址区域（MapArea）
  fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
    //将当前逻辑段到物理内存的映射从传入的该逻辑段所属的地址空间的多级页表中加入
    map_area.map(&mut self.page_table); 
    if let Some(data) = data {
      //将数据拷贝到映射的内存区域中
      map_area.copy_data(&self.page_table, data);
    }
    self.areas.push(map_area);
  }
  ///在当前地址空间插入一个 Framed 方式映射到物理内存的逻辑段
  pub fn insert_frames_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
    self.push(MapArea::new(
      start_va,
      end_va,
      MapType::Framed,
      permission,
    ), None);
  }
  ///生成内核的地址空间
  pub fn new_kernel() -> Self{
    let mut memory_set = Self::new_bare();
    //跳板映射？
    memory_set.map_trampoline();
    //映射内核段
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
    println!("mapping .text section");
    memory_set.push(MapArea::new(
      (stext as usize).into(),
      (etext as usize).into(),
      MapType::Identical,
      MapPermission::R | MapPermission::X,
    ), None);
    println!("mapping .rodata section");
    memory_set.push(MapArea::new(
      (srodata as usize).into(),
      (erodata as usize).into(),
      MapType::Identical,
      MapPermission::R,
    ), None);
    println!("mapping .data section");
    memory_set.push(MapArea::new(
      (sdata as usize).into(),
      (edata as usize).into(),
      MapType::Identical,
      MapPermission::R | MapPermission::W,
    ), None);
    println!("mapping .bss section");
    memory_set.push(MapArea::new(
      (sbss as usize).into(),
      (ebss as usize).into(),
      MapType::Identical,
      MapPermission::R | MapPermission::W,
    ), None);
    println!("mapping physical memory");
    //确保了内核可以直接访问剩余物理内存，用于动态内存分配等功能
    memory_set.push(MapArea::new(
      (ekernel as usize).into(),
      MEMORY_END.into(),
      MapType::Identical,
      MapPermission::R | MapPermission::W,
    ), None);
    memory_set
  }

  ///分析应用的 ELF 文件格式的内容，解析出各数据段并生成对应的地址空间
  pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
    let mut memory_set = Self::new_bare();
    //映射跳板
    memory_set.map_trampoline();
    //用外部 crate xmas_elf 来解析传入的应用 ELF 数据并取出各个部分
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    //校验elf文件中的魔数
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    //Program Header 的数量
    let ph_count = elf_header.pt2.ph_count();
    //记录最大的虚拟页号
    let mut max_end_vpn = VirtPageNum(0);
    for i in 0..ph_count {
      let ph = elf.program_header(i).unwrap();
      //从 ELF 文件中提取所有 Load 类型的段，并将它们映射到内存中--包含了程序运行时需要加载到内存的数据，比如代码段、数据段等
      if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
        //设置段的虚拟地址范围
        let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
        let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize);
        //设置段的内存权限
        let mut map_perm = MapPermission::U; // 初始化权限为用户权限
        //获取段的标志，返回一个表示段权限的标志位（如是否可读、可写、可执行）
        let ph_flags = ph.flags();
        if ph_flags.is_read() { map_perm |= MapPermission::R; }
        if ph_flags.is_write() { map_perm |= MapPermission::W; }
        if ph_flags.is_execute() { map_perm |= MapPermission::X; }
        //创建逻辑段
        let map_area = MapArea::new(
          start_va,
          end_va,
          MapType::Framed,
          map_perm,
        );
        max_end_vpn = map_area.vpn_range.get_end();
        //将该逻辑段加入用户地址空间
        memory_set.push(
          map_area,
          Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
        ); //通过 elf.input[...] 提取该段的数据
      }
    }
    //映射用户栈
    let max_end_va: VirtAddr = max_end_vpn.into();
    let mut user_stack_bottom: usize = max_end_va.into();
    //增加一个保护页面:如何避免栈溢出？？？
    user_stack_bottom += PAGE_SIZE;
    let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
    //将用户栈映射到用户地址空间
    memory_set.push(MapArea::new(
      user_stack_bottom.into(),
      user_stack_top.into(),
      MapType::Framed,
      MapPermission::R | MapPermission::W | MapPermission::U,
    ), None);
    //将TrapContext映射到用户地址空间
    memory_set.push(MapArea::new(
      TRAP_CONTEXT.into(),
      TRAMPOLINE.into(),
      MapType::Framed,
      MapPermission::R | MapPermission::W,
    ), None);
    //返回应用地址空间、用户栈虚拟地址、应用入口地址
    (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
  }
}
