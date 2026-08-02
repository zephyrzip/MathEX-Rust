use crate::ast::{Expr, BinaryOp, UnaryOp};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    // We use a simple index to track which token we are currently looking at.
    current: usize, 
    known_funcs: std::collections::HashSet<String>,
}

impl Parser {
    // We update `new` to run the Lexer immediately. If the Lexer fails 
    // (e.g., an invalid character), the Parser immediately returns that Error.
    pub fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        
        let mut known_funcs = std::collections::HashSet::new();
        // Pre-populate with the C test functions and the macro operator
        for f in ["add", "next", "nop", "print", "$"] {
            known_funcs.insert(f.to_string());
        }

        Ok(Self {
            tokens,
            current: 0,
            known_funcs
        })
    }

    // The entry point for our parser.
    pub fn parse(&mut self) -> Result<Expr, String> {
        // MathEX expects empty strings to evaluate to NaN
        if self.tokens.is_empty() {
            return Ok(Expr::Number(0.0));
        }
        // We start at the lowest precedence level.
        let expr = self.expression()?;
        
        // If we finished parsing but still have tokens left over, 
        // the user typed something invalid like "42 5" instead of "42 + 5".
        if !self.is_at_end() {
            return Err("Unexpected tokens at the end of the expression".to_string());
        }
        
        Ok(expr)
    }

    // --- Core Recursive Descent Entry Point ---
    
    fn expression(&mut self) -> Result<Expr, String> {
        // We start at the absolute lowest precedence: The Comma
        self.comma()
    }

    // --- Comma (,) ---
    
    fn comma(&mut self) -> Result<Expr, String> {
        let mut expr = self.assignment()?;
        
        while let Some(Token::Comma) = self.peek() {
            self.advance(); // Consume the comma
            
            // Skip consecutive commas or trailing commas
            if self.is_at_end() || self.peek() == Some(&Token::Comma) || self.peek() == Some(&Token::RParen) {
                break; 
            }
            
            let right = self.assignment()?;
            expr = Expr::Binary {
                op: BinaryOp::Comma,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // --- Assignment (=) ---
    
    fn assignment(&mut self) -> Result<Expr, String> {
        // Evaluate the left side
        let expr = self.logical_or()?;

        // MathEX supports chained assignments like `x = y = 5`.
        // To handle this properly, assignment is RIGHT-associative.
        if let Some(Token::Assign) = self.peek() {
            self.advance();
            // We recursively call `assignment()` instead of moving down the chain
            let value = self.assignment()?; 

            if !matches!(expr, Expr::Variable(_)) {
                return Err("The left side of an assignment must be a variable".to_string());
            }
            
            return Ok(Expr::Binary {
                op: BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
            });
        }
        Ok(expr)
    }

    // --- Logical OR (||) ---
    
    fn logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logical_and()?;
        while let Some(Token::LogOr) = self.peek() {
            self.advance();
            let right = self.logical_and()?;
            expr = Expr::Binary { op: BinaryOp::LogOr, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Logical AND (&&) ---
    
    fn logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_or()?;
        while let Some(Token::LogAnd) = self.peek() {
            self.advance();
            let right = self.bitwise_or()?;
            expr = Expr::Binary { op: BinaryOp::LogAnd, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Bitwise OR (|) ---
    
    fn bitwise_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_xor()?;
        while let Some(Token::BitOr) = self.peek() {
            self.advance();
            let right = self.bitwise_xor()?;
            expr = Expr::Binary { op: BinaryOp::BitOr, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Bitwise XOR (^) ---
    
    fn bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_and()?;
        while let Some(Token::BitXor) = self.peek() {
            self.advance();
            let right = self.bitwise_and()?;
            expr = Expr::Binary { op: BinaryOp::BitXor, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Bitwise AND (&) ---
    
    fn bitwise_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while let Some(Token::BitAnd) = self.peek() {
            self.advance();
            let right = self.equality()?;
            expr = Expr::Binary { op: BinaryOp::BitAnd, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Equality (==, !=) ---
    
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.relational()?;
        while let Some(token) = self.peek() {
            let op = match token {
                Token::Eq => BinaryOp::Eq,
                Token::Neq => BinaryOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.relational()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Relational (<, >, <=, >=) ---
    
    fn relational(&mut self) -> Result<Expr, String> {
        let mut expr = self.shift()?;
        while let Some(token) = self.peek() {
            let op = match token {
                Token::Less => BinaryOp::Lt,
                Token::Greater => BinaryOp::Gt,
                Token::LessEq => BinaryOp::Le,
                Token::GreaterEq => BinaryOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.shift()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Bitwise Shift (<<, >>) ---
    
    fn shift(&mut self) -> Result<Expr, String> {
        // This is the crucial link! It calls `self.term()`, connecting 
        // the top half of our parser directly into the arithmetic we already wrote.
        let mut expr = self.term()?;
        while let Some(token) = self.peek() {
            let op = match token {
                Token::ShiftLeft => BinaryOp::Shl,
                Token::ShiftRight => BinaryOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.term()?;
            expr = Expr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    // --- Addition and Subtraction ---
    
    fn term(&mut self) -> Result<Expr, String> {
        // First, grab the higher-precedence left side (multiplication/division)
        let mut expr = self.factor()?;

        // While the next token is a '+' or '-', keep chaining them to the tree
        while let Some(token) = self.peek() {
            let op = match token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break, // If it's not + or -, we are done with this term
            };
            
            self.advance(); // Consume the operator
            let right = self.factor()?; // Parse the right side
            
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // --- Multiplication, Division, and Remainder ---
    
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.power()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Rem,
                _ => break,
            };
            
            self.advance();
            let right = self.power()?;
            
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // --- Exponents (Power) ---
    
    fn power(&mut self) -> Result<Expr, String> {
        let expr = self.unary()?;

        if let Some(Token::Power) = self.peek() {
            self.advance();
            let right = self.power()?; 
            
            return Ok(Expr::Binary {
                op: BinaryOp::Pow,
                left: Box::new(expr),
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    // --- Unary Prefix Operators (-x, !x, ^x) ---
    
    fn unary(&mut self) -> Result<Expr, String> {
        if let Some(token) = self.peek() {
            let op = match token {
                Token::Minus => Some(UnaryOp::Neg),
                Token::LogNot => Some(UnaryOp::Not),
                Token::BitXor => Some(UnaryOp::BitNot), // MathEX uses ^ for BitNot
                _ => None,
            };
            
            if let Some(unary_op) = op {
                self.advance(); // Consume the prefix operator
                // Recursively call `unary` to handle nested prefixes like `!!true` or `-(-5)`
                let operand = self.unary()?; 
                
                return Ok(Expr::Unary {
                    op: unary_op,
                    operand: Box::new(operand),
                });
            }
        }
        
        // If there is no unary prefix, drop down to the numbers/variables
        self.primary()
    }

    // --- The Bottom of the Chain (Numbers, Variables, Functions, Grouping) ---
    
    fn primary(&mut self) -> Result<Expr, String> {
        // --- 1. ADD THIS SAFETY CHECK ---
        if self.is_at_end() {
            return Err("Unexpected end of expression. Expected a number or variable.".to_string());
        }
        // --------------------------------

        let token = self.advance().clone();

        match token {
            Token::Number(n) => Ok(Expr::Number(n)),
            
            Token::Identifier(name) => {
                if let Some(Token::LParen) = self.peek() {
                self.advance(); 
                let mut args = Vec::new();
                if self.peek() != Some(&Token::RParen) {
                    loop {
                        args.push(self.assignment()?);
                        if let Some(Token::Comma) = self.peek() { self.advance(); } else { break; }
                    }
                }
                self.consume(&Token::RParen, "Expected ')' after function arguments.")?;
        
                 // --- PARSE TIME VALIDATION ---
                if name == "$" {
                    if args.is_empty() { return Err("$ requires at least 1 argument".to_string()); }
                    if let Expr::Variable(macro_name) = &args[0] {
                        self.known_funcs.insert(macro_name.clone()); // Register macro dynamically!
                    } else {
                        return Err("First argument of $ must be an identifier".to_string());
                    }
                } else if !self.known_funcs.contains(&name) {
                    return Err(format!("Unknown function at parse time: {}", name));
                }
        
                Ok(Expr::FunctionCall { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            
            Token::LParen => {
                let expr = self.expression()?;
                self.consume(&Token::RParen, "Expected ')' after expression.")?;
                Ok(expr)
            }
            
            _ => Err(format!("Expected a number, variable, or '(', but found {:?}", token)),
        }
    }

    // --- Helper Methods ---

    // Look at the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    // Move to the next token and return the one we just passed.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    // Check if we've run out of tokens.
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    // Require a specific token to be next, or throw an error.
    fn consume(&mut self, expected: &Token, error_message: &str) -> Result<&Token, String> {
        if let Some(token) = self.peek() {
            if token == expected {
                return Ok(self.advance());
            }
        }
        Err(error_message.to_string())
    }
}