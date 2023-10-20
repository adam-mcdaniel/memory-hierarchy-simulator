# Programming Assignment \#1

***COSC 530 - Adam McDaniel***
---

## Overview

I created 2 other versions of the long trace: one copy where all the operations are writes, another copy where all the operations are reads. The read-only trace is very easy to pass because no behavior changes for the different strategies. To create the read-only and write-only traces, I simply replaced all `W`s with `R`s, and `R`s with `W`s.
For the write-only trace, the simulator produces the correct trace for every strategy, but messes up *just the stat counters* for the L2 and main memory references in some cases. For the original long trace input provided in the lab, however, the output differs at most 361 of the 866428 total lines of output.

Here is a table of the capabilities of my simulator using those traces:

|DC Strategy|L2 Strategy|Works For Long-Trace With All Reads|Works For Long-Trace With All Writes|Works For Provided Long-Trace|
|:---------:|:---------:|:----------------------------:|:-----------------------------:|:-----------------------------:|
|Write Through|Write Through|✅ (**100% correct**)|✅ (**100% correct**)|✅ (**100% correct**)|
|Write Back/Allocate|Write Through|✅ (**100% correct**)|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|
|Write Through|Write Back/Allocate|✅ (**100% correct**)|✅❔ (**trace 100% correct**; main memory ref counter incorrect)|✅❔(12/866428 lines of output differ from reference implementation output; only 6 trace operation results differ)|
|Write Back/Allocate|Write Back/Allocate|✅ (**100% correct**)|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|

Additionally, my program should work for all configurations of the small trace.

You can use my Makefile to compare my output against the reference. Pass the trace to test with using `trace=` on the command line.
```bash
$ make trace=long-trace.dat
Running mine...
Running reference...
0 lines differ in outputs
```

## Usage

To build my program, use the 🚀🚀🚀🚀Rust 🚀🚀package 🚀🚀🚀manager: 🚀🚀🚀`cargo` (blazingly fast🚀🚀🚀🚀🚀🚀🚀).

```bash
$ cd memory-hierarchy-simulator
$ # Compile the simulator in release mode
$ cargo build --release
$ # You can use this command to copy the compiled executable to the working directory, if you want.
$ cp target/release/memory-hierarchy .
```

You can run my program by passing the trace file as a command line argument, or by passing it as standard input.

```bash
$ # Compile the simulator in release mode
$ cargo build --release
$ # Use STDIN to supply the trace like the reference executable
$ ./target/release/memory-hierarchy < long-trace.dat > output.txt
$ # Pass file as a command line argument
$ ./target/release/memory-hierarchy long-trace.dat > output.txt
```

#### Logging

If you want to run with logging, use the `RUST_LOG` environment variable. You can choose from `info`, `debug`, or `trace` log levels for increasing verbosity.

```bash
$ # Run the program with `info` log-level
$ RUST_LOG=info ./target/release/memory-hierarchy < long-trace.dat > output.txt
```