pub struct ForthHighlighter;

impl ForthHighlighter {
    pub fn new() -> Self {
        ForthHighlighter
    }

    pub fn highlight(&self, text: &str) -> String {
        text.to_string()
    }
}

impl Default for ForthHighlighter {
    fn default() -> Self {
        Self::new()
    }
}