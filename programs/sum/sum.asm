# DESCRIPTION:
#	This program reads up to 10 of numbers (or until the user
#	enters the value 9999) from the standard input, and then 
#	computes and prints their sum.
#
# ARGUMENTS:
#	None
#
# INPUT:
# 	The numbers to be summed.
#
# OUTPUT:
#	A "before" line with the 10 numbers in the order they were
#	entered, and an "Sum=" and then the sum of all the numbers

#-------------------------------
# CONSTANTS

MAX_SIZE= 10			# number of array elements
PRINT_STRING = 4		# arg for syscall to tell it to write
PRINT_INT = 1
FRAMESIZE = 8

#-------------------------------
# DATA AREAS

	.data

	.align	2		# word data must be on word boundaries
array:	
	.space	4*MAX_SIZE	# Reserve space for array to hold data
				# the array is up to MAX_SIZE words (4byte
				# each). Note the array isn't initialized
size:
	.word	0		# Actual number of values in the array

	.align	0		# string data doesn't have to be aligned
space:	
	.asciiz	" "
lf:	
	.asciiz	"\n"
before:	
	.asciiz	"Values entered: "
sum_msg:	
	.asciiz	"Sum= "
prompt:	
	.asciiz	"Enter Number: "
reprompt:	
	.asciiz	"Number not positive.\nRe-Enter Number: "

# CODE AREAS

	.text			# this is program code
	.align	2		# instructions must be on word boundaries

	.globl	main		# main is a global label

# Name:         main
#
# Description:  EXECUTION BEGINS HERE
# Arguments:    none
# Returns:      none
# Destroys:     t0,t1,t2,t3
main:
				# allocate space for the stack frame
	addi	$sp,$sp,-FRAMESIZE	
	sw	$ra,4($sp)	# store the ra on the stack
	sw	$s0,0($sp)	# store the s0 on the stack

	la	$a0,array	# Pass the address of the array in a0
	li	$a1,MAX_SIZE	# and its max size in a1
	jal	readarray

# The number of elements in the array is returned in v0
# store it into memory then print the array

	la	$t0,size
	sw	$v0,0($t0)		# store num. of val. entered

	li	$v0,PRINT_STRING	# print a "Values:"
	la	$a0,before
	syscall

	jal	parray

# Sum up the elements in the array

	li 	$t0,0			# t0 loop counter
	la      $t1,size                # t1 is addr in mem of the size val
	lw      $t1,0($t1)		# t1 is number of elements read

	la	$s0,array		# s0 is pointer into array
	li 	$t2,0			# t2 is the running total
	
sum_loop:				
	beq	$t0,$t1,sum_loop_end
	lw	$t3,0($s0)		# read the value
	add	$t2,$t2,$t3		# add it to the sum

	addi	$t0,$t0,1		# increment counter
	addi	$s0,$s0,4		# increment array pointer
	j	sum_loop
sum_loop_end:
	
	li	$v0,PRINT_STRING		# print "Sum=	"
	la	$a0,sum_msg
	syscall

	li	$v0,PRINT_INT	# print the sum
	move	$a0,$t2
	syscall

	li	$v0,PRINT_STRING
	la	$a0,lf
	syscall			# print a newline

	lw	$ra,4($sp)	# restore the registers
	lw	$s0,0($sp)	
	addi	$sp,$sp,FRAMESIZE	
	jr	$ra		# return from main and exit program

# Name:         parray
#
# Description:  prints the "size" number of integers from the 
#		array "array"
# Arguments:    none
# Returns:      none
# Destroys:     t0,t1
parray:
	la	$a0,array	# a0 is the location of the array
	la	$t0,size
	lw	$a1, 0($t0)	# a1 is the number of elements entered

	li	$t0,0		# i=0;
	move	$t1,$a0		# t1 is pointer to array
pa_loop:
	beq	$t0,$a1,done	# done if i==n

	lw	$a0,0($t1)	# get a[i]
	li	$v0,PRINT_INT
	syscall			# print one int

	li	$v0,PRINT_STRING
	la	$a0,space
	syscall			# print a space

	addi    $t1,$t1,4       # update pointer
	addi	$t0,$t0,1	# and count
	j	pa_loop
done:
	li	$v0,PRINT_STRING
	la	$a0,lf
	syscall			# print a newline

	jr	$ra

# Name:         readnumber
#
# Description:  reads in a positive integers
# Arguments:    none
# Returns:      return the number read in (in v0)
# Destroys:     none
readnumber:
	li	$v0,PRINT_STRING
	la	$a0,prompt
	syscall			# print string
readloop:
        li      $v0,5
        syscall

	slti	$t0, $v0, 0
	beq	$t0, $zero, goodnumber
	li	$v0,PRINT_STRING
	la	$a0,reprompt
	syscall			# print string
	j	readloop

goodnumber:
	jr $ra

# Name:         readarray
#
# Description:  reads in an array of integers, can read up to MAX_SIZE
#		elements or until the user enters the sentinal 9999 
# Arguments:    $a0 the address of the array
#		$a1 the max number of elements that can be in the array
# Returns:      return the number of values read in
# Destroys:     t0,t1,t9
readarray:
	addi	$sp, $sp, -4
	sw	$ra, 0($sp)	# save ra on stack
	addi	$sp, $sp, -4
	sw	$s0, 0($sp)	# save s0 on stack
	addi	$sp, $sp, -4
	sw	$s1, 0($sp)	# save s1 on stack

	li	$s0,0		# s0 will hold the num. of ele. entered
	move	$s1,$a0		# s1 is pointer to array
ra_loop:
	beq	$s0,$a1,ra_done	# done if num_ele == max allowed

	jal	readnumber

	li	$t9,9999	# sentinal to leave loop
	beq	$v0,$t9,ra_done

	sw	$v0,0($s1)

	addi	$s1,$s1,4	# update pointer
	addi	$s0,$s0,1	# and increment the count
	j	ra_loop
ra_done:
	li	$v0,PRINT_STRING
	la	$a0,lf
	syscall			# print a message

	move	$v0,$s0		# return the number of values read in


	lw      $s1, 0($sp)     # restore the s1 from the stack
	addi    $sp, $sp, 4
	lw      $s0, 0($sp)     # restore the s0 from the stack
	addi    $sp, $sp, 4
	lw	$ra, 0($sp)	# restore the ra from the stack
	addi    $sp, $sp, 4
	jr	$ra		# return execution

