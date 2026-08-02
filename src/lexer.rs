#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Data
    Number(f32),
    Identifier(String), // Handles both variables like "x" and functions like "add"

    // Arithmetic [cite: 220]
    Plus, Minus, Star, Slash, Percent, Power,
    
    // Bitwise [cite: 220, 221]
    ShiftLeft, ShiftRight, BitAnd, BitOr, BitXor, BitNot,
    
    // Logical [cite: 221, 222]
    Eq, Neq, Less, Greater, LessEq, GreaterEq, LogAnd, LogOr, LogNot,
    
    // Syntax [cite: 222]
    Assign, // =
    Comma,  // ,
    LParen, // (
    RParen, // )
}

pub struct Lexer<'a> {
    // Instead of raw string indexing (which is unsafe in Rust due to UTF-8), 
    // we use a "Peekable Iterator" over the characters. 
    // This lets us look at the next character without actually consuming it—
    // perfect for checking if a '<' is followed by an '='.
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        // `while let` is a beautiful Rust idiom. It loops as long as `self.chars.peek()` 
        // successfully returns a character (Some). When it hits the end of the string (None), it stops.
        while let Some(&c) = self.chars.peek() {
            match c {
                // 1. Whitespace Management
                ' ' | '\t' | '\\' => {
                    self.chars.next(); // Consume the space and move on
                }
                
                // MathEX treats newlines and semicolons as Commas!
                '\n' | '\r' | ';' => {
                    self.chars.next();
                    if let Some(last_token) = tokens.last() {
                        match last_token {
                            Token::Number(_) | Token::Identifier(_) | Token::RParen => {
                                tokens.push(Token::Comma);
                            }
                            _ => {} // Ignore consecutive newlines, or newlines after operators (like `a=\n3`)
                        }
                    }
                }

                // 2. Parse Multi-Character and Single-Character Operators
                
                '=' => {
                    self.chars.next(); // Consume the first '='
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next(); // Consume the second '='
                        tokens.push(Token::Eq);
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '<' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LessEq);
                    } else if let Some(&'<') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::ShiftLeft);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::GreaterEq);
                    } else if let Some(&'>') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::ShiftRight);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '!' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::Neq);
                    } else {
                        tokens.push(Token::LogNot);
                    }
                }
                '&' => {
                    self.chars.next();
                    if let Some(&'&') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LogAnd);
                    } else {
                        tokens.push(Token::BitAnd);
                    }
                }
                '|' => {
                    self.chars.next();
                    if let Some(&'|') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LogOr);
                    } else {
                        tokens.push(Token::BitOr);
                    }
                }
                '*' => {
                    self.chars.next();
                    if let Some(&'*') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::Power); // MathEX supports ** for power
                    } else {
                        tokens.push(Token::Star);
                    }
                }
                
                // Single-character tokens
                '+' => { self.chars.next(); tokens.push(Token::Plus); }
                '-' => { self.chars.next(); tokens.push(Token::Minus); }
                '/' => { self.chars.next(); tokens.push(Token::Slash); }
                '%' => { self.chars.next(); tokens.push(Token::Percent); }
                '^' => { self.chars.next(); tokens.push(Token::BitXor); } // MathEX uses ^ for XOR/BitNot
                '(' => { self.chars.next(); tokens.push(Token::LParen); }
                ')' => { self.chars.next(); tokens.push(Token::RParen); }
                ',' => { self.chars.next(); tokens.push(Token::Comma); }

                // 3. Parse Numbers (e.g., 42.5)
                '0'..='9' | '.' => {
                    let mut num_str = String::new();
                    
                    // Keep peeking and consuming as long as it's a digit or a decimal
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() || next_c == '.' {
                            num_str.push(next_c);
                            self.chars.next(); // Consume it
                        } else {
                            break; // Stop when we hit a space or operator
                        }
                    }
                    
                    // Safely attempt to parse the collected string into an f32
                    match num_str.parse::<f32>() {
                        Ok(num) => tokens.push(Token::Number(num)),
                        Err(_) => return Err(format!("Invalid number format: {}", num_str)),
                    }
                }

                // 4. Parse Identifiers (Variables like 'x' and Functions like 'add')
                c if c.is_alphabetic() || c == '_' || c == '$' => {
                    let mut id_str = String::new();
                    
                    while let Some(&next_c) = self.chars.peek() {
                        // Identifiers can contain letters, numbers, underscores, or hashes
                        if next_c.is_alphanumeric() || next_c == '_' || next_c == '#' || next_c == '$' {
                            id_str.push(next_c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Identifier(id_str));
                }
                
                // Catch-all for unrecognized characters
                _ => {
                    return Err(format!("Unexpected character: {}", c));
                }
            }
        }

        Ok(tokens)
    }
}