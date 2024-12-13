#保存Trap上下文的 __alltraps 的实现

#定义了一个宏 SAVE_GP，用于将通用寄存器 x\n（例如 x1, x2, x3 等）的值保存到栈中。保存的地址是 sp + n*8，即栈指针偏移 n*8 字节的位置

.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
#将后续的代码或数据按照 4 字节对齐
__alltraps:
    #csrrw 是一个原子操作，交换 sp（栈指针）和 sscratch 寄存器的值--sp保存用户栈指针,sscratch 会保存内核栈指针。
    csrrw sp, sscratch, sp
    #将x1保存到栈顶的偏移量为 1*8 的位置
    sd x1, 1*8(sp)
    #先跳过x2和x4
    sd x3, 3*8(sp)
    #使用.rept 指令（重复）来依次保存 x5 到 x31（共 27 个寄存器）的值到栈中
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    #csrr 指令用于读取 sstatus 和 sepc 寄存器的值，并保存到 t0 和 t1 中
    csrr t0, sstatus
    csrr t1, sepc
    #并将这两个寄存器中的值保存到内核栈中
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    #sscratch 的值是进入 Trap 之前的 sp 的值，指向用户栈
    csrr t2, sscratch
    sd t2, 2*8(sp)
    #将内核地址空间的token载入到t0寄存器中
    ld t0, 34*8(sp)
    #将trap_handler入口点的虚拟地址放入t1
    ld t1, 36*8(sp)
    #将sp修改为应用内核栈顶地址
    ld sp, 35*8(sp)
    #切换到内核空间
    csrw satp, t0
    #刷新快表
    sfence.vma
    #跳转到trap_handler
    jr t1


__restore:
    #a0:用户地址空间的TrapContext的位置（在所有应用的地址空间都相同）
    #a1：用户地址空间的token
    #切换到用户地址空间
    csrw satp, a1
    #清空快表
    sfence.vma
    #将传入的 Trap 上下文位置保存在 sscratch 寄存器中
    csrw sscratch, a0
    #将sp修改为用户空间的trap上下文的位置--基于该位置恢复寄存器
    mv sp, a0
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    #恢复普通寄存器除了x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n,n+1
    .endr
    #使sp回到用户栈
    ld sp, 2*8(sp)
    #回到U特权级继续运行应用程序控制流
    sret

