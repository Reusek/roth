// Generated from optimized IR
use std::collections::HashMap;

pub struct OptimizedForth {
    stack: Vec<i32>,
    words: HashMap<String, Vec<String>>,
}

impl OptimizedForth {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            words: HashMap::new(),
        }
    }

    // Function: ADD2 (consumes: 0, produces: 0)
    fn add2(&mut self) -> Result<(), String> {
        // Definition: ADD2
        // Push constant 2
        self.stack.push(2);
        // Addition
        { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
        return Ok(());
        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), String> {
        // Push constant 3
        self.stack.push(3);
        // Call user-defined word: ADD2
        self.add2()?;
        // Print entire stack
        println!("<{}> {:?}", self.stack.len(), self.stack);
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut forth = OptimizedForth::new();
    forth.execute()?;
    Ok(())
}
