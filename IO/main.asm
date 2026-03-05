main:
    jump red

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
