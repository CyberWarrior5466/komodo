EXIT = 2
PRINT_STR = 3

.section .data
hello:
    .asciz "Hello World!\n"

.section .text
    ldr r0, =hello
    swi #PRINT_STR

    swi #EXIT
