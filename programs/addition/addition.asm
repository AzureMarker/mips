# A series of addition operations

.text
.globl main

TEST2 = TEST1

main:
	li $t0, 1		# $t0 = 1
	li $t1, 10		# $t1 = 10
	add $t2, $t0, $t1	# $t2 = $t0 + $t1

