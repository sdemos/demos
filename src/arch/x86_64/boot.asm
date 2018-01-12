; boot.asm
;
; the first of our code to be executed. the asm in this file performs a bunch of
; checks for features we need, sets up an initial stack, enters long mode, and
; then calls our kernel.
;
; the code in this file has several possible error conditions, most of them
; relating to feature checks failing. currently, the possible errors and their
; related codes are:
; 0 - check_multiboot failed, bootloader doesn't have multiboot support
;     specifically the magic number was not in eax
; 1 - check_cpuid failed, cpu doesn't have the cpuid instruction
; 2 - check_long_mode failed, the cpu doesn't support long mode

global start                    ; export the start label

section .text                   ; executable code goes in the .text section

bits 32                         ; we are still in protected mode, so we need our
                                ; instructions to be 32 bits until we switch to
                                ; long mode.

; the start label is the entrypoint of our kernel
start:
  ; setup the stack
  mov esp, stack_top

  ; call all the check functions
  call check_multiboot
  call check_cpuid
  call check_long_mode

  ; print 'OK' to the screen
  mov dword [0xb8000], 0x2f4b2f4f
  ; halt for now
  hlt

; check functions
;
; these functions perform various cpu checks. they are necessary to make sure
; we are actually booting up on a cpu that has the features we need.

; check_multiboot makes sure the bootloader that is loading us is actually
; multiboot compliant. the multiboot spec requires a magic number to be loaded
; into the eax register before it loads a kernel, so we check to make sure it's
; there
check_multiboot:
  cmp eax, 0x36d76289
  jne .no_multiboot
  ret
.no_multiboot:
  mov al, "0"
  jmp error

; check_cpuid jumps through a bunch of hoops to detect if the cpu we are running
; on has the cpuid assembly instruction. the routine is mostly copied from the
; osdev wiki. the idea is that if we can flip the ID bit (bit 21) in the FLAGS
; register, than the cpuid instruction is supported. unfortunately, we can't
; flip flags in the FLAGS register directly, which is why this is so tedious.
check_cpuid:
  ; copy flags into eax via stack
  pushfd
  pop eax

  ; copy to ecx as well for comparing later on
  mov ecx, eax

  ; flip the id bit
  xor eax, 1 << 21

  ; copy eax to flags via the stack
  push eax
  popfd

  ; copy flags back to eax (with the flipped bit if cpuid is supported)
  pushfd
  pop eax

  ; restore flags from the old version stored in ecx
  push ecx
  popfd

  ; copmare eax and ecx. if they are equal then the bit wasn't flipped and cpuid
  ; isn't supported on this cpu.
  cmp eax, ecx
  je .no_cpuid
  ret
.no_cpuid:
  mov al, "1"
  jmp error

; check_long_mode uses cpuid to detect whether long mode is available on this
; cpu. cpuid is a bit of a pain to use. the first call gets the highest
; supported parameter value, because we need to make sure that we can even ask
; it this question in the first place. once we make sure the processor is new
; enough to support asking about long mode, we ask.
check_long_mode:
  ; test if extended processor information is available
  mov eax, 0x80000000           ; implicit argument for cpuid
  cpuid                         ; get highest supported argument
  cmp eax, 0x80000001           ; it needs to be at least 0x80000001
  jb .no_long_mode              ; if it's less, the cpu is too old

  ; use extended info to test if long mode is available
  mov eax, 0x80000001           ; argument for extended processor info
  cpuid                         ; returns various feature bits in ecx and edx
  test edx, 1 << 29             ; test if the LM-bit is set in edx
  jz .no_long_mode              ; if it's not set, there is no long mode
  ret
.no_long_mode:
  mov al, "2"
  jmp error

; a simple error routine
; prints 'ERR: X' where X is the given error code. then it halts the cpu.
; parameter: error code (in ascii) in al
error:
  mov dword [0xb8000], 0x4f524f45
  mov dword [0xb8004], 0x4f3a4f52
  mov dword [0xb8008], 0x4f204f20
  mov byte  [0xb800a], al
  hlt

; reserve some bytes for the stack
section .bss
stack_bottom:
  resb 64
stack_top:
