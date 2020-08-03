# MIPS program to stress-test simulators - version 4; tests 0-7 working.
# if all is well, $s0 = 0; $s1 = 1; $s2 = 2; $s3 = 3; $s4 = 4; $s5 = 5;
# $s6 = 6; $s7 = 7
# 
# This program tests all "required" instructions in the SPIM simulator
# project.  Used in CPS 104, Spring 2003.
#
# Taken from http://robotics.duke.edu/courses/cps104/spring05/homework/torture.s

	.data				# Begin data segment
var1:	.word 0				# first memory variable
var2:	.word 0				# second memory variable
var3:	.word 0,0			# make sure var3 holds two words
					#   consecutively
	.text				# Begin code segment
	.globl main			# first instr must be global

###################### BEGIN  main #########################################

main:	subu $sp,$sp,16			# set up main's stack frame:
					#   need room for two args, two 
					#   two callee save registers
	sw $ra,12($sp)			# save main's return address
	sw $fp,8($sp)			# save main's caller's frame pointer
	addu $fp,$sp,12			# establish main's frame pointer

# test 0: zero register; result should be 0 in the zero register; 
# $s0 = 0 if all is well at end of test

	addi $0,$0,2			# in this case, 0+1=0 if all is well!
	addi $s0,$0,0			# MOVE $0 TO $s0 ; should be 0

# test 1: logical operations; 
# $s1 = 1 if all is well at end of test

	lui $t0,0x5555			# $t0 = 0x55555555
	ori $t0,$t0,0x5555
	lui $t1,0x0000			# $t1 = 0x00000000
	ori $t1,0x0000			
	lui $t2,0xCCCC			# $t2 = 0xCCCCCCCC
	ori $t2,$t2,0xCCCC
	or  $t0,$t2,$t0			# $t0 OR $t2  = 0xDDDDDDDD
	lui $t3,0x3333			# $t3 = 0x33333333
	ori $t3,$t3,0x3333
	and $t0,$t3,$t0			# $t0 AND $t3 = 0x11111111
	sll $t0,$t0,2			# $t0 = 0x44444444
	sra $t0,$t0,1			# $t0 = 0x22222222
	srl $t0,$t0,1			# $t0 = 0x11111111
	sll $t0,$t0,3			# $t0 = 0x88888888
	sra $t0,$t0,1			# $t0 = 0xC4444444
	lui $t7,0xC444			# $t7 = 0xC4444445
	ori $t7,0x4445
	xor $s1,$t0,$t7			#   if correct, set $s1 = 1

# test 2: load/store - simple lw/sw commands
# $s2 = 2 if all is well at end of test

# first store two numbers in memory
	lui $t0,0x7654			# $t0 = 0x76543210
	ori $t0,0x3210
	la $t1,var1			# $t1 = &var1 (pointer to var1)
	sw $t0,0($t1)			# var1 = $t0
	lui $t2,0x1234			# $t2 = 0x12345678
	ori $t2,0x5678
	la $t3,var2			# $t3 = &var2
	sw $t2,0($t3)			# var2 = $t3
# now retrieve the two numbers and add them
	lw $t4,0($t1)			# $t4 = var1
	lw $t5,0($t3)			# $t5 = var2
	addu $s2,$t4,$t5		# $s2 = $t4 + $t5
	lui $t7,0x8888			# $t7 = 0x88888886
	ori $t7,0x8886
	sub $s2,$s2, $t7		#   if correct, set $s2 = 2
		
# test 3: arithmetic
# $s3 = 3 if all is well at end of test

	add $t0,$0,$0			# clear $t0
	addi $t0,$t0,0xff		# $t0 = 255
	addi $t0,$t0,-240		# $t0 = 15
	addiu $t1,$0,15			# $t1 = 15 

	mult $t1,$t0			# $t2 = $t1*$t0 = 225
	mflo $t2			
	mfhi $t6			# $t6 should be 0
	add $s3,$t2,$t6			# $s3 = 225
	li $t1, 0x4FFFFFFF
	mult $s3, $t1
	mflo $t1			# $t1 = 0x4FFFFF1F
	mfhi $t2			# $t2 = 0x00000046
	add $s3,$t1,$t2			# $s3 = 0x4FFFFF65
	li $t0, 0x4FFFFF62
	subu $s3,$s3,$t0		# $s3 = 3
		
# test 4: jumping
# $s4 = 4 if all is well at end of test

j_top:	add $s4,$0,$0			# clear $s4
	j j_skip1
j_bad1:	addi $s4,$s4,17			# should not happen!
j_skip1:addi $s4,$s4,1			# $s4 = 1 now if all is well
	la $t0,j_skip2			# $t0 is pointer to j_skip2
	jr $t0				# jump to j_skip2
j_bad2:	addi $s4,$s4,23			# should not happen!
j_skip2:addi $s4,$s4,1			# $s4 = 2 now if all is well
	jal inc_s4 			# $s4 = 3 now if all is well on return
	jal inc_s4			# $s4 = 4 now if all is well on return

# test 5: branching
# $s5 = 5 if all is well at end of test

b_top:	add $s5,$0,$0			# clear $s5
	beq $s5,$0,b_skip1		# if $s5 = 0 goto b_skip1
b_bad1:	addi $s5,$s5,7			# should not happen!
b_skip1:addi $s5,$s5,1			# $s5 = 1 now if all is well
	addi $t0,$0,-1			# $t0 = -1
	bgez $t0,b_skip2		# if $t0 >= 0 goto b_skip2 (it isn't!)
b_good1:addi $s5,$s5,-10		# should do this!
b_skip2:addi $s5,$s5,11			# $s5 = 2 now if all is well
	bltz $t0,b_skip3		# if $t0 < 0 goto b_skip 3 (it is!)
b_bad2:	addi $s5,$s5,13			# should not happen!
b_skip3:addi $s5,$s5,1			# $s5 = 3 now if all is well
	add  $t1,$0,$0			# clear $t1
	blez $t1,b_skip4		# if $t1 <= 0 goto b_skip4 (it is!)
	addi $s5,$s5,17			# should not happen!
b_skip4:bgtz $t1,b_skip5		# if $t1 > 0 goto b_skip5 (it isn't!)
	addi $s5,$s5,-14		# should do this!
b_skip5:addi $s5,$s5,16			# $s5 = 5 now if all is well

# test 6: set instructions
# $s6 = 6 if all is well at end of test

	add $s6,$0,$0			# clear $s6
	addi $t0,$0,-10			# $t0 = -10 ( = 0xFFFFFFF6)
	addi $t1,$0,20			# $t1 = 20
	slt $t2,$t0,$t1			# $t2 = 1 if $t0 < $t1 (it is!)
	add $s6,$s6,$t2			# $s6 = 1 now if all is well
	sltu $t3,$t0,$t1		# $t3 = 1 if $t0 < $t1 unsigned (not!)
	add $s6,$s6,$t3			# $s6 = 1 still if all is well
	slti $t4,$t0,-5			# $t4 = 1 if $t0 < -5 (it is!)
	add $s6,$s6,$t4			# $s6 = 2 now if all is well
	add $s6, $s6, $s6
	sltiu $t5,$t0,-5		# $t5 = 1 if $t0 < -5 unsigned (yes!)
					# sltiu sign-extends but then compares
					# as unsigned; 0xFF..F6 is < 0xFF..FB
	add $s6,$s6,$t5			# $s6 = 5 still if all is well
	add $s6,$s6,1			# $s6 = 6 if all is well

# test 7: nonaligned load/store instructions
# $s7 = 7 if all is well at end of test

	lui $t0,0x1234			# $t0 = 0x12345678
	ori $t0,0x5678
	lui $t1,0x9abc			# $t1 = 0x9abcdef0
	ori $t1,0xdef0

	la $t2,var3			# $t2 = &var3 (pointer to var3)
	sw $t0,0($t2)			# var3[0] = $t0
	sw $t1,4($t2)			# var3[1] = $t1

	lb $t3,0($t2)			# $t3 = 0x00000012
	lb $t4,1($t2)			# $t4 = 0x00000034
	lb $t5,4($t2)			# $t5 = 0xffffff9a (sign extends)
	lbu $t6,5($t2)			# $t6 = 0x000000bc (doesn't sign extend)
	addu $t3, $t3, $t4
	addu $t5, $t5, $t6
	addu $s7, $t3, $t5		# $s3 = 9C
	sb $s7,1($t2)			# var3[0] = 0x129c5678
	lw $s7,0($t2)			# $s7 = 0x129c5678 if all is well
	lui $t0,0x129c			# $t0 = 0x129c5671
	ori $t0,0x5671
	sub $s7,$s7,$t0			# $s7 = 7 if all is well

# print s0 to s7
	li	$a1, 0
	add	$a2, $s0, $0
	jal	printout
	li	$a1, 1
	add	$a2, $s1, $0
	jal	printout
	li	$a1, 2
	add	$a2, $s2, $0
	jal	printout
	li	$a1, 3
	add	$a2, $s3, $0
	jal	printout
	li	$a1, 4
	add	$a2, $s4, $0
	jal	printout
	li	$a1, 5
	add	$a2, $s5, $0
	jal	printout
	li	$a1, 6
	add	$a2, $s6, $0
	jal	printout
	li	$a1, 7
	add	$a2, $s7, $0
	jal	printout

# test syscalls
	li	$v0, 4
	la	$a0, snum
	syscall
	li	$v0, 5
	syscall			# read a number
	add	$s1, $v0, $0	# store in $s1
	li	$v0, 4
	la	$a0, snumo
	syscall
	li	$v0, 1
	add	$a0, $s1, 0
	syscall			# print the same number
	li	$v0, 4
	la	$a0, nl
	syscall
	li	$v0, 4
	la	$a0, scorr
	syscall			# ask to input "yes/no"
 	la	$a0, sin
	li	$v0, 8
	li	$a1, 5
	syscall
	la	$a0, sin
	lb	$t0, 1($a0)
	li	$t1, 101
	bne	$t0, $t1, exit	# if the second char is "e"
	li	$v0, 4
	la	$a0, sgood
	syscall			# then print "good"
		
# prepare to exit program
exit:	
        lw $ra,12($sp)                  # restore return address
        lw $fp,8($sp)                   # restore frame pointer address
        addu $sp,$sp,16			# pop main's stack frame
	li $v0, 10			# setup for exit syscall
	syscall				# exec syscall

printout:
	li	$v0, 4
	la	$a0, ss
	syscall
	li	$v0, 1
	add	$a0, $a1, $0
	syscall
	li	$v0, 4
	la	$a0, sis
	syscall
	li	$v0, 1
	add	$a0, $a2, $0
	syscall
	li	$v0, 4
	la	$a0, nl
	syscall
	jr	$ra

##################### END MAIN ###########################################

# inc_s4: Leaf procedure, called from jump test to verify call/return
# behavior.  Increments $s4 once.

inc_s4: addi $s4,$s4,1
	jr $ra

	.data
nl:	.asciiz "\n"
ss:	.asciiz "s"
sis:	.asciiz "="
sin:	.word 0, 0, 0, 0
scorr:	.asciiz "Are they correct? (yes/no)"
sgood:	.asciiz "Good!\n"
snum:	.asciiz "Input a number: "
snumo:	.asciiz "The number is: "

