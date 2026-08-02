# Benchmark Methodology

## Environment Setup
All benchmarks were executed within a sandboxed Docker container running a standard Linux environment to ensure isolation and reproducibility.

## Compilation Flags
To ensure a mathematically fair comparison, both engines were compiled with optimizations enabled for their respective toolchains:
* **C99 Engine (Original):** Built using standard `gcc -O3` optimizations via the original Makefile.
* **Rust Engine (Port):** Compiled using `cargo build --release` to enforce zero-cost abstractions and maximum binary optimization.

## The Workload
[cite_start]The tests were conducted using the completely unmodified `test-bench.c` script provided in the original MathEX repository[cite: 5452]. [cite_start]This script evaluates a variety of mathematical expressions 1,000,000 times in a loop to capture reliable aggregate metrics[cite: 5439, 5452].

## Metrics Captured
The benchmark harness tracks two primary indicators of engine performance:
1. **Average Execution Time:** Measured in nanoseconds per operation (ns/op).
2. **Throughput:** Measured in operations per second (op/sec).