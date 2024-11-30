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
}
impl TrapContext {
  ///使栈指针寄存器指向x2寄存器
  pub fn set_sp(&mut self, sp: usize) {
    self.x[2] = sp;
  }
  ///初始化app context
  pub fn app_init_context(entry: usize, sp: usize) -> Self{
    let mut sstatus = sstatus::read();
    sstatus.set_spp(SPP::User);//将 sstatus 寄存器的 SPP 字段设置为 User
    let mut cx = Self{
      x: [0; 32],
      sstatus,
      sepc: entry,
    };
    cx.set_sp(sp); //应用程序的栈指针
    cx
  }
}