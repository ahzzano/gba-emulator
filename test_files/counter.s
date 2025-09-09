    MOV r0, #0x02000000   
    MOV r1, #1            
    MOV r2, #10           

loop:
    STR r1, [r0], #4      
    ADD r1, r1, #1        
    SUBS r2, r2, #1       
    BNE loop              

