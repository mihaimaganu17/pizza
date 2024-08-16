    section .text
[bits 16]
real_mode:
    ; Have all data selectors point to the 2nd entry of the real-mode gdt
    mov ax, 0x10
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

[bits 32]
    global _switch_to_real_mode
_switch_to_real_mode:
    ; Clear interrupts
    cli
    ; Save registers
    pusha
    ; Load the real-mode gdt
    xor ax, ax
    mov ds, ax
    lgdt [ds:real_mode_gdtr]

    ; Clear the PE flag from CR0 to enable real-mode
    mov eax, cr0
    ; PE flag is bit 0
    and eax, -1
    ; Make sure that paging is disabled
    and eax, 0x7ffffffe
    mov cr0, eax

    ; Move 0x00 into the CR3 register to flush the TLB (translation-lookaside buffer)
    xor eax, eax
    mov cr3, eax

    ; Jump into real mode
    jmp 0x0008:real_mode


    global _add_2_numbers
_add_2_numbers:
    push ebp
    mov ebp, esp
    mov eax, [esp+8]
    mov edx, [esp+12]
    add eax,edx
    pop ebp
    ret

align 8
real_mode_gdt:
    ; First entry in the GDT must always be 0
    dq 0
    ; Code segment in 16-bit protected mode
    db 0xff, 0xff, 0x00, 0x00, 0x00, 0x9a, 0x00, 0x00
    ; DAata segment in 16-bit protected mode
    db 0xff, 0xff, 0x00, 0x00, 0x00, 0x93, 0x00, 0x00

real_mode_gdtr:
    ; 16-bit size of the GDT
    dw (real_mode_gdtr - real_mode_gdt) - 1
    ; 32-bit address of the GDT
    dd real_mode_gdt

