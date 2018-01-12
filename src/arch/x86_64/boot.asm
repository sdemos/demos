; boot.asm
;
; the first of our code to be executed. the asm in this file performs a bunch of
; checks for features we need, sets up an initial stack, enters long mode, and
; then calls our kernel. the actual kernel call happens in 64-bit code, which is
; in the `long_mode_init.asm` file.
;
; the code in this file has several possible error conditions, most of them
; relating to feature checks failing. currently, the possible errors and their
; related codes are:
; 0 - check_multiboot failed, bootloader doesn't have multiboot support
;     specifically the magic number was not in eax
; 1 - check_cpuid failed, cpu doesn't have the cpuid instruction
; 2 - check_long_mode failed, the cpu doesn't support long mode

global start                    ; export the start label

extern long_mode_start          ; 64-bit code for when we are in long mode

; the .rodata section is for read-only data. we are using it to build a gdt
; (global descriptor table) that we have to pass to the cpu for various
; x86-is-backwards-compatible-to-the-dawn-of-time reasons.
section .rodata
; a gdt starts with a zero entry and contains an arbitrary number of segment
; entries following it. our 64-bit gdt only needs one segment, a code segment.
gdt64:
  dq 0                          ; zero entry
.code: equ $ - gdt64            ; calculate location of code segment
  ; code segment. they have the descriptor type, present, executable, and 64-bit
  ; bits set, which are 44, 47, 43, and 53 respectively.
  dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.pointer:
  ; to load the gdt, we use the lgdt cpu instruction, which requires a special
  ; pointer structure that includes the length and location of the gdt.
  dw $ - gdt64 - 1
  dq gdt64

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

  ; setup paging and enter long mode
  call setup_page_tables
  call enable_paging

  ; load the 64-bit gdt
  lgdt [gdt64.pointer]

  ; calls into our 64-bit code, and also reloads the code selector
  jmp gdt64.code:long_mode_start

  ; we hopefully won't return from that call...
  hlt

; page functions
;
; these functions are needed to setup the page tables so we can enter long mode

; setup_page_tables sets up all the page tables that we need to start our kernel
; we set up a p4 table pointing at a p3 table pointing at a p2 table that is
; mapped with 2MiB pages
setup_page_tables:
  ; map first p4 entry to p3 table
  mov eax, p3_table
  or eax, 0b11                  ; present + writable
  mov [p4_table], eax

  ; map first p3 entry to p2 table
  mov eax, p2_table
  or eax, 0b11                  ; present + writable
  mov [p3_table], eax

  ; map each p2 entry to a huge 2MiB page
  mov ecx, 0                    ; counter variable

.map_p2_table:
  ; map ecx-th p2 entry to a huge page starting at address 2MiB*ecx
  mov eax, 0x200000             ; 2MiB
  mul ecx                       ; start address of ecx-th page
  or eax, 0b10000011            ; present + writable + huge
  mov [p2_table + ecx * 8], eax ; map ecx-th entry

  inc ecx                       ; increase counter
  cmp ecx, 512                  ; if counter == 512, we are done mapping
  jne .map_p2_table             ; else map the next entry

  ; done!
  ret

; enable_paging does the magic to tell the cpu to turn on paging and enter long
; mode. we write the address of our p4 table to cr3, enable pae, set the long
; mode bit in the efer register, and then actually enable paging.
enable_paging:
  ; load p4 to cr3 register (the cpu uses cr3 to access the p4 table)
  mov eax, p4_table
  mov cr3, eax

  ; enable pae-flag in cr4 (Physical Address Extension)
  mov eax, cr4
  or eax, 1 << 5
  mov cr4, eax

  ; set the long mode bit in the EFER MSR (Model Specific Register)
  mov ecx, 0xC0000080
  rdmsr
  or eax, 1 << 8
  wrmsr

  ; enable paging in the cr0 register
  mov eax, cr0
  or eax, 1 << 31
  mov cr0, eax

  ret

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

section .bss
; setup page tables
align 4096
p4_table:
  resb 4096
p3_table:
  resb 4096
p2_table:
  resb 4096

; reserve some bytes for the stack
stack_bottom:
  resb 64
stack_top:
