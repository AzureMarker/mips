	.text
	.globl main
main:
	li $t0, 1
	jal func
	li $t0, 2

func:
	add $s1, $t0, $zero
