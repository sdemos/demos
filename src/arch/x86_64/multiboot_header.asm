; multiboot_header.asm
;
; this file implements the header described in the multiboot standard,
; signaling to any compatible bootloader that they can use multiboot to start
; us

section .multiboot_header

header_start:
  dd 0xe85250d6                 ; magic number for multiboot 2
  dd 0                          ; architecture 0 (protected mode i386)
  dd header_end - header_start  ; header length

  ; checksum
  dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

  ; optional mutliboot tags go here

  ; required end tag
  dw 0                          ; type
  dw 0                          ; flags
  dd 8                          ; size
header_end:
