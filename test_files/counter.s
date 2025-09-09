    .section .text
    .global _start

_start:
    MOV r0, #0x02000000    @ RAM base
    MOV r1, #1             @ counter = 1
    MOV r2, #10            @ max = 10

loop:
    STR r1, [r0], #4       @ store counter
    ADD r1, r1, #1         @ counter++

    SUB r3, r2, r1         @ r3 = max - counter
    BGT loop               @ <-- oops, you don’t have conditionals yet

    @ So instead we fake it:
    CMP r1, r2             @ if you have CMP+skip, works
    BNE loop

done:
    B done

