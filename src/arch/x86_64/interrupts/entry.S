// One entry per vector gives the Rust handler a uniform vector/error prefix.
// CPU error-code vectors include modern #CP, AMD #VC and #SX. Reserved vectors
// receive a synthetic zero, and an unexpected vector follows the fatal path.
// Sources: Intel SDM vol. 3, Interrupt and Exception Handling; AMD APM vol. 2.
.section .text.ferrum_interrupts,"ax",@progbits
.altmacro

.macro FERRUM_ENTRY vector
.p2align 4
ferrum_interrupt_\vector:
    .if (\vector == 8) || (\vector == 10) || (\vector == 11) || (\vector == 12) || (\vector == 13) || (\vector == 14) || (\vector == 17) || (\vector == 21) || (\vector == 29) || (\vector == 30)
        // The CPU already pushed the error code below its return frame.
    .else
        push 0
    .endif
    push \vector
    jmp ferrum_interrupt_common
.endm

.set ferrum_vector, 0
.rept 256
    FERRUM_ENTRY %ferrum_vector
    .set ferrum_vector, ferrum_vector + 1
.endr

ferrum_interrupt_common:
    // Interrupt gates clear IF. Clearing DF also establishes the System V
    // direction-flag invariant before calling compiler-generated Rust code.
    cld
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // The interrupted register frame remains above our temporary save area.
    // RBX is callee-saved by the Rust System V ABI and keeps that exact RSP.
    mov rbx, rsp
    mov rdi, rsp
    sub rsp, 527
    and rsp, -16
    // FXSAVE64 owns 512 bytes aligned to 16. The call's own stack grows below
    // this area. The kernel target disables the red zone and AVX by default.
    fxsave64 [rsp]
    call {dispatcher}
    fxrstor64 [rsp]
    mov rsp, rbx

    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
    // Discard vector + real/synthetic error. IRETQ restores RIP, CS, RFLAGS,
    // RSP and SS, including the interrupted direction and enable flags.
    add rsp, 16
    iretq

.section .rodata.ferrum_interrupt_entries,"a",@progbits
.balign 4
.global ferrum_interrupt_entries
ferrum_interrupt_entries:
.macro FERRUM_ADDRESS vector
    // A signed image-relative displacement permits PIE loading without an
    // absolute address relocation inside a read-only segment.
    .long ferrum_interrupt_\vector - ferrum_interrupt_entries
.endm
.set ferrum_vector, 0
.rept 256
    FERRUM_ADDRESS %ferrum_vector
    .set ferrum_vector, ferrum_vector + 1
.endr
.noaltmacro
