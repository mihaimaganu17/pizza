[bits 32]
; Define a state structure that holds the value of all 32-bit registers, except the ESP and all the
; 16-bit selectors, except CS. This structure will be used for both input and output register values
; whenever we are calling a BIOS Interrupt handler.
; WARNING: The order of the 32-bit register is important, because it is the order in which pushad
; pushes them to the stack
struc reg_sel_state
    .eax: resd 1
    .ecx: resd 1
    .edx: resd 1
    .ebx: resd 1
    .esp: resd 1
    .ebp: resd 1
    .esi: resd 1
    .edi: resd 1
    .eflags: resd 1
    .ds: resw 1
    .es: resw 1
    .ss: resw 1
    .gs: resw 1
    .fs: resw 1
endstruc

    section .text
    global _real_mode_int
; Call a real mode interrupt from a protected mode CPU
_real_mode_int:
	cld
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
    jmp 0x0008:(.pm_16bit - image_base)

[bits 16]
; Protected mode 16-bit
.pm_16bit:
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

    ; Zero the segments (not necessarily needed here, as a far return should zero them out :-?
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov fs, ax
    mov ss, ax

    ; Perform a far return to real-mode
    ; Push flags
    pushfd
    ; Push code segment
    push dword (image_base >> 4)
    ; Push instruction pointer
    push dword (.call_int - image_base)
    iretd

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
;.inject_stack_frame:
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
    mov ecx, dword [eax + reg_sel_state.ecx]
    mov edx, dword [eax + reg_sel_state.edx]
    mov ebx, dword [eax + reg_sel_state.ebx]
    mov ebp, dword [eax + reg_sel_state.ebp]
    mov esi, dword [eax + reg_sel_state.esi]
    mov edi, dword [eax + reg_sel_state.edi]
    mov eax, dword [eax + reg_sel_state.eax]

    ; Perform a far return to the interrupt entry point, simulating a software interrupt
    ; instruction. So essentially this iret transfers control to the BIOS, in that specific
    ; interrupt handler. Interrupt handler which will need to return and will use the next available
    ; parameters on the stack to fill in (ip, cs and eflags), parameters which we control and we
    ; injected at `.inject_stack_frame` label.
    iretw

.return_from_int:
    ; We push all the state to the stack in order to save the results we got from the interrupt
    push eax
    push ecx
    push edx
    push ebx
    push ebp
    push esi
    push edi
    pushfd
    push ds
    push es
    push ss
    push gs
    push fs

    ; Get a pointer to the structure passed by as a second argument from the Rust call. This is
    ; located above everything we pushed above int this `return_from_int` label + everything we
    ; pushed in `read_mode_int` + the first argument + the return address
    mov eax, dword [esp + (4*8 + 5*2 + 4*8 + 4 + 4)]

    ; Now we update the register and selector state of the passed in register state structure
    ; mentioned above by popping everything we pushed above.
    pop word [eax + reg_sel_state.fs]
    pop word [eax + reg_sel_state.gs]
    pop word [eax + reg_sel_state.ss]
    pop word [eax + reg_sel_state.es]
    pop word [eax + reg_sel_state.ds]
    pop dword [eax + reg_sel_state.eflags]
    pop dword [eax + reg_sel_state.edi]
    pop dword [eax + reg_sel_state.esi]
    pop dword [eax + reg_sel_state.ebp]
    pop dword [eax + reg_sel_state.ebx]
    pop dword [eax + reg_sel_state.edx]
    pop dword [eax + reg_sel_state.ecx]
    pop dword [eax + reg_sel_state.eax]

    ; Now we have the result of the interrupt and we need to go back in 32-bit protected mode

    ; Load the GDT data selector from the program that called us. Image base is 32-bit, translated
    ; into real mode this means -> high 4 bytes for the selector and low 4 bytes for the offset
    mov ax, (image_base >> 4)
    mov ds, ax

    ; Set the PE flag in cr0 to enable protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    ; Offset of the GDT
    mov eax, (pm_gdtr - image_base)
    lgdt [eax]

    ; Set all the segments to the data segment selector from the new GDT, which is the 3rd entry
    ; and each entry is 8 bytes (first is 0, second is code selector)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov gs, ax
    mov fs, ax

    ; Set another interrupt frame that will make us jump back to protected mode
    ; Push flags
    pushfd
    ; Push CS selector
    push dword 0x0008
    ; Push EIP
    push dword ret_to_rust

    ; Return control back to the caller. This will perform a far return and get us back in protected
    ; mode
    iretd

[bits 32]
    global _pxe_call
; Call a real mode interrupt from a protected mode CPU
_pxe_call:
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
    jmp 0x0008:(.pxe_call_16bit - image_base)

[bits 16]
; Protected mode 16-bit
.pxe_call_16bit:
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

    ; Zero the segments (not necessarily needed here, as a far return should zero them out :-?
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov fs, ax
    mov ss, ax

    ; Perform a far return to real-mode
    ; Push flags
    pushfd
    ; Push code segment
    push dword (image_base >> 4)
    ; Push instruction pointer
    push dword (.call_pxe - image_base)
    iretd

.call_pxe:
    ; pub fn pxecall(pxe_code_seg: u16, pxe_offset: u16, data_seg: u16, param_offset: u16,
    ;                   pxe_call: u16)
    movzx eax, word [esp + (4 * 8 + 4)]; arg1, pxe undi code segment
    movzx ebx, word [esp + (4 * 8 + 8)]; arg2, pxe undi code segment offset
    movzx ecx, word [esp + (4 * 8 + 12)]; arg3, pxe undi data segment
    movzx edx, word [esp + (4 * 8 + 16)]; arg4, parameter offset for the pxe function to be called
    movzx esi, word [esp + (4 * 8 + 20)]; arg5, pxe code for the function to be called

    ; Push parameters for the PXE call
    push cx,
    push dx,
    push si,

    ; Setting up the far return that will make pxe give execution back after it executes the call
    mov ebp, (.ret_from_pxe_call - image_base)
    push cs
    push bp

    ; Setting up the stack frame that will be used by the BIOS real mode to execute the PXE call
    ; with the arguments we got from the caller in eax(code segment) and ebx(segment offset)
    ; Push flags
    pushfw
    ; Push CS
    push ax,
    ; Push offset / IP
    push bx,

    ; Perform a far return which will jump to real mode and execute the PXE call, after which
    ; it will return execution to `.ret_from_pxe_call`
    iretw

.ret_from_pxe_call:
    ; Clear interrupts
    cli
    ; Clear up the stack from the last 3 arguments we passed to pxe
    add sp, 3 * 2

    ; Load the GDT data selector from the program that called us. Image base is 32-bit, translated
    ; into real mode this means -> high 4 bytes for the selector and low 4 bytes for the offset
    mov ax, (image_base >> 4)
    mov ds, ax

    ; Set the PE flag in cr0 to enable protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; Offset of the GDT
    mov eax, (pm_gdtr - image_base)
    lgdt [eax]

    ; Set all the segments to the data segment selector from the new GDT, which is the 3rd entry
    ; and each entry is 8 bytes (first is 0, second is code selector)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov gs, ax
    mov fs, ax

    ; Set another interrupt frame that will make us jump back to protected mode
    ; Push flags
    pushfd
    ; Push CS selector
    push dword 0x0008
    ; Push EIP
    push dword ret_to_rust

    ; Return control back to the caller. This will perform a far return and get us back in protected
    ; mode
    iretd

[bits 32]
ret_to_rust:
    ; Pop the first register state that we saved upon entering `_real_mode_int`, which was the
    ; callers register state
    popad
    ret

[bits 32]
    global _enter_ia32e
_enter_ia32e:
    ; Get the parameters passed by the function
    ; dword [esp + 0x14] ; value to be put in the cr3
    ; quad [esp + 0x0c] ; stack
    ; quad [esp + 0x04] ; entry point
    mov esi, dword [esp + 0x1c] ; Pointer to the PML4 page table

    ; Disable paging (Set PG to 0)
    mov eax, cr0
    or eax, 0x7fffffff
    mov cr0, eax
    ; Enable physical address extension. This allows addresses with more than 32 bits to be
    ; represented.
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    ; Load the CR3 with the physical base address of the Level 4 page map table (PML4)
    mov cr3, esi
    ; Enable IA-32e mode, by setting the IA32_EFER.LME = 1, which is the 8th bit in MSR C000_0080H
    mov ecx, 0xc0000080
    ; Reads MSR from the adress in ECX into registers EDX:EAX
    rdmsr
    xor eax, eax
    xor edx, edx
    ; Enable IA-32e mode operation and not execute (bit 8 and 11)
    mov eax, 0b1001 << 8
    ; Writes the contents of registers EDX:EAX into a 64-bit MSR address specified in the ECX
    ; register
    wrmsr
    ; Enable paging by setting CR0.PG = 1
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ; Load the 64-bit IA-32 GDT
    lgdt [ia32e_gdtr]

    ; Jump to IA-32e mode (also known as long mode)
    jmp 0x0008:ia32e_mode



    section .data
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

pm_gdt:
    ; First entry is always Null
    dq 0
    db 0xff,0xff,0x00,0x00,0x00,0x9a,0xcf,0x00
    db 0xff,0xff,0x00,0x00,0x00,0x92,0xcf,0x00

; GDTR descriptor loaded using the LGDT assembly, which contains a size and a pointer to the GDT
pm_gdtr:
    ; Size of the GDT. In the case GDT is maxed out (2*16) bytes, we cannot fit that value in
    ; 16-bits, so we have to substract 1
    dw (pm_gdtr - pm_gdt) - 1
    ; 4-bytes for the offset in 32-bit mode
    dd pm_gdt

ia32e_gdt:
    ; First entry is always Null
    dq 0
    ; In 64-bit mode, Base and Limit values are ignored, thus we set them to 0
    db 0x00,0x00,0x00,0x00,0x00,0x9a,0x20,0x00 ; Long-mode code flag, limit is 0
    db 0x00,0x00,0x00,0x00,0x00,0x92,0x00,0x00

; GDTR descriptor loaded using the LGDT assembly, which contains a size and a pointer to the GDT
ia32e_gdtr:
    ; Size of the GDT. In the case GDT is maxed out (2*16) bytes, we cannot fit that value in
    ; 16-bits, so we have to substract 1
    dw (ia32e_gdtr - ia32e_gdt) - 1
    ; 4-bytes for the offset in 32-bit mode
    dd ia32e_gdt
    dd 0

[bits 64]
ia32e_mode:
    ; Set all the segments to the data segment selector from the new GDT, which is the 3rd entry
    ; and each entry is 8 bytes (first is 0, second is code selector)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov gs, ax
    mov fs, ax

    mov rdi, qword [rsp + 0x04] ; Entry point for the function that will execute from Rust in ia32e
    mov rbp, qword [rsp + 0x0c] ; Stack
    ; In the MSFT x64 calling convention, stack space is allocated even for parameters passed in
    ; registers, like the first 4 parameters passed in RCX, RDX, R8 and R9. This means that we need
    ; to consider an additional 32 bytes (8 * 4) of space on the stack. In addition, we need to
    ; add a fake return address that the `iretq` will return to
    sub rbp, 0x28

    mov rcx, qword [rsp+0x14]

    ; Push the iret frame -> Intel Manual Vol 3a, 6.14.4
    ; SS
    push qword 0x0010
    ; RSP
    push qword rbp
    ; Push flags
    pushfq
    ; Push code selector
    push qword 0x0008
    ; push the instruction pointer
    push qword rdi
    ; Execute the interrupt
    iretq
