#保存Trap上下文的 __alltraps 的实现

#定义了一个宏 SAVE_GP，用于将通用寄存器 x\n（例如 x1, x2, x3 等）的值保存到栈中。保存的地址是 sp + n*8，即栈指针偏移 n*8 字节的位置

.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
#将后续的代码或数据按照 4 字节对齐
__alltraps:
    csrrw sp, sscratch, sp
    #csrrw 是一个原子操作，交换 sp（栈指针）和 sscratch 寄存器的值--sp保存用户栈指针,sscratch 会保存内核栈指针。
    addi sp, sp, -34*8
    #为TrapContext在内核栈上分配空间
    sd x1, 1*8(sp)
    #将x1保存到栈顶的偏移量为 1*8 的位置
    sd x3, 3*8(sp)
    #先跳过x2和x4
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    #使用.rept 指令（重复）来依次保存 x5 到 x31（共 27 个寄存器）的值到栈中
    csrr t0, sstatus
    csrr t1, sepc
    #csrr 指令用于读取 sstatus 和 sepc 寄存器的值，并保存到 t0 和 t1 中
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    #并将这两个寄存器中的值保存到内核栈中
    csrr t2, sscratch
    sd t2, 2*8(sp)
    #sscratch 的值是进入 Trap 之前的 sp 的值，指向用户栈
    mv a0, sp
    # a0 指向内核栈的栈指针（保存的 Trap 上下文的地址）
    call trap_handler


__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0
    #sp指向内核栈栈顶
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    #恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n,n+1
    .endr
    #在核心栈释放Trap上下文
    addi sp, sp, 34*8
    #使sp重新指向用户栈栈顶
    csrrw sp, sscratch, sp
    #回到U特权级继续运行应用程序控制流
    sret


  