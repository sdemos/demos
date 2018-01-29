; long_mode_init.asm
;
; this file is called once boot.asm gets us into long mode and enables 64-bit
; instructions. from here we will actually call our kernel.

global long_mode_start          ; declare this file provides long_mode_start

extern rust_main                ; define rust_main as a function somewhere else

section .text                   ; executable code goes in the .text section

bits 64                         ; specify that our instructions are 64 bits

; long_mode_start gets called once boot.asm gets us into long mode and enables
; 64-bit instructions.
long_mode_start:
  ; load 0 into all data segment registers
  mov ax, 0
  mov ss, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  ; call the kernel
  mov rax, rust_main
  or rax, 0xffffff0000000000
  call rust_main

  ; print 'OKAY' to the screen
  mov rax,  0x2f592f412f4b2f4f
  mov qword [0xb8000], rax
  ; halt for now
  hlt
