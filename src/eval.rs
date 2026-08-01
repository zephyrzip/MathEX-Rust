// src/eval.rs

use std::collections::HashMap;
use crate::ast::{Expr, BinaryOp, UnaryOp};

// The Evaluator acts as the memory state for our math engine.
// It replaces the need for `struct expr_var_list` and `struct expr_func` from C.
pub struct Evaluator {
    // Stores our variable values (e.g., "x" -> 42.0)
    pub variables: HashMap<String, f32>,
    
    // Stores custom functions. 
    // This is a map of a String name to a safe Rust function pointer.
    pub functions: HashMap<String, fn(&[f32]) -> Result<f32, String>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    // The core recursive evaluation engine.
    pub fn eval(&mut self, expr: &Expr) -> Result<f32, String> {
        match expr {
            Expr::Number(n) => Ok(*n),
            
            Expr::Variable(name) => {
                // If the variable exists in our map, return it.
                // If it doesn't, MathEX's default behavior is to treat it as 0.0.
                Ok(*self.variables.get(name).unwrap_or(&0.0))
            }

            Expr::Binary { op, left, right } => {
                // --- SPECIAL CASES ---

                // 1. Assignment (=)
                // We cannot evaluate the left side normally, because it's a variable name, 
                // not a mathematical calculation.
                if let BinaryOp::Assign = op {
                    let value = self.eval(right)?;
                    if let Expr::Variable(name) = &**left {
                        // Insert or update the variable in our HashMap
                        self.variables.insert(name.clone(), value);
                        return Ok(value);
                    } else {
                        return Err("The left side of an assignment must be a variable".to_string());
                    }
                }

                // 2. Logical Short-Circuiting (&&, ||)
                // We only evaluate the right side if the left side doesn't resolve the logic.
                if let BinaryOp::LogAnd = op {
                    let left_val = self.eval(left)?;
                    if left_val == 0.0 { return Ok(0.0); }
                    let right_val = self.eval(right)?;
                    return Ok(if right_val != 0.0 { 1.0 } else { 0.0 });
                }
                
                if let BinaryOp::LogOr = op {
                    let left_val = self.eval(left)?;
                    if left_val != 0.0 { return Ok(1.0); }
                    let right_val = self.eval(right)?;
                    return Ok(if right_val != 0.0 { 1.0 } else { 0.0 });
                }

                // --- STANDARD BINARY OPERATIONS ---
                
                // Evaluate both branches of the AST
                let l = self.eval(left)?;
                let r = self.eval(right)?;

                match op {
                    // Arithmetics
                    BinaryOp::Add => Ok(l + r),
                    BinaryOp::Sub => Ok(l - r),
                    BinaryOp::Mul => Ok(l * r),
                    BinaryOp::Div => Ok(l / r), // Rust naturally handles f32 / 0.0 as Infinity (C99 compliant)
                    BinaryOp::Rem => Ok(l % r),
                    BinaryOp::Pow => Ok(l.powf(r)),
                    
                    // Notice the extra outer parentheses wrapping the whole operation!
                    BinaryOp::BitAnd => Ok(((l as i32) & (r as i32)) as f32),
                    BinaryOp::BitOr  => Ok(((l as i32) | (r as i32)) as f32),
                    BinaryOp::BitXor => Ok(((l as i32) ^ (r as i32)) as f32),
                    BinaryOp::Shl    => Ok(((l as i32) << (r as i32)) as f32),
                    BinaryOp::Shr    => Ok(((l as i32) >> (r as i32)) as f32),

                    // Logical & Relational (1.0 = True, 0.0 = False)
                    BinaryOp::Eq  => Ok(if l == r { 1.0 } else { 0.0 }),
                    BinaryOp::Neq => Ok(if l != r { 1.0 } else { 0.0 }),
                    BinaryOp::Lt  => Ok(if l < r { 1.0 } else { 0.0 }),
                    BinaryOp::Gt  => Ok(if l > r { 1.0 } else { 0.0 }),
                    BinaryOp::Le  => Ok(if l <= r { 1.0 } else { 0.0 }),
                    BinaryOp::Ge  => Ok(if l >= r { 1.0 } else { 0.0 }),
                    
                    // The Comma operator executes the left (already done above), discards it, and returns the right.
                    BinaryOp::Comma => Ok(r), 
                    
                    // These were handled in our Special Cases block above!
                    BinaryOp::Assign | BinaryOp::LogAnd | BinaryOp::LogOr => unreachable!(),
                }
            }

            Expr::Unary { op, operand } => {
                let val = self.eval(operand)?;
                match op {
                    UnaryOp::Neg => Ok(-val),
                    UnaryOp::Not => Ok(if val == 0.0 { 1.0 } else { 0.0 }),
                    UnaryOp::BitNot => Ok(!(val as i32) as f32),
                }
            }

            Expr::FunctionCall { name, args } => {
                // Traverse and evaluate every argument inside the function parentheses
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }

                // Look up the function pointer and execute it
                if let Some(func) = self.functions.get(name) {
                    func(&evaluated_args)
                } else {
                    Err(format!("Unknown function called: {}", name))
                }
            }
        }
    }
}