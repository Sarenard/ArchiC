; init of stack
let r0 0x00fffffc
copy sp r0
; main function
main:
    ; u32 x = Add(Int(0), Int(1))
    push r0
    push r1
    push 0;
    push 1;
    pop r0
    pop r1
    add r0 r0 r1
    push r0
    copy r0 0
    pop r1 ; the stack contains the value of x
    store [r0] r1
    pop r1
    pop r0
    ; u32 y = Add(Var("x"), Int(1))
    push r0
    push r1
    copy r0 0
    load r1 [r0]
    push r1
    push 1;
    pop r0
    pop r1
    add r0 r0 r1
    push r0
    copy r0 4
    pop r1 ; the stack contains the value of y
    store [r0] r1
    pop r1
    pop r0
    ; return Var("y")
    push r0
    push r1
    copy r0 4
    load r1 [r0]
    push r1
    pop r0
    skip 1 ifne r0 0
    jump green
    jump red
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
