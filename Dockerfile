# ==========================================
# STAGE 1: Build the Rust Static Library
# ==========================================
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy your entire Rust project into the container
COPY . .

# Compile the Rust engine in release mode for maximum performance
RUN cargo build --release

# ==========================================
# STAGE 2: Compile C Tests and Run
# ==========================================
FROM gcc:latest

WORKDIR /app

# Copy the compiled Rust static library from the Stage 1 builder
# (In Linux, Rust static libraries are prefixed with 'lib' and end in '.a')
COPY --from=builder /app/target/release/libmathex.a /app/

# Copy the original unmodified C test suite
COPY tests/original/ /app/tests/original/

# Compile the C test runner, linking it to your Rust static library.
# We include -lm (Math), -lpthread, and -ldl which Rust requires on Linux.
RUN gcc tests/original/test-unit.c -I tests/original -L. -lmathex -lpthread -ldl -lm -o test-unit

# When the container runs, execute the tests!
CMD ["./test-unit"]