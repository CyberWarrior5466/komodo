loop:
add r1, r1, #1

mov r0, r1
swi #4

mov r0, #'\n'
swi #0

cmp r1, #0xff0
bllt loop
