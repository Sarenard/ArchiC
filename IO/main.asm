; init of stack
let r0 0x00fffffc
copy sp r0
; main function
main:
    ; u32 a = Int(4)
    push r0
    push r1
    push 4;
    copy r0 0
    pop r1 ; the stack contains the value of a
    store [r0] r1
    pop r1
    pop r0
    ; u32 b = Int(7)
    push r0
    push r1
    push 7;
    copy r0 4
    pop r1 ; the stack contains the value of b
    store [r0] r1
    pop r1
    pop r0
    ; u32 tot = Int(0)
    push r0
    push r1
    push 0;
    copy r0 8
    pop r1 ; the stack contains the value of tot
    store [r0] r1
    pop r1
    pop r0
    ; while (Var("a"))
    push r0
    push r1
    while_0_check:
    copy r0 0
    load r1 [r0]
    push r1
    pop r0
    skip 1 ifeq r0 0
    jump while_0_true
    jump while_0_end
    while_0_true:
    ; if (And(Var("a"), Int(1)))
    push r0
    push r1
    copy r0 0
    load r1 [r0]
    push r1
    push 1;
    pop r1
    pop r0
    and r0 r0 r1
    push r0
    pop r0
    skip 1 ifeq r0 0
    jump if_1_true
    jump if_1_false
    if_1_true:
    ; tot = Add(Var("tot"), Var("b"))
    push r0
    push r1
    copy r0 8
    load r1 [r0]
    push r1
    copy r0 4
    load r1 [r0]
    push r1
    pop r1
    pop r0
    add r0 r0 r1
    push r0
    pop r1 ; the stack contains the value of tot
    copy r0 8
    store [r0] r1
    pop r1
    pop r0
    jump if_1_end
    if_1_false:
    jump if_1_end
    if_1_end:
    pop r1
    pop r0
    ; a = RShift(Var("a"), Int(1))
    push r0
    push r1
    copy r0 0
    load r1 [r0]
    push r1
    push 1;
    pop r1
    pop r0
    lsr r0 r0 r1
    push r0
    pop r1 ; the stack contains the value of a
    copy r0 0
    store [r0] r1
    pop r1
    pop r0
    ; b = LShift(Var("b"), Int(1))
    push r0
    push r1
    copy r0 4
    load r1 [r0]
    push r1
    push 1;
    pop r1
    pop r0
    lsl r0 r0 r1
    push r0
    pop r1 ; the stack contains the value of b
    copy r0 4
    store [r0] r1
    pop r1
    pop r0
    jump while_0_check
    while_0_end:
    pop r1
    pop r0
    ; u32 ok = Sub(Var("tot"), Int(56))
    push r0
    push r1
    copy r0 8
    load r1 [r0]
    push r1
    push 56;
    pop r1
    pop r0
    sub r0 r0 r1
    push r0
    copy r0 12
    pop r1 ; the stack contains the value of ok
    store [r0] r1
    pop r1
    pop r0
    ; return Var("ok")
    push r0
    push r1
    copy r0 12
    load r1 [r0]
    push r1
    pop r0
    skip 1 ifne r0 0
    jump green ; true
    jump red ; false
    pop r1
    pop r0

green:
    copy r3 0
    let r0 0x01000000
    let r1 0x0000FF00
gloop:
    xor r2 r2 r2
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    store [r0 + r2] r1
    add r3 r3 1
    jump gloop

red:
    copy r3 0
    let r0 0x01000000
    let r1 0x000000FF
rloop:
    xor r2 r2 r2
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    store [r0 + r2] r1
    add r3 r3 1
    jump rloop
