#
# simple startup routine
#
# scss id: @(#)r2k_startup.asm	1.1        11/21/07
#
# linked in before any user code
#

#
# we use EXIT2 instead of EXIT
#
SYS_EXIT2 = 17

#
# want to invoke the user's entry point
#
	.globl	main

#
# before the entry point, insert a break instruction
# to ensure that we can't just fall into this code
# (as it will be at the end of the text section)
#
	break	0x2eb4

#
# our entry point (must match that in <common/exec.h>
#
	.globl	__r2k__entry__

__r2k__entry__:

#
# at entry, the stack contains this:
#
#	sp ->	argc
#		argv ptr
#		env ptr
#		argv[0]
#		 ...
#		argv[argc-1]
#		0
#		env[0]
#		 ...
#		env[n-1]
#		0

#
# call the user's main() routine
#
	lw	$a0, 0($sp)	# main( argc, argvptr, envptr )
	lw	$a1, 4($sp)
	lw	$a2, 8($sp)
	jal	main
	nop			# in case we're doing delayed branches

#
# exit, using whatever was in $v0 as the status
#
	add	$a0, $v0, $zero
	li	$v0, SYS_EXIT2
	syscall
