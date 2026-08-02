# MathEX-Rust 

> 🎥 **[Watch the 1-Minute Project Demo Video](YOUR_VIDEO_URL_HERE):** https://drive.google.com/file/d/1xWhsTwlxDwhguf4VKKfJhIA4TL04gfrv/view?usp=drive_link

> **A 100% Memory-Safe, Zero-Cost Rust Rewrite of the MathEX C99 Engine**

**MathEX-rs** is a complete, bottom-up rewrite of the legacy **MathEX C99** mathematical expression evaluator into safe, idiomatic Rust. It guarantees **100% C ABI drop-in compatibility**, passes the original unmodified test suite, and eliminates all unsafe pointer arithmetic and manual memory management—without sacrificing execution speed.

---

## Migration Rationale: Why Rewrite?

The original MathEX engine was a powerful but fragile piece of C software. It relied on several low-level implementation techniques that made the code difficult to maintain and prone to memory safety issues.

### Original C Implementation

- **Manual Memory Management**
  - Extensive use of `calloc`, `realloc`, and `free` across dynamically allocated arrays.
- **Unsafe Type-Punning**
  - AST nodes were represented using C `union`s, creating opportunities for invalid variant access and null-pointer dereferencing.
- **Tangled Architecture**
  - Tokenization, parsing, and evaluation were tightly coupled in a single execution pass using raw pointer arithmetic and C library functions such as `strncmp` and `strtof`.

### The Rust Solution

The unsafe core was replaced with modern, zero-cost Rust abstractions.

- **Strict Pipeline Separation**
  - A clean three-stage architecture:

    ```text
    Lexer → Parser → Evaluator
    ```

    This cleanly separates syntax analysis from runtime evaluation.

- **Tagged Enums**
  - The AST is represented using strongly typed Rust enums wrapped in `Box<T>`, eliminating invalid variant access and null-pointer dereferences at compile time.

- **Stateful `HashMap`s**
  - Native `HashMap<String, f32>` storage replaces raw linked-list traversal for variable management, providing average **O(1)** lookups while preventing use-after-free bugs through Rust's ownership model.

---

## Hackathon Compliance Statement

This submission strictly adheres to all hackathon requirements.

###  No Source-Language Runtime

The Rust implementation is a completely independent library.

It does **not** depend on:

- Original C binaries
- Original C runtime libraries
- Original C source code (other than the unmodified legacy test suite)

---

###  ABI Drop-In Compatibility

The engine exposes the original public API through a compatible C FFI:

- `expr_create`
- `expr_eval`
- `expr_destroy`

The Rust implementation preserves the original **32-byte C struct layout**, including padding and alignment, ensuring binary compatibility.

---

###  100% Test Suite Compatibility

The Rust implementation passes the complete, unmodified legacy test suite, including:

- `test-unit.c`
- `test-simple.c`
- `test-bench.c`

> For a detailed explanation of the architectural decisions made during the rewrite, see **`DECISIONS.md`**.

---


##  Quick Start: Testing & Benchmarking

We provide two ways to evaluate this submission:

1. **Docker (Recommended)** — A reproducible, zero-setup environment.
2. **Native Linux** — Compile and run everything directly on your system.

### Option 1: Frictionless Docker (Recommended)

Our multi-stage `Dockerfile` guarantees a clean, reproducible environment. It compiles the Rust library, statically links it against the legacy C test suite, and executes all three test runners:

- `test-unit`
- `test-simple`
- `test-bench`

#### 1. Build the Docker image

```bash
docker build -t mathex-rs .
```

#### 2. Run the test suite and benchmarks

```bash
docker run --rm mathex-rs
```

---

### Option 2: Native Linux Compilation

If you prefer to build and run the project locally, ensure you have the following installed:

- Rust toolchain (`cargo` and `rustc`)
- GCC
- Standard system libraries (`pthread`, `dl`, and `libm`)

#### 1. Build the optimized Rust library

```bash
cargo build --release
```

#### 2. Compile the legacy C test suite against the Rust FFI

```bash
gcc -O3 tests/original/test-unit.c \
    -L./target/release \
    -lmathex -lpthread -ldl -lm \
    -o test-unit

gcc -O3 tests/original/test-simple.c \
    -L./target/release \
    -lmathex -lpthread -ldl -lm \
    -o test-simple

gcc -O3 tests/original/test-bench.c \
    -L./target/release \
    -lmathex -lpthread -ldl -lm \
    -o test-bench
```

#### 3. Execute the test runners

```bash
./test-unit
./test-simple
./test-bench
```

All three executables should complete successfully, demonstrating full compatibility with the original MathEX C test suite while running against the Rust implementation.

---

##  Performance

Compiled using:

```bash
cargo build --release
```

The Rust implementation generates highly optimized machine code that matches the performance class of the original C implementation while providing complete memory safety.

### Benchmark Results

Based on **1,000,000 iterations** (`test-bench.c`):

| Benchmark | Performance |
|-----------|------------:|
| Raw literal parsing (`5`) | \~11.72 ns/op (~85,000,000 ops/sec) |
| Variable assignment (`x = 5`) | \~66.71 ns/op (~14,000,000 ops/sec) |

Additional benchmark methodology and raw benchmark data are available in the **`bench/`** directory.

---

##  Performance & Safety Validation

* **Fuzzing & Memory Safety:** During the differential fuzzing setup, modern GCC immediately flagged `-Wformat-truncation` warnings in the legacy C codebase regarding unsafe buffer sizes. Our Rust port fundamentally eliminates these classes of bugs by relying on safe string formatting (`format!()`) and memory-safe abstractions, compiling completely silently.

##  Bonus: Upstream Bug Discovered

By running a 60-second differential fuzzer (`libFuzzer` + `AddressSanitizer`) against both engines, we discovered a latent memory vulnerability (Out-Of-Bounds Read / SEGV) in the original upstream C repository when handling empty strings or specific control characters.

Because our Rust engine relies on safe Enums and strict bounds checking, it gracefully evaluates these edge cases as `NaN` without crashing.

* **Upstream Issue Filed:** [[Issue](https://github.com/jserv/MathEX/issues/1)]
---
##  License

Code is distributed under MIT X License that can be found in the `LICENSE` file.
