# A series of addition operations

.text
.globl main

main:
	addi $t0, $zero, 1      # $t0 = 1
	addi $t1, $zero, 10     # $t1 = 10
	add $t2, $t0, $t1      # $t2 = $t0 + $t1

