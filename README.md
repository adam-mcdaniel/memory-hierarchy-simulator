# Programming Assignment \#1

***COSC 530 - Adam McDaniel***
---

## Overview

I created 2 other versions of the long trace: one copy where all the operations are writes, another copy where all the operations are reads. The read-only trace is very easy to pass because no behavior changes for the different strategies.
For the write-only trace, the simulator produces the correct trace for every strategy, but messes up the stats for the L2 and main memory references in some cases. For the original long trace input provided in the lab, the output differs at most 361 of the 866428 total lines of output.

Here is a table of the capabilities of my simulator using those traces:

|DC Strategy|L2 Strategy|Works For Read-Only Long-Trace|Works For Write-Only Long-Trace|Works For Provided Long-Trace|
|:---------:|:---------:|:----------------------------:|:-----------------------------:|:-----------------------------:|
|Write Through|Write Through|✅|✅ (**100% correct**)|✅ (**100% correct**)|
|Write Allocate|Write Through|✅|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|
|Write Through|Write Allocate|✅|✅ (**trace 100% correct**; main memory ref counter incorrect)|✅❔(12/866428 lines of output differ from reference implementation output; only 6 trace operation results differ)|
|Write Allocate|Write Allocate|✅|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|

Additionally, my program should work for all configurations of the small trace.

You can use my Makefile to compare my output against the reference. Pass the trace to test with using `trace=` on the command line.
```bash
$ make trace=long-trace.dat
Running mine...
Running reference...
0 lines differ in outputs
```

## Usage

To build my program, use the Rust package manager: `cargo`.

```bash
$ cd memory-hierarchy-simulator
$ # Compile the simulator in release mode
$ cargo build --release
$ # You can use this command to copy the compiled executable to the working directory, if you want.
$ cp target/release/memory-hierarchy .
```

You can run my program by passing the `trace.dat` file as a command line argument, or by passing it as standard input.

```bash
$ # Compile the simulator in release mode
$ cargo build --release
$ # Use STDIN to supply the trace like the reference executable
$ ./target/release/memory-hierarchy < trace.dat > output.txt
$ # Pass file as a command line argument
$ ./target/release/memory-hierarchy trace.dat > output.txt
```