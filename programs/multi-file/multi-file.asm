# Imports a function from lib.asm and executes it

PRINT_INT = 1

    .globl main
    .globl lib_function
    .text

main:
    addi $sp, $sp, -4
    sw $ra, 0($sp)

    # Get the number
    jal lib_function
    nop # In case delay slots are enabled

    # Print the number
    move $a0, $v0
    li $v0, PRINT_INT
    syscall

    lw $ra, 0($sp)
    jr $ra
