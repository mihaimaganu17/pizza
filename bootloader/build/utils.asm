[bits 32]
section .text
    global _add_2_numbers
_add_2_numbers:
    push ebp
    mov ebp, esp
    mov eax, [ebp + 8]
    mov edx, [ebp + 12]
    add eax, edx
    pop ebp
    ret

