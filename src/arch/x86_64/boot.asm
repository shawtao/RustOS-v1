global start
extern long_mode_start

section .text
bits 32
start:
    mov esp, stack_top
    mov edi, ebx

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables
    call enable_paging

    lgdt [Gdt64.pointer]

    jmp Gdt64.code64_dsc:long_mode_start
    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    ; Check if CPUID is supported by attempting to filp the ID bit (bit 21)
    ; in the FLAGS register. If we can filp it, CPUID is avaliable.

    ; Copy FLAGS in to EAX via stack (we can only load FLAGS by pushfd)
    pushfd
    pop eax

    ; backup in ecx
    mov ecx, eax
    ; filp ID bit
    xor eax, 1 << 21

    push eax
    popfd

    pushfd
    pop eax

    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000     ; use eax to implicit argument for cpuid
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode        ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode
    ret
.no_long_mode:
    mov al, "2"
    jmp error

set_up_page_tables:
    ; map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    mov eax, p4_table
    or  eax, 0b11
    mov [p4_table + 511 * 8], eax

    mov ecx, 0 ;use ecx as counter

.map_p2_table:
    ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000
    mul ecx
    or  eax, 0b10000011
    mov [p2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ret

enable_paging:
    ; load p4_table to cr3
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or  eax, 1 << 5
    mov cr4, eax 

    ;set the long mode bit in the ERER MSR (model specofoc register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or  eax, 1 << 31
    mov cr0, eax

    ret




; parameter: error code (in ascii) in al
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt


section .bss
; reserved memory for paging
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
; reserved memory for stack
stack_bottom:
    resb 4096 * 4
stack_top:

section .rodata
; 64bit GDT table
Gdt64:
.null_dsc: 
    dq 0 ; the first entry must be zero
.code64_dsc: equ $ - Gdt64
    dq 0x0020980000000000  ;code segment
.data64_dsc: 
    dq 0x0000920000000000  ;data segment
.pointer:
    dw $ - Gdt64 - 1
    dq Gdt64

