; Address where the BIOS loads the current bootloader in memory and start executing it
[org 0x7c00]
; Execution starts in 16-bit Real Mode
[bits 16]

entry:
    ; Stop serving IRQs (interrupt requests)
    cli
    ; Make sure we go from lowest to highest address incrementing
    cld

    ; Enable the A20 to avoid wraparound to 0 of addresses bigger than 1 MiB
    in al, 0x92
    or al, 2
    out 0x92, al

    ; Make sure ds is set to 0 such that we can use the selector with a known value
    xor ax, ax
    mov ds, ax

    ; Load a 32-bit GDT
    lgdt [ds:pm_gdt]

    ; Set the CR0.PE to enable protected mode
    xor eax, eax
    mov eax, 1
    mov cr0, eax

    ; Now that we are in protected mode, we can use a far jump to A:B, where A is a selector
    ; offset inside the GDT (in this case the second value) and B is the offset we got to from
    ; that segment base
    jmp 0x0008:pm_entry

[bits 32]
pm_entry:
    ; At this points CS segment register is loaded, because of the the far jump we did previously.
    ; And we want to load all the other segment registers as well. And we will load them with
    ; the selector offset of the GDTs 3rd entry.
    mov ax, 0x10
    mov es, ax
    mov ds, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; Loop for ever
    cli
    hlt
;--------------------------------------------------------------------------------------------------

; Global Descriptor Table for protected mode. Each entry is 8-bytes in size and referred to as a
; segment descriptor. The segment descriptor structure is a bit weird:
; https://wiki.osdev.org/Global_Descriptor_Table
; Entries are 8-bytes in size so we align
align 8
pm_gdt:
    ; First entry is always Null
    dq 0
    ; Next-state (kernel) code segment mapping, which begins at
    ; Base 0x00100000, which is right past the last bit of the BIOSes memory map
    ; and we have a maximum addressable unit of 256 kilobytes * 4 Kib block size from the flags bit
    ; 3 set (0x3fffff), which is the maximum value we can expresse for the limit.
    ; We want the flags to define a 32-bit protected mode segment (bit 2 set) and set the Limit
    ; unit in 1 byte block granularity (bit 3 set).
    ; For the access byte we want: 0b10011010
    db 0xff,0xff,0x00,0x00,0x10,0x9a,0xcf,0x00
    ; Next-state (kernel) data segment mapping
    ; Base = 0x00500000
    ; Limit (
    ; Flags = 0b1100
    ; Access byte is: 0b10010010 (0x92)
    db 0xff,0xff,0x00,0x00,0x50,0x92,0xcf,0x00

; GDTR descriptor loaded using the LGDT assembly, which contains a size and a pointer to the GDT
pm_gdtr:
    ; Size of the GDT. In the case GDT is maxed out (2*16) bytes, we cannot fit that value in
    ; 16-bits, so we have to substract 1
    dw (pm_gdtr - pm_gdt) - 1
    ; 4-bytes for the offset in 32-bit mode
    dd pm_gdt

; Fill the rest of the bootloader with 0
times 510-($-$$) db 0
; Tell the BIOS that this is a valid sector to be used as a bootloader by setting the last 2 bytes
; of the 512 bytes to 0x55 and 0xAA
dw 0x55,0xAA
