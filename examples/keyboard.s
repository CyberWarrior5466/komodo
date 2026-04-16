mov r0, #'\n'
swi #0

loop:
    swi #1
    swi #0
    b loop
