# MathEX C99 → Rust Port: Engineering Decisions

This document outlines the core engineering decisions made while porting the **MathEX C99 engine** to safe, idiomatic Rust. The primary goal was to completely eliminate unsafe memory management and pointer arithmetic while maintaining a **100% drop-in compatible C ABI** capable of passing the original, unmodified test suite.

---

## 1. Pipeline Architecture: Dedicated Lexer & Parser Separation

### Original C Architecture

Tokenization, AST construction, and memory traversal were tightly coupled using standard C library functions such as `strncmp` and `strtof`. This design relied heavily on inline token checks and manual string slicing.

### Rust Port

Implemented a strict three-stage pipeline:

```text
Lexer → Parser → Evaluator
```

### Rationale

- Eliminates unsafe string pointer arithmetic.
- Uses a dedicated `Peekable` iterator for multi-character operators such as `==` and `**`.
- Removes whitespace during lexing.
- Isolates syntax errors to the parsing stage.
- Produces a cleaner and more maintainable architecture.

---

## 2. Abstract Syntax Tree (AST): Tagged Enums vs. C Unions

### Original C Architecture

The `struct expr` type relied on C `union`s and type-punning, introducing risks such as:

- Invalid variant access
- Null-pointer dereferencing
- Undefined behavior

### Rust Port

Replaced the union-based representation with data-carrying enums, for example:

- `Expr::Binary`
- `Expr::Variable`
- `Expr::Literal`

Recursive nodes are stored using heap-allocated `Box<T>` smart pointers.

### Rationale

- Provides strict compile-time type safety.
- Eliminates invalid variant access.
- Prevents null-pointer dereferences because enum variants guarantee the existence of their associated data.

---

## 3. Memory Ownership: `HashMap` vs. Manual Linked Lists

### Original C Architecture

Variables were stored in dynamically allocated arrays managed with:

- `calloc`
- `realloc`
- `free`

Traversal occurred through raw pointers using `struct expr_var_node`.

### Rust Port

Implemented a native:

```rust
HashMap<String, f32>
```

managed within a stateful `Evaluator` structure.

### Rationale

- Provides **O(1)** average lookup performance.
- Uses Rust's ownership model for automatic cleanup.
- Eliminates manual memory management.
- Prevents use-after-free bugs during recursive evaluation.

---

## 4. Dynamic C ABI Bridge (FFI)

### Original C Architecture

The public API exposed functions such as:

- `expr_create`
- `expr_eval`

These accepted raw pointers to arrays of C structures.

### Rust Port

Implemented:

- `#[repr(C)]` for layout compatibility.
- `#[allow(non_camel_case_types)]` to preserve C naming.
- Safe traversal of null-terminated C arrays using:

```rust
while !(*current).name.is_null()
```

alongside carefully controlled pointer arithmetic within the FFI boundary.

### Rationale

- Preserves binary compatibility with the original C API.
- Matches the original **32-byte structure alignment**, including intermediate function pointers.
- Prevents segmentation faults caused by ABI or padding mismatches.

---

## 5. MathEX Compatibility Rules & Legacy Quirks

To achieve a **100% pass rate** against the original legacy test suite, several non-standard behaviors from the C implementation were intentionally preserved.

### Right-Associative Exponentiation

Exponentiation was modified to associate from right to left.

Example:

```text
2^(2^3) = 256
```

rather than

```text
(2^2)^3 = 64
```

---

### Truthy/Falsy Semantics

Logical operators replicate MathEX's JavaScript-like behavior by returning the evaluated truthy operand rather than a boolean value.

Example:

```text
2 && 3  →  3.0
```

---

### Flexible Lexer Rules

The lexer was extended to support:

- Unicode identifiers (e.g., Ukrainian, Chinese, Greek)
- Newlines (`\n`) as interchangeable comma separators

This preserves compatibility while making the language more flexible.

---

# Summary

The Rust port replaces unsafe C constructs with safe, idiomatic Rust while maintaining complete binary compatibility with the original C interface. Key improvements include:

- A dedicated **Lexer → Parser → Evaluator** pipeline
- Strongly typed enum-based ASTs
- Ownership-driven memory safety
- A compatible C ABI via `#[repr(C)]`
- Faithful recreation of legacy MathEX behaviors required for complete test-suite compatibility