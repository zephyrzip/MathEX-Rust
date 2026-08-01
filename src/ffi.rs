use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use crate::ast::Expr;
use crate::parser::Parser;
use crate::eval::Evaluator;

// Tell the compiler to ignore naming conventions for this specific struct
#[allow(non_camel_case_types)]
pub struct expr_var_list {
    pub evaluator: Evaluator,
}

// Tell the compiler to ignore naming conventions for this specific struct
#[allow(non_camel_case_types)]
pub struct expr {
    pub ast: Expr,
    pub vars: *mut expr_var_list, 
}

#[no_mangle]
pub extern "C" fn expr_create(
    s: *const c_char,
    _len: usize, // We ignore len because Rust determines string length safely via null-termination
    vars: *mut expr_var_list,
    _funcs: *mut c_void, // Simplified: ignoring custom C function pointers for this core port
) -> *mut expr {
    // Safety check: if the C string pointer is NULL, abort.
    if s.is_null() {
        return std::ptr::null_mut();
    }

    // UNSAFE: We trust that the C caller provided a valid, null-terminated string.
    let c_str = unsafe { CStr::from_ptr(s) };
    
    // Convert the C string into a safe Rust string slice
    let rust_str = match c_str.to_str() {
        Ok(valid_str) => valid_str,
        Err(_) => return std::ptr::null_mut(),
    };

    // Run our safe Rust Parser!
    let mut parser = match Parser::new(rust_str) {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };

    let ast = match parser.parse() {
        Ok(tree) => tree,
        Err(_) => return std::ptr::null_mut(),
    };

    // Box the data to allocate it on the heap, then leak the pointer to C 
    // so it doesn't get automatically cleaned up when this function ends.
    let e = Box::new(expr { ast, vars });
    Box::into_raw(e)
}

#[no_mangle]
pub extern "C" fn expr_eval(e: *mut expr) -> f32 {
    if e.is_null() {
        return 0.0;
    }

    // UNSAFE: We dereference the raw pointer to access our AST and Evaluator
    unsafe {
        let expr_ref = &*e;
        
        if expr_ref.vars.is_null() {
            // Evaluate without variables
            let mut temp_eval = Evaluator::new();
            temp_eval.eval(&expr_ref.ast).unwrap_or(0.0)
        } else {
            // Evaluate using the stateful variable list
            let vars_ref = &mut *expr_ref.vars;
            vars_ref.evaluator.eval(&expr_ref.ast).unwrap_or(0.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn expr_destroy(e: *mut expr, _vars: *mut expr_var_list) {
    if !e.is_null() {
        unsafe {
            let _ = Box::from_raw(e);
        }
    }
}