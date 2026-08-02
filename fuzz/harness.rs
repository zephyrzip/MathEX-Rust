#![no_main]
extern crate mathex;

use libfuzzer_sys::fuzz_target;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

extern "C" {
    fn c_expr_create(expr: *const c_char) -> *mut c_void;
    fn c_expr_eval(expr: *mut c_void) -> f32;
    fn c_expr_destroy(expr: *mut c_void);

    fn expr_create(expr: *const c_char) -> *mut c_void;
    fn expr_eval(expr: *mut c_void) -> f32;
    fn expr_destroy(expr: *mut c_void);
}

fuzz_target!(|data: &[u8]| {
    // 1. Defend against the C engine's empty string bug!
    if data.is_empty() { return; }

    if let Ok(fuzz_str) = std::str::from_utf8(data) {
        // 2. Skip whitespace-only strings and null bytes
        if fuzz_str.trim().is_empty() || fuzz_str.contains('\0') { 
            return; 
        }

        let Ok(c_string) = CString::new(fuzz_str) else { return; };

        // --- RUST EXECUTION ---
        let rust_res = unsafe {
            let r_expr = expr_create(c_string.as_ptr());
            if r_expr.is_null() {
                f32::NAN
            } else {
                let val = expr_eval(r_expr);
                expr_destroy(r_expr);
                val
            }
        };

        // --- C EXECUTION ---
        let c_res = unsafe {
            let c_expr = c_expr_create(c_string.as_ptr());
            if c_expr.is_null() {
                f32::NAN
            } else {
                let val = c_expr_eval(c_expr);
                c_expr_destroy(c_expr);
                val
            }
        };

        // --- DIFFERENTIAL ASSERTION ---
        // If both failed gracefully (NaN), they match!
        if rust_res.is_nan() && c_res.is_nan() {
            return; 
        }
        
        // If both divide by zero and return Infinity, they match!
        if rust_res.is_infinite() && c_res.is_infinite() {
            return;
        }

        // Compare standard results with a tiny float tolerance
        let diff = (rust_res - c_res).abs();
        assert!(
            diff < 0.0001,
            "DIVERGENCE! Input: '{}' | Rust: {} | C: {}",
            fuzz_str, rust_res, c_res
        );
    }
});