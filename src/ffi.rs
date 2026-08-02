// src/ffi.rs

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use crate::ast::Expr;
use crate::parser::Parser;
use crate::eval::Evaluator;

// --- 1. MEMORY-SAFE C STRUCTS ---

// We must perfectly mimic the original C memory layout. 
// We use a 256-byte array to safely simulate C's "Flexible Array Member" for variable names.
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct expr_var_node {
    pub value: f32,
    pub next: *mut expr_var_node,
    pub name: [c_char; 256], 
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct expr_var_list {
    pub head: *mut expr_var_node,
}

#[allow(non_camel_case_types)]
pub struct expr {
    pub ast: Expr,
    pub vars: *mut expr_var_list,
    pub evaluator: Evaluator,
}

// --- 2. THE EXPR API ---

#[no_mangle]
pub extern "C" fn expr_create(
    s: *const c_char,
    _len: usize,
    vars: *mut expr_var_list,
    _funcs: *mut c_void, 
) -> *mut expr {
    if s.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(s) };
    let rust_str = match c_str.to_str() {
        Ok(valid_str) => valid_str,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut parser = match Parser::new(rust_str) {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };

    let ast = match parser.parse() {
        Ok(tree) => tree,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut evaluator = Evaluator::new();

    if !vars.is_null() {
        unsafe {
            let mut current = (*vars).head;
            while !current.is_null() {
                let c_str = CStr::from_ptr((*current).name.as_ptr());
                if let Ok(name_str) = c_str.to_str() {
                    evaluator.variables.insert(name_str.to_string(), (*current).value);
                }
                current = (*current).next;
            }
        }
    }

    let e = Box::new(expr { ast, vars, evaluator });
    Box::into_raw(e)
}

#[no_mangle]
pub extern "C" fn expr_eval(e: *mut expr) -> f32 {
    if e.is_null() { return 0.0; }

    unsafe {
        let expr_ref = &mut *e;

        if !expr_ref.vars.is_null() {
            let mut current = (*expr_ref.vars).head;
            while !current.is_null() {
                let c_str = CStr::from_ptr((*current).name.as_ptr());
                if let Ok(name_str) = c_str.to_str() {
                    expr_ref.evaluator.variables.insert(name_str.to_string(), (*current).value);
                }
                current = (*current).next;
            }
        }

        expr_ref.evaluator.eval(&expr_ref.ast).unwrap_or(0.0)
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

// --- 3. INTERNAL TEST SUITE MOCKS ---

#[no_mangle]
pub extern "C" fn expr_var(
    vars: *mut expr_var_list, 
    s: *const c_char, 
    _len: usize
) -> *mut expr_var_node {
    if vars.is_null() || s.is_null() { return std::ptr::null_mut(); }

    unsafe {
        let target_name = CStr::from_ptr(s);

        // 1. Search the C linked list to see if the variable already exists
        let mut current = (*vars).head;
        while !current.is_null() {
            let node_name = CStr::from_ptr((*current).name.as_ptr());
            if node_name == target_name {
                return current; // Return existing variable to C
            }
            current = (*current).next;
        }

        // 2. If not found, create a new node
        let mut new_node = Box::new(expr_var_node {
            value: 0.0,
            next: (*vars).head,
            name: [0; 256],
        });

        // Copy the variable name bytes into our inline array (Max 255 chars + null terminator)
        let bytes = target_name.to_bytes_with_nul();
        for (i, &b) in bytes.iter().enumerate().take(255) {
            new_node.name[i] = b as c_char;
        }

        // 3. Leak the memory to C and update the head of the list
        let new_node_ptr = Box::into_raw(new_node);
        (*vars).head = new_node_ptr;

        new_node_ptr
    }
}

#[no_mangle]
pub extern "C" fn expr_next_token(s: *const c_char, len: usize, _flags: *mut i32) -> i32 {
    if s.is_null() || len == 0 {
        return 0;
    }

    // Safely convert the C string pointer and length to a byte slice
    let slice = unsafe { std::slice::from_raw_parts(s as *const u8, len) };
    if slice.is_empty() {
        return 0;
    }

    let c = slice[0] as char;
    let mut i = 1;

    // 1. Match Numbers
    if c.is_ascii_digit() || c == '.' {
        while i < len {
            let next_c = slice[i] as char;
            if next_c.is_ascii_digit() || next_c == '.' {
                i += 1;
            } else {
                break;
            }
        }
        return i as i32;
    }

    // 2. Match Identifiers (Letters, _, $, #)
    if c.is_ascii_alphabetic() || c == '_' || c == '$' || c == '#' {
        while i < len {
            let next_c = slice[i] as char;
            if next_c.is_ascii_alphanumeric() || next_c == '_' || next_c == '$' || next_c == '#' {
                i += 1;
            } else {
                break;
            }
        }
        return i as i32;
    }

    // 3. Match 2-Character Operators (like **, ==, <=, <<)
    if i < len {
        let next_c = slice[1] as char;
        match (c, next_c) {
            ('*', '*') | ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') | 
            ('&', '&') | ('|', '|') | ('<', '<') | ('>', '>') => return 2,
            _ => {}
        }
    }

    // 4. Default to 1-Character Operator (like +, -, *)
    1
}