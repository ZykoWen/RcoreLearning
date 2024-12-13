//实现trap上下文的类型
use riscv::register::sstatus::{self,Sstatus,SPP};

#[repr(C)] //结构体内存布局将与 C 语言中的布局相同。这意味着该结构体或枚举可以与 C 语言代码进行交互
pub struct TrapContext {
  ///通用寄存器 x0~x31
  pub x: [usize; 32], 
  ///CSR sstatus--在 Trap 控制流最后 sret 的时候还用到了它们
  pub sstatus: Sstatus, 
  ///CSR spec
  pub sepc: usize,
  //内核地址空间的 token（内核页表的起始物理地址）
  pub kernel_satp: usize,
  //当前应用在内核地址空间中的内核栈栈顶的虚拟地址
  pub kernel_sp: usize,
  // 内核中 trap handler 入口点的虚拟地址
  pub trap_handler: usize,
}
impl TrapContext {
  ///使栈指针寄存器指向x2寄存器
  pub fn set_sp(&mut self, sp: usize) {
    self.x[2] = sp;
  }
  ///初始化app context
  pub fn app_init_context(entry: usize, sp: usize, kernel_satp: usize, kernel_sp: usize, trap_handler: usize) -> Self {
    let mut sstatus = sstatus::read();
    sstatus.set_spp(SPP::User);
    let mut cx = Self {
      x: [0;32],
      sstatus,
      sepc: entry,
      kernel_satp,
      kernel_sp,
      trap_handler,
    };
    cx.set_sp(sp);
    cx
  }
}