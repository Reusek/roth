//! Builtin operations for the Forth runtime.
//!
//! These are implemented as methods on RuntimeContext and are called
//! directly from generated code.

use crate::context::RuntimeContext;
use crate::error::{ForthError, ForthResult, SourceLocation};
use std::io::{self, Write};

impl RuntimeContext {
    // =========================================================================
    // Stack Operations
    // =========================================================================

    /// DUP: Duplicate the top stack element.
    /// ( a -- a a )
    pub fn dup(&mut self) -> ForthResult<()> {
        let a = self.peek()?;
        self.push(a)
    }

    /// DROP: Remove the top stack element.
    /// ( a -- )
    pub fn drop_top(&mut self) -> ForthResult<()> {
        self.pop()?;
        Ok(())
    }

    /// SWAP: Swap the top two stack elements.
    /// ( a b -- b a )
    pub fn swap(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(b);
        self.push(a)
    }

    /// OVER: Copy the second element to the top.
    /// ( a b -- a b a )
    pub fn over(&mut self) -> ForthResult<()> {
        let a = self.peek_n(1)?;
        self.push(a)
    }

    /// ROT: Rotate the top three elements.
    /// ( a b c -- b c a )
    pub fn rot(&mut self) -> ForthResult<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(b);
        self.stack.push(c);
        self.push(a)
    }

    /// -ROT: Reverse rotate the top three elements.
    /// ( a b c -- c a b )
    pub fn rot_rev(&mut self) -> ForthResult<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(c);
        self.stack.push(a);
        self.push(b)
    }

    /// NIP: Remove the second element.
    /// ( a b -- b )
    pub fn nip(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        self.pop()?;
        self.push(b)
    }

    /// TUCK: Copy top element under second.
    /// ( a b -- b a b )
    pub fn tuck(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(b);
        self.stack.push(a);
        self.push(b)
    }

    /// 2DUP: Duplicate the top two elements.
    /// ( a b -- a b a b )
    pub fn dup2(&mut self) -> ForthResult<()> {
        let b = self.peek_n(0)?;
        let a = self.peek_n(1)?;
        self.stack.push(a);
        self.push(b)
    }

    /// 2DROP: Remove the top two elements.
    /// ( a b -- )
    pub fn drop2(&mut self) -> ForthResult<()> {
        self.pop()?;
        self.pop()?;
        Ok(())
    }

    /// 2SWAP: Swap the top two pairs.
    /// ( a b c d -- c d a b )
    pub fn swap2(&mut self) -> ForthResult<()> {
        let d = self.pop()?;
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(c);
        self.stack.push(d);
        self.stack.push(a);
        self.push(b)
    }

    /// 2OVER: Copy second pair to top.
    /// ( a b c d -- a b c d a b )
    pub fn over2(&mut self) -> ForthResult<()> {
        let a = self.peek_n(3)?;
        let b = self.peek_n(2)?;
        self.stack.push(a);
        self.push(b)
    }

    /// ?DUP: Duplicate top if non-zero.
    /// ( a -- a a ) if a != 0, else ( a -- a )
    pub fn dup_if_nonzero(&mut self) -> ForthResult<()> {
        let a = self.peek()?;
        if a != 0 {
            self.push(a)?;
        }
        Ok(())
    }

    /// PICK: Copy nth element to top.
    /// ( xn ... x1 x0 n -- xn ... x1 x0 xn )
    pub fn pick(&mut self) -> ForthResult<()> {
        let n = self.pop()? as usize;
        let val = self.peek_n(n)?;
        self.push(val)
    }

    /// ROLL: Move nth element to top.
    /// ( xn xn-1 ... x0 n -- xn-1 ... x0 xn )
    pub fn roll(&mut self) -> ForthResult<()> {
        let n = self.pop()? as usize;
        let len = self.stack.len();
        if n >= len {
            return Err(self.underflow_error_internal());
        }
        let idx = len - 1 - n;
        let val = self.stack.remove(idx);
        self.push(val)
    }

    // Internal helper for underflow errors
    fn underflow_error_internal(&self) -> ForthError {
        ForthError::StackUnderflow {
            location: self.current_location.clone(),
        }
    }

    // =========================================================================
    // Arithmetic Operations
    // =========================================================================

    /// +: Add top two elements.
    /// ( a b -- a+b )
    pub fn add(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_add(b))
    }

    /// -: Subtract top from second.
    /// ( a b -- a-b )
    pub fn sub(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_sub(b))
    }

    /// *: Multiply top two elements.
    /// ( a b -- a*b )
    pub fn mul(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_mul(b))
    }

    /// /: Divide second by top.
    /// ( a b -- a/b )
    pub fn div(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        if b == 0 {
            return Err(ForthError::DivisionByZero {
                location: self.current_location.clone(),
            });
        }
        self.push(a / b)
    }

    /// MOD: Remainder of second divided by top.
    /// ( a b -- a%b )
    pub fn modulo(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        if b == 0 {
            return Err(ForthError::DivisionByZero {
                location: self.current_location.clone(),
            });
        }
        self.push(a % b)
    }

    /// /MOD: Division with remainder.
    /// ( a b -- a%b a/b )
    pub fn divmod(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        if b == 0 {
            return Err(ForthError::DivisionByZero {
                location: self.current_location.clone(),
            });
        }
        self.stack.push(a % b);
        self.push(a / b)
    }

    /// NEGATE: Negate top element.
    /// ( a -- -a )
    pub fn negate(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(-a)
    }

    /// ABS: Absolute value.
    /// ( a -- |a| )
    pub fn abs(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(a.abs())
    }

    /// MIN: Minimum of top two.
    /// ( a b -- min(a,b) )
    pub fn min(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.min(b))
    }

    /// MAX: Maximum of top two.
    /// ( a b -- max(a,b) )
    pub fn max(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.max(b))
    }

    /// 1+: Increment top.
    /// ( a -- a+1 )
    pub fn inc(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(a.wrapping_add(1))
    }

    /// 1-: Decrement top.
    /// ( a -- a-1 )
    pub fn dec(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(a.wrapping_sub(1))
    }

    /// 2*: Double (left shift by 1).
    /// ( a -- a*2 )
    pub fn double(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(a << 1)
    }

    /// 2/: Halve (arithmetic right shift by 1).
    /// ( a -- a/2 )
    pub fn halve(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(a >> 1)
    }

    // =========================================================================
    // Comparison Operations
    // =========================================================================

    /// =: Test equality.
    /// ( a b -- flag )
    pub fn eq(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a == b { -1 } else { 0 })
    }

    /// <>: Test inequality.
    /// ( a b -- flag )
    pub fn ne(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a != b { -1 } else { 0 })
    }

    /// <: Test less than.
    /// ( a b -- flag )
    pub fn lt(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a < b { -1 } else { 0 })
    }

    /// >: Test greater than.
    /// ( a b -- flag )
    pub fn gt(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a > b { -1 } else { 0 })
    }

    /// <=: Test less than or equal.
    /// ( a b -- flag )
    pub fn le(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a <= b { -1 } else { 0 })
    }

    /// >=: Test greater than or equal.
    /// ( a b -- flag )
    pub fn ge(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a >= b { -1 } else { 0 })
    }

    /// 0=: Test if zero.
    /// ( a -- flag )
    pub fn zero_eq(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(if a == 0 { -1 } else { 0 })
    }

    /// 0<: Test if negative.
    /// ( a -- flag )
    pub fn zero_lt(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(if a < 0 { -1 } else { 0 })
    }

    /// 0>: Test if positive.
    /// ( a -- flag )
    pub fn zero_gt(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(if a > 0 { -1 } else { 0 })
    }

    // =========================================================================
    // Logical Operations
    // =========================================================================

    /// AND: Bitwise AND.
    /// ( a b -- a&b )
    pub fn and(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a & b)
    }

    /// OR: Bitwise OR.
    /// ( a b -- a|b )
    pub fn or(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a | b)
    }

    /// XOR: Bitwise XOR.
    /// ( a b -- a^b )
    pub fn xor(&mut self) -> ForthResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a ^ b)
    }

    /// INVERT: Bitwise NOT.
    /// ( a -- ~a )
    pub fn invert(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.push(!a)
    }

    /// LSHIFT: Left shift.
    /// ( a n -- a<<n )
    pub fn lshift(&mut self) -> ForthResult<()> {
        let n = self.pop()? as u32;
        let a = self.pop()?;
        self.push(a << n)
    }

    /// RSHIFT: Logical right shift.
    /// ( a n -- a>>n )
    pub fn rshift(&mut self) -> ForthResult<()> {
        let n = self.pop()? as u32;
        let a = self.pop()? as u64;
        self.push((a >> n) as i64)
    }

    // =========================================================================
    // I/O Operations
    // =========================================================================

    /// . (DOT): Print and remove top of stack.
    /// ( a -- )
    pub fn print_top(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        print!("{} ", a);
        io::stdout().flush().map_err(|e| ForthError::IOError {
            message: e.to_string(),
            location: self.current_location.clone(),
        })
    }

    /// .S: Print the entire stack without modifying it.
    /// ( -- )
    pub fn print_stack(&mut self) -> ForthResult<()> {
        print!("<{}> ", self.stack.len());
        for val in &self.stack {
            print!("{} ", val);
        }
        io::stdout().flush().map_err(|e| ForthError::IOError {
            message: e.to_string(),
            location: self.current_location.clone(),
        })
    }

    /// EMIT: Print a character.
    /// ( c -- )
    pub fn emit(&mut self) -> ForthResult<()> {
        let c = self.pop()?;
        if let Some(ch) = char::from_u32(c as u32) {
            print!("{}", ch);
            io::stdout().flush().map_err(|e| ForthError::IOError {
                message: e.to_string(),
                location: self.current_location.clone(),
            })
        } else {
            Err(ForthError::RuntimeError {
                message: format!("Invalid character code: {}", c),
                location: self.current_location.clone(),
            })
        }
    }

    /// CR: Print a newline.
    /// ( -- )
    pub fn cr(&mut self) -> ForthResult<()> {
        println!();
        Ok(())
    }

    /// SPACE: Print a space.
    /// ( -- )
    pub fn space(&mut self) -> ForthResult<()> {
        print!(" ");
        io::stdout().flush().map_err(|e| ForthError::IOError {
            message: e.to_string(),
            location: self.current_location.clone(),
        })
    }

    /// SPACES: Print n spaces.
    /// ( n -- )
    pub fn spaces(&mut self) -> ForthResult<()> {
        let n = self.pop()?;
        for _ in 0..n {
            print!(" ");
        }
        io::stdout().flush().map_err(|e| ForthError::IOError {
            message: e.to_string(),
            location: self.current_location.clone(),
        })
    }

    /// TYPE: Print a string (address and length on stack - simplified for REPL).
    /// In our implementation, we'll use a different approach with print_string.
    pub fn print_string(&mut self, s: &str) -> ForthResult<()> {
        print!("{}", s);
        io::stdout().flush().map_err(|e| ForthError::IOError {
            message: e.to_string(),
            location: self.current_location.clone(),
        })
    }

    /// KEY: Read a single character.
    /// ( -- c )
    pub fn key(&mut self) -> ForthResult<()> {
        let mut buf = [0u8; 1];
        io::stdin()
            .read_exact(&mut buf)
            .map_err(|e| ForthError::IOError {
                message: e.to_string(),
                location: self.current_location.clone(),
            })?;
        self.push(buf[0] as i64)
    }

    // =========================================================================
    // Memory Operations
    // =========================================================================

    /// !: Store value to variable.
    /// ( value -- ) with variable name
    pub fn store(&mut self, var: &str) -> ForthResult<()> {
        let value = self.pop()?;
        self.memory.insert(var.to_string(), value);
        Ok(())
    }

    /// @: Fetch value from variable.
    /// ( -- value ) with variable name
    pub fn fetch(&mut self, var: &str) -> ForthResult<()> {
        let value =
            self.memory
                .get(var)
                .copied()
                .ok_or_else(|| ForthError::InvalidMemoryAccess {
                    variable: var.to_string(),
                    location: self.current_location.clone(),
                })?;
        self.push(value)
    }

    /// +!: Add to variable.
    /// ( n -- ) with variable name
    pub fn add_store(&mut self, var: &str) -> ForthResult<()> {
        let n = self.pop()?;
        let current =
            self.memory
                .get(var)
                .copied()
                .ok_or_else(|| ForthError::InvalidMemoryAccess {
                    variable: var.to_string(),
                    location: self.current_location.clone(),
                })?;
        self.memory.insert(var.to_string(), current + n);
        Ok(())
    }

    // =========================================================================
    // Return Stack Operations
    // =========================================================================

    /// >R: Move top of data stack to return stack.
    /// ( a -- ) R:( -- a )
    pub fn to_r(&mut self) -> ForthResult<()> {
        let a = self.pop()?;
        self.rstack.push(a);
        Ok(())
    }

    /// R>: Move top of return stack to data stack.
    /// ( -- a ) R:( a -- )
    pub fn from_r(&mut self) -> ForthResult<()> {
        let a = self
            .rstack
            .pop()
            .ok_or_else(|| ForthError::ReturnStackUnderflow {
                location: self.current_location.clone(),
            })?;
        self.push(a)
    }

    /// R@: Copy top of return stack to data stack.
    /// ( -- a ) R:( a -- a )
    pub fn r_fetch(&mut self) -> ForthResult<()> {
        let a = *self
            .rstack
            .last()
            .ok_or_else(|| ForthError::ReturnStackUnderflow {
                location: self.current_location.clone(),
            })?;
        self.push(a)
    }

    /// I: Copy innermost loop index to data stack (alias for R@).
    /// ( -- index )
    pub fn loop_i(&mut self) -> ForthResult<()> {
        self.r_fetch()
    }

    /// J: Copy second loop index to data stack.
    /// ( -- index )
    pub fn loop_j(&mut self) -> ForthResult<()> {
        let len = self.rstack.len();
        if len < 2 {
            return Err(ForthError::ReturnStackUnderflow {
                location: self.current_location.clone(),
            });
        }
        let j = self.rstack[len - 2];
        self.push(j)
    }

    // =========================================================================
    // Word Invocation
    // =========================================================================

    /// Call a user-defined word by name.
    pub fn call_word(&mut self, name: &str) -> ForthResult<()> {
        let func = self
            .words
            .get(name)
            .copied()
            .ok_or_else(|| ForthError::UndefinedWord {
                name: name.to_string(),
                location: self.current_location.clone(),
            })?;

        // Save current location and set word as current
        let prev_location = self.current_location.clone();
        self.set_current_word(name);

        // Call the word
        let result = func(self);

        // Restore previous location
        self.current_location = prev_location;

        result
    }
}

use std::io::Read;
