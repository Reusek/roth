use crate::codegen::framework::{CodeEmitter, CommentStyle};

pub struct BaseEmitter {
    output: String,
    indent_level: usize,
    indent_string: String,
    comment_style: CommentStyle,
}

impl BaseEmitter {
    pub fn new(comment_style: CommentStyle) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_string: "    ".to_string(),
            comment_style,
        }
    }

    pub fn with_indent_string(mut self, indent: String) -> Self {
        self.indent_string = indent;
        self
    }

    fn emit_indent(&self) -> String {
        self.indent_string.repeat(self.indent_level)
    }

    fn format_comment(&self, text: &str) -> String {
        match self.comment_style {
            CommentStyle::DoubleSlash => format!("// {}", text),
            CommentStyle::Hash => format!("# {}", text),
            CommentStyle::CStyle => format!("/* {} */", text),
        }
    }
}

impl CodeEmitter for BaseEmitter {
    fn emit_line(&mut self, line: &str) {
        if line.is_empty() {
            self.output.push('\n');
        } else {
            self.output.push_str(&format!("{}{}\n", self.emit_indent(), line));
        }
    }

    fn emit_block<F>(&mut self, header: &str, body: F) 
    where F: FnOnce(&mut Self) 
    {
        self.emit_line(header);
        self.push_indent();
        body(self);
        self.pop_indent();
    }

    fn emit_function(&mut self, name: &str, params: &[&str], body: &str) {
        let params_str = params.join(", ");
        let header = match self.comment_style {
            CommentStyle::DoubleSlash => format!("fn {}({}) {{", name, params_str),
            CommentStyle::CStyle => format!("void {}({}) {{", name, params_str),
            CommentStyle::Hash => format!("def {}({}):", name, params_str),
        };
        
        self.emit_block(&header, |emitter| {
            for line in body.lines() {
                emitter.emit_line(line);
            }
        });
        
        if self.comment_style != CommentStyle::Hash {
            self.emit_line("}");
        }
    }

    fn emit_comment(&mut self, text: &str) {
        let comment = self.format_comment(text);
        self.emit_line(&comment);
    }

    fn push_indent(&mut self) {
        self.indent_level += 1;
    }

    fn pop_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn get_output(&self) -> String {
        self.output.clone()
    }

    fn clear(&mut self) {
        self.output.clear();
        self.indent_level = 0;
    }
}

pub struct RustEmitter {
    base: BaseEmitter,
}

impl RustEmitter {
    pub fn new() -> Self {
        Self {
            base: BaseEmitter::new(CommentStyle::DoubleSlash),
        }
    }

    pub fn emit_struct(&mut self, name: &str, fields: &[(&str, &str)]) {
        self.base.emit_line(&format!("pub struct {} {{", name));
        self.base.push_indent();
        for (field_name, field_type) in fields {
            self.base.emit_line(&format!("pub {}: {},", field_name, field_type));
        }
        self.base.pop_indent();
        self.base.emit_line("}");
        self.base.emit_line("");
    }

    pub fn emit_impl_block<F>(&mut self, struct_name: &str, body: F)
    where F: FnOnce(&mut Self)
    {
        self.base.emit_line(&format!("impl {} {{", struct_name));
        self.base.push_indent();
        body(self);
        self.base.pop_indent();
        self.base.emit_line("}");
        self.base.emit_line("");
    }

    pub fn emit_use(&mut self, path: &str) {
        self.base.emit_line(&format!("use {};", path));
    }

    pub fn emit_pub_fn(&mut self, name: &str, params: &[(&str, &str)], return_type: Option<&str>, body: &str) {
        let params_str = params.iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect::<Vec<_>>()
            .join(", ");
        
        let signature = match return_type {
            Some(ret) => format!("pub fn {}({}) -> {} {{", name, params_str, ret),
            None => format!("pub fn {}({}) {{", name, params_str),
        };

        self.base.emit_block(&signature, |emitter| {
            for line in body.lines() {
                emitter.emit_line(line);
            }
        });
        self.base.emit_line("}");
        self.base.emit_line("");
    }
}

impl CodeEmitter for RustEmitter {
    fn emit_line(&mut self, line: &str) {
        self.base.emit_line(line);
    }

    fn emit_block<F>(&mut self, header: &str, body: F) 
    where F: FnOnce(&mut Self) 
    {
        self.base.emit_line(header);
        self.base.push_indent();
        body(self);
        self.base.pop_indent();
    }

    fn emit_function(&mut self, name: &str, params: &[&str], body: &str) {
        self.base.emit_function(name, params, body);
    }

    fn emit_comment(&mut self, text: &str) {
        self.base.emit_comment(text);
    }

    fn push_indent(&mut self) {
        self.base.push_indent();
    }

    fn pop_indent(&mut self) {
        self.base.pop_indent();
    }

    fn get_output(&self) -> String {
        self.base.get_output()
    }

    fn clear(&mut self) {
        self.base.clear();
    }
}

pub struct CEmitter {
    base: BaseEmitter,
}

impl CEmitter {
    pub fn new() -> Self {
        Self {
            base: BaseEmitter::new(CommentStyle::CStyle),
        }
    }

    pub fn emit_include(&mut self, header: &str) {
        self.base.emit_line(&format!("#include <{}>", header));
    }

    pub fn emit_define(&mut self, name: &str, value: &str) {
        self.base.emit_line(&format!("#define {} {}", name, value));
    }

    pub fn emit_struct(&mut self, name: &str, fields: &[(&str, &str)]) {
        self.base.emit_line(&format!("typedef struct {{"));
        self.base.push_indent();
        for (field_name, field_type) in fields {
            self.base.emit_line(&format!("{} {};", field_type, field_name));
        }
        self.base.pop_indent();
        self.base.emit_line(&format!("}} {};", name));
        self.base.emit_line("");
    }

    pub fn emit_c_function(&mut self, return_type: &str, name: &str, params: &[(&str, &str)], body: &str) {
        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params.iter()
                .map(|(name, ty)| format!("{} {}", ty, name))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let signature = format!("{} {}({}) {{", return_type, name, params_str);
        
        self.base.emit_block(&signature, |emitter| {
            for line in body.lines() {
                emitter.emit_line(line);
            }
        });
        self.base.emit_line("}");
        self.base.emit_line("");
    }
}

impl CodeEmitter for CEmitter {
    fn emit_line(&mut self, line: &str) {
        self.base.emit_line(line);
    }

    fn emit_block<F>(&mut self, header: &str, body: F) 
    where F: FnOnce(&mut Self) 
    {
        self.base.emit_line(header);
        self.base.push_indent();
        body(self);
        self.base.pop_indent();
    }

    fn emit_function(&mut self, name: &str, params: &[&str], body: &str) {
        self.emit_c_function("void", name, &params.iter().map(|p| (*p, "int")).collect::<Vec<_>>(), body);
    }

    fn emit_comment(&mut self, text: &str) {
        self.base.emit_comment(text);
    }

    fn push_indent(&mut self) {
        self.base.push_indent();
    }

    fn pop_indent(&mut self) {
        self.base.pop_indent();
    }

    fn get_output(&self) -> String {
        self.base.get_output()
    }

    fn clear(&mut self) {
        self.base.clear();
    }
}