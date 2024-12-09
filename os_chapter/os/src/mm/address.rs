//!地址相关的数据结构抽象和类型定义

use super::PageTableEntry;
use crate::config::{PAGE_SIZE_BITS,PAGE_SIZE};
use core::fmt::{self,Debug,Formmater};

///物理地址
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
///虚拟地址
const VA_WIDTH_SV39: usize = 39;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;


///物理地址的Rust类型声明
#[derive(Copy,Clone,Ord,PartialOrd,Eq,PartialEq)]
pub struct PhysAddr(pub usize); //元组式结构体--对usize的一种简单封装

///虚拟地址的Rust类型声明
#[derive(Copy,Clone,Ord,PartialOrd,Eq,PartialEq)]
pub struct VirtAddr(pub usize);

///物理页号的Rust类型声明
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

///虚拟页号的Rust类型声明
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

///Debugging
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

///实现从usize到四种类型的转换From特性
impl From<usize> for PhysAddr {
  fn from(v: usize) -> Self {
    Self(v & ((1 << PA_WIDTH_SV39)-1))//先将1向左移动，再减一，当作掩码，保证位数合法
  }
}
impl From<usize> for PhysPageNum {
  fn from(v: usize) -> Self {
    Self(v & ((1 << PPN_WIDTH_SV39)-1))
  }
}
impl From<usize> for VirtAddr {
  fn from(v: usize) -> Self {
    Self(v & ((1 << VA_WIDTH_SV39)-1))
  }
}
impl From<usize> for VirtPageNum {
  fn from(v: usize) -> Self {
    Self(v & ((1 << VPN_WIDTH_SV39)-1))
  }
}
///实现从四种类型到usize的转换
impl From<PhysAddr> for usize {
  fn from(v: PhysAddr) -> Self {
    v.0 //v.0 表示访问元组结构体 v 的第一个字段的值
  }
}
impl From<PhysPageNum> for usize {
  fn from(v: PhysPageNum) -> Self {
    //定v.0是否位于高半部分的地址空间--即该虚拟地址是否为内核的虚拟地址
    if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
      v.0 | (!((1 << VA_WIDTH_SV39) - 1)) //高39位都置位1 --对高位进行填充
    }else {
      v.0
    }
  }
}
impl From<VirtPageNum> for usize {
  fn from(v: VirtPageNum) -> Self {
    v.0
  }
}

impl PhysAddr {
  ///获取页面偏移量函数
  pub fn page_offset(&self) -> usize {
    self.0 & (PAGE_SIZE - 1) //按位与得到低(PAGE_SIZE - 1)位
  }
  /// 物理地址向下取整
  pub fn floor(&self) -> PhysPageNum {
    PhysPageNum(self.0 / PAGE_SIZE)
  }
  /// 物理地址向上取整
  pub fn ceil(&self) -> PhysPageNum {
    (self.0 + PAGE_SIZE - 1) / PAGE_SIZE
  }
  ///判断物理地址的偏移量是否为0--是否对齐
  pub fn aligned(&self) -> bool {
    self.page_offset() == 0
  }
  ///将物理地址转换为某种类型 T 的不可变引用
  pub fn get_ref<T>(&self) -> &'static T {
    unsafe { (self.0 as *const T).as_ref().unwrap()}
  }
  ///将物理地址转换为某种类型 T 的可变引用
  pub fn get_mut<T>(&self) -> &'static mut T {
    unsafe { (self.0 as *mut T).as_mut().unwrap() }
  }
}

///实现物理地址和物理页号之间的转换
impl From<PhysAddr> for PhysPageNum {
  fn from(v: PhysAddr) -> Self {
    assert_eq!(v.page_offset(), 0); //确保偏移部分为 0
    v.floor()
  }
}
impl From<PhysPageNum> for PhysAddr {
  fn from(v: PhysPageNum) -> Self {
    Self {
      v.0 << PAGE_SIZE_BITS
    }
  }
}

impl VirtAddr {
  ///虚拟地址向下取整
  pub fn floor(&self) -> VirtPageNum {
    VirtPageNum(self.0 / PAGE_SIZE)
  }
  ///虚拟地址向上取整
  pub fn ceil(&self) -> VirtPageNum {
    VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
  }
  ///取虚拟地址的页面偏移量
  pub fn page_offset(&self) -> usize {
    self.0 & (PAGE_SIZE - 1)
  }
  ///是否对齐
  pub fn aligned(&self) -> bool {
    self.page_offset() == 0
  }
}

///实现虚拟地址和虚拟页面的转换
impl From<VirtAddr> for VirtPageNum {
  fn from(v: VirtAddr) -> Self {
    assert_eq!(v.page_offset(), 0);
    v.floor()
  }
}
impl From<VirtPageNum> for VirtAddr {
  fn from(v: VirtPageNum) -> Self {
    Self(v.0 << PAGE_SIZE_BITS)
  }
}

impl PhysPageNum {
  ///返回一个页表项定长数组的可变引用
  pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
    let pa: PhysAddr = self.clone().into();
    unsafe {
      core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
    }
  }
  ///返回一个字节数组的可变引用
  pub fn get_bytes_array(&self) -> &'static mut [u8] {
    let pa: PhysAddr = self.clone().into();
    unsafe {
      core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
    }
  }
  ///将当前物理页号 (PhysPageNum) 对应的物理内存页，直接转换为某种自定义数据类型 T 的可变引用
  pub fn get_mut<T>(&self) -> &'static mut T {
    let pa: PhysAddr = self.clone().into();
    unsafe {
      (pa.0 as *mut T).as_mut().unwrap() //as_mut将裸指针转换为可变引用
    }
  }
}

impl VirtPageNum {
  ///取出虚拟页号的三级页索引
  pub fn indexes(&self) -> [usize; 3] {
    let mut vpn = self.0;
    let mut idx = [0usize, 3];
    for i in (0..3).rev() { //逆序遍历
      idx[i] = vpn & 511; //取低九位
      vpn >>= 9; //将虚拟地址右移9位，并删除低9位
    }
    idx
  }
}

///实现该特性的类型可以通过 step 方法递增
pub trait StepByOne {
  fn step(&mut self);
}
impl StepByOne for VirtPageNum {
  fn step(&mut self) {
    self.0 += 1;
  }
}
impl StepByOne for PhysPageNum {
  fn step(&mut self) {
    self.0 += 1;
  }
}

///泛型范围数据结构
#[derive(Copy, Clone)] 
pub struct SimpleRange<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  l: T,
  r: T,
}

impl<T> SimpleRange<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  pub fn new(start: T, end: T) -> Self {
    assert!(start <= end, "start {:?} > end {:?}!", start, end);
    Self { l: start, r: end}
  }
  pub fn get_start(&self) -> T {
    self.l
  }
  pub fn get_end(&self) -> T {
    self.r
  }
}

impl<T> IntoIterator for SimpleRange<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  type Item = T;
  type IntoIter = SimpleRangeIterator<T>;
  fn into_iter(self) -> Self::IntoIter {
    SimpleRangeIterator::new(self.l, self.r)
  }
}

///范围迭代器
pub struct SimpleRangeIterator<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  current: T,
  end: T,
}
impl<T> SimpleRangeIterator<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  pub fn new(l: T, r: T) -> Self {
    Self { current: l, end: r}
  } 
}
impl<T> Iterator for SimpleRangeIterator<T>
where
  T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
  type Item = T;
  fn next(&mut self) -> Option<Self::Item> {
    if self.current == self.end {
      None
    } else {
      let t = self.current;
      self.current.step();
      Some(t)
    }
  }
}
pub type VPNRange = SimpleRange<VirtPageNum>;