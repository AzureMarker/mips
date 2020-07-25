# MIPS assembler, simulator, and debugger
Or at least that's what I plan on doing :)

## Goals
- Assembler
  - [ ] Parse assembly code
  - [ ] Assemble code into object files
  - [ ] Link object files into final executable
- Linker
  - [ ] TODO: fill out goals
- Simulator
  - [ ] Create a simulated MIPS CPU
  - [ ] Parse R instructions
  - [ ] Execute R instructions
  - [ ] Parse I instructions
  - [ ] Execute I instructions
  - [ ] Parse J instructions
  - [ ] Execute J instructions
- Debugger
  - [ ] Drive the simulator
  - [ ] Inspect the registers
  - [ ] Inspect the surrounding code/instructions
  - [ ] Breakpoints

## Test Programs
The `programs` directory contains some test programs which have been assembled
and linked with a tool called "rsim". This was the tool I used in college when
learning MIPS, but access to it was very restricted (only available on
university servers, execute permissions only) and I have not found it anywhere
online. I am using it to provide assembled binaries while my own assembler and
linker are still in development.
