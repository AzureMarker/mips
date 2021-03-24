# MIPS assembler, linker, simulator, and debugger

## Goals
- Assembler
  - [X] Parse assembly code
  - [X] Assemble code into object files
  - [ ] Support all directives and instructions
  - [X] Handle global (extern) references
  - [X] Support data sections like sdata
- Linker
  - [ ] Support transforming one independent object module (only exports
        main) into an executable file (R2K format).
  - [ ] Support linking (at least) two object modules together such that a
        global exported by one and used by another is wired up correctly.
- Simulator
  - [X] Create a simulated MIPS CPU
  - [X] Parse R instructions
  - [ ] Execute all R instructions
  - [X] Parse I instructions
  - [ ] Execute all I instructions
  - [X] Parse J instructions
  - [X] Execute all J instructions
- Debugger
  - [X] Drive the simulator
  - [X] Inspect the registers
  - [ ] Inspect the surrounding code/instructions
  - [X] Breakpoints

## Test Programs
The `programs` directory contains some test programs which have been assembled
and linked with a MIPS toolchain I refer to as "R2K" (the assembler is "rasm",
the linker is "rlink", the simulator is "rsim", and the debugger is "rbug").
This was the tool I used in college when learning MIPS, but access to it was
very restricted (only available on university servers, execute permissions only)
and I have not found it anywhere online. I am using it to provide assembled
binaries while my own assembler and linker are still in development.
