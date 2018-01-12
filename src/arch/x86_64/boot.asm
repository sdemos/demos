global start                    ; export the start label

section .text                   ; executable code goes in the .text section

bits 32                         ; we are still in protected mode, so we need our
                                ; instructions to be 32 bits until we switch to
                                ; long mode.

; the start label is the entrypoint of our kernel
start:
  ; print 'OK' to the screen
  mov dword [0xb8000], 0x2f4b2f4f
  ; halt for now
  hlt
