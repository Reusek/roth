pub struct BaseGenerator {
    output: String,
    indent_level: usize,
}

impl BaseGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.output.clear();
        self.indent_level = 0;
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }
}