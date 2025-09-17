.section .text
.global _start

_start:
    MOV r0, #0x02000000    @ RAM base
    MOV r1, #1
    MOV r2, #1
    MOV r3, #20             @ run for 20 iterations

loop:
    adds r1, r2
    adds r2, r1
    subs r3, #1
    bne loop

    str r1, [r5, #10]
    str r2, [r5, #11]

    ldr r1, [r5]
    ldr r2, [r5]

    ldr r1, [r5, #10]
    ldr r2, [r5, #11]

done:
    b done
