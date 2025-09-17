    .section .text
    .global _start

_start:
    MOV r0, #0x02000000    @ RAM base
    MOV r1, #1             @ counter = 1
    MOV r2, #10            @ max = 10

loop:
    subs    r2, r2, #1
    bne     loop

done:
    b       done

