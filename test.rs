// Generated from optimized IR
use std::collections::HashMap;

pub struct OptimizedForth {
    stack: Vec<i32>,
    words: HashMap<String, Vec<String>>,
    loop_stack: Vec<(i32, i32)>, // (index, limit) pairs
}

impl OptimizedForth {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            words: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    // Function: ABS-SIMPLE (consumes: 0, produces: 0)
    fn abs_simple(&mut self) -> Result<(), String> {
        // Definition: ABS-SIMPLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Push constant 0
        self.stack.push(0);
        // Less than comparison
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b { -1 } else { 0 }); }
        // Unknown word: IF
        // Negate
        { let a = self.stack.pop().unwrap(); self.stack.push(-a); }
        // Unknown word: THEN
        return Ok(());
    }

    // Function: SUM-OF-SQUARES (consumes: 0, produces: 0)
    fn sum_of_squares(&mut self) -> Result<(), String> {
        // Definition: SUM-OF-SQUARES
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Multiplication
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
        // Swap top two stack items
        { let len = self.stack.len(); self.stack.swap(len-1, len-2); }
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Multiplication
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        return Ok(());
    }

    // Function: DOUBLE (consumes: 0, produces: 0)
    fn double(&mut self) -> Result<(), String> {
        // Definition: DOUBLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        return Ok(());
    }

    // Function: TRIPLE (consumes: 0, produces: 0)
    fn triple(&mut self) -> Result<(), String> {
        // Definition: TRIPLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        return Ok(());
    }

    pub fn execute(&mut self) -> Result<(), String> {
        // Push constant 5
        self.stack.push(6);
        // Push constant 1
        // Addition
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 5
        self.stack.push(4);
        // Push constant 1
        // Subtraction
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 0
        self.stack.push(-1);
        // Push constant 0
        // Equal comparison
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 5
        self.stack.push(0);
        // Push constant 0
        // Equal comparison
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant -3
        self.stack.push(-1);
        // Push constant 0
        // Less than comparison
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 7
        self.stack.push(-1);
        // Push constant 0
        // Greater than comparison
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 5
        self.stack.push(5);
        // Push constant 0
        self.stack.push(0);
        // DO loop
        // ?DO: setup loop
        {
            let start = self.stack.pop().unwrap();
            let limit = self.stack.pop().unwrap();
            if start < limit {
                for loop_index in start..limit {
                    self.loop_stack.push((loop_index, limit));
                    // Loop index I
                    // I: push loop index
                    if let Some((index, _)) = self.loop_stack.last() { self.stack.push(*index); } else { self.stack.push(0); }
                    // Print top of stack
                    print!("{}", self.stack.pop().unwrap());
                    // LOOP
                    self.loop_stack.pop();
                }
            }
        }
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 5
        self.stack.push(5);
        // Call user-defined word: DOUBLE
        // Definition: DOUBLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 4
        self.stack.push(4);
        // Call user-defined word: TRIPLE
        // Definition: TRIPLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 1
        self.stack.push(1);
        // Push constant 2
        self.stack.push(2);
        // Push constant 3
        self.stack.push(3);
        // Print entire stack
        println!("<{}> {:?}", self.stack.len(), self.stack);
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 6
        self.stack.push(6);
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Multiplication
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 3
        self.stack.push(35);
        // Push constant 4
        // Addition
        // Push constant 5
        // Multiplication
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant 3
        self.stack.push(3);
        // Push constant 4
        self.stack.push(4);
        // Call user-defined word: SUM-OF-SQUARES
        // Definition: SUM-OF-SQUARES
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Multiplication
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
        // Swap top two stack items
        { let len = self.stack.len(); self.stack.swap(len-1, len-2); }
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Multiplication
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        // Push constant -5
        self.stack.push(-5);
        // Call user-defined word: ABS-SIMPLE
        // Definition: ABS-SIMPLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Push constant 0
        self.stack.push(0);
        // Less than comparison
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b { -1 } else { 0 }); }
        // Unknown word: IF
        // Negate
        { let a = self.stack.pop().unwrap(); self.stack.push(-a); }
        // Unknown word: THEN
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Push constant 7
        self.stack.push(7);
        // Call user-defined word: ABS-SIMPLE
        // Definition: ABS-SIMPLE
        // Duplicate top of stack
        { let top = *self.stack.last().unwrap(); self.stack.push(top); }
        // Push constant 0
        self.stack.push(0);
        // Less than comparison
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b { -1 } else { 0 }); }
        // Unknown word: IF
        // Negate
        { let a = self.stack.pop().unwrap(); self.stack.push(-a); }
        // Unknown word: THEN
        // Print top of stack
        print!("{}", self.stack.pop().unwrap());
        // Print newline
        self.stack.push(10);
        print!("{}", char::from(self.stack.pop().unwrap() as u8));
        Ok(())
    }
}

fn main() {
    let mut forth = OptimizedForth::new();
    match forth.execute() {
        Ok(()) => {},
        Err(e) => eprintln!("Error: {}", e),
    }
}
