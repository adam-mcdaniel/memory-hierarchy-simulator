# Programming Assignment \#1

***COSC 530 - Adam McDaniel***
---

## Overview

Here is a table of the capabilities of my simulator.


|DC Strategy|L2 Strategy|Works for Read-Only Long-Trace|Works for Write-Only Long-Trace|Works for Read + Write Long-Trace|
|:---------:|:---------:|:----------------------------:|:-----------------------------:|:-----------------------------:|
|Write Through|Write Through|✅|✅|✅|
|Write Allocate|Write Through|✅|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|
|Write Through|Write Allocate|✅|✅ (**trace 100% correct**; main memory ref counter incorrect)|✅❔(8/866428 lines of output differ from reference implementation output; only 5 trace operation results differ)|
|Write Allocate|Write Allocate|✅|✅❔ (**trace 100% correct**; L2 hits and main memory ref counters incorrect)|✅❓(361/866428 lines of output differ from reference implementation output)|

Here are the commands I used to calculate the differences in lines.
```bash
$ # Run my program, save to mine.txt
$ cargo run --release < trace.dat > mine.txt
$ # Run Jantz's program, save to solution.txt
$ ./memhier_ref < trace.dat > solution.txt
$ # Count the number of lines that start with a brace referencing a line from my file.
$ # This is exactly equal to the number of lines that differ between the files.
$ diff mine.txt solution.txt | grep "<" | wc
```

## Usage

To build my program, use `cargo`.

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