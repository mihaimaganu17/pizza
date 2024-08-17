    section .text
[bits 16]
; Protected mode 16-bit
_pm_16bit:
    ; Clear the PE flag from CR0 to enable real-mode
    mov eax, cr0
    ; PE flag is bit 0
    and eax, -1
    ; Make sure that paging is disabled (bit 31 of cr0 has to be 0)
    and eax, 0x7ffffffe
    mov cr0, eax

    ; Move 0x00 into the CR3 register to flush the TLB (translation-lookaside buffer)
    xor eax, eax
    mov cr3, eax

.call_int:
    ; Get the interrupt number
    ; This is the first argument of the function calling `_real_mode_int`. Because we previously
    ; pushad'ed, we have 8 registers and a return address on the stack before the actual argument
    movzx ebx, byte [esp + (4 * 8 + 4)]
    ; Get the interrupt offset in the IVT, which contains the segment and the offset where the
    ; interrupt routine is. Each IVT is 4 bytes entry, so we have to scale our interrupt code to
    ; reflect that
    shl ebx, 2
    ; Get the register state contained in the second argument pushed to the stack by the calling
    ; Rust function
    mov eax, dword [esp + (4 * 8 + 4 + 4)]
    ; Computer the relative address where we have to return after the interrupt
    mov ebp, (.return_from_int - image_base)

;--------------------------------------------------------------------------------------------------
    ; Construct the interrupt stack frame that the interrupt call expects
.inject_stack_frame:
    ; Setting up the stack frame that we control, which will be popped off by the interrupt handler
    ; that we call in order to return execution after the interrupt is finished.
    ; This will switch controll to the address of `.return_from_int`.
    ; This is because the interrupt handler will perform another iret, which expects to pop and
    ; restore the CS and EIP registers and the EFLAGS to resume execution of the interrupted
    ; procedure that called this interrupt
    ; Push flags
    pushfw
    ; Push code selector
    push cs
    ; Push address for ip to return to
    push bp

    ; Following, we are setting up the stack frame that will be popped of by iret and which will
    ; essentially transfer the call to the interrupt handler
    pushfw
    ; At this point, the we want to access the code handling the interrupt. As such
    ; the CPU expects the Segment and the Offset from the IVT entry in the IVT table along with.
    push word [bx+2]
    push word [bx+0]

    ; Now, we want to set all the registers to the state which was sent by the caller. We leave eax
    ; at the end, because it has the pointer to the entire state structure
    mov ebx, dword [eax + reg_sel_state.ebx]
    mov ecx, dword [eax + reg_sel_state.ecx]
    mov edx, dword [eax + reg_sel_state.edx]
    mov esi, dword [eax + reg_sel_state.esi]
    mov edi, dword [eax + reg_sel_state.edi]
    mov ebp, dword [eax + reg_sel_state.ebp]
    mov eax, dword [eax + reg_sel_state.eax]

    ; Perform a far return to the interrupt entry point, simulating a software interrupt
    ; instruction. So essentially this iret transfers control to the BIOS, in that specific
    ; interrupt handler. Interrupt handler which will need to return and will use the next available
    ; parameters on the stack to fill in (ip, cs and eflags), parameters which we control and we
    ; injected at `.inject_stack_frame` label.
    iretw

.return_from_int:
    cli
    hlt

[bits 32]
    global _real_mode_int
; Call a real mode interrupt from a protected mode CPU
_real_mode_int:
    ; Save all the 8 register state of the caller
    pushad
    ; Load the real-mode GDT
    lgdt [real_mode_gdtr]

    ; Load all segments with the data selector (3rd entry in the RealMode GDT)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Jump into protected mode 16bit and load CS with the second selector in the real mdoe GDT
    jmp 0x0008:(_pm_16bit - image_base)

    global _add_2_numbers
_add_2_numbers:
    push ebp
    mov ebp, esp
    mov eax, [esp+8]
    mov edx, [esp+12]
    add eax,edx
    pop ebp
    ret

; Define a state structure that holds the value of all 32-bit registers, except the ESP and all the
; 16-bit selectors, except CS. This structure will be used for both input and output register values
; whenever we are calling a BIOS Interrupt handler
struc reg_sel_state
    .eax: resd 1
    .ebx: resd 1
    .ecx: resd 1
    .edx: resd 1
    .esi: resd 1
    .edi: resd 1
    .ebp: resd 1
    .ds: resw 1
    .es: resw 1
    .ss: resw 1
    .gs: resw 1
    .fs: resw 1
endstruc

align 8
real_mode_gdt:
    ; First entry in the GDT must always be 0
    dq 0
    ; Code segment in 16-bit protected mode, populated with the image base
    dq 0x00009a000000ffff | (image_base << 16)
    ; DAata segment in 16-bit protected mode
    db 0xff, 0xff, 0x00, 0x00, 0x00, 0x92, 0x00, 0x00

real_mode_gdtr:
    ; 16-bit size of the GDT
    dw (real_mode_gdtr - real_mode_gdt) - 1
    ; 32-bit address of the GDT
    dd real_mode_gdt

