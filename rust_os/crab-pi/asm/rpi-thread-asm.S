/*
 * implement these.  ordered in difficulty.
 */
#include "rpi-asm.h"

@ return the current stack pointer.
.global rpi_get_sp
rpi_get_sp:
    mov r0, sp
    bx lr

@ void rpi_cswitch(uint32_t **old_sp_save, const uint32_t *new_sp);
@   - called to context switch from currently running thread
@     (so must save registers) to next thread (so must load
@     registers).  eg:
@        rpi_cswitch(&cur_th->saved_sp, next_th->saved_sp);
.global rpi_cswitch
rpi_cswitch:
    @ Store all callee + <LR> on to current threads stack
    push {{r4-r11, lr}}

    @ Save current stack pointer to saved_sp
    str sp, [r0]

    @ Move next thread's sp to stack pointer
    mov sp, r1

    @ Pop next thread's registers
    pop {{r4-r11, lr}}

    @ return
    bx lr

@ use this to setup each thread for the first time.
@ setup the stack so that when cswitch runs it will:
@	- load address of <rpi_init_trampoline> into LR
@	- <code> into r4,
@	- <arg> into r5
@
.global rpi_init_trampoline
rpi_init_trampoline:
    @ register popped by rpi_cswitch, setup argument
    mov r0, r5

    @ setup call
    blx r4

    @ end of the call, go back to rpi_exit
    b rpi_exit