use crate::ir::{IRProgram, IRFunction, IRInstruction, IRValue};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone)]
pub struct CodegenError {
    pub message: String,
    pub location: Option<String>,
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(loc) => write!(f, "Codegen error at {}: {}", loc, self.message),
            None => write!(f, "Codegen error: {}", self.message),
        }
    }
}

impl std::error::Error for CodegenError {}

pub type CodegenResult<T = String> = Result<T, CodegenError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentStyle {
    DoubleSlash,  // //
    Hash,         // #
    CStyle,       // /* */
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SectionType {
    Header,
    Imports,
    Types,
    Constants,
    Functions,
    Main,
    Footer,
}

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub name: String,
    pub architecture: String,
    pub pointer_size: usize,
    pub endianness: String,
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub supports_inline_assembly: bool,
    pub supports_tail_calls: bool,
    pub supports_computed_goto: bool,
    pub max_stack_size: Option<usize>,
    pub native_types: Vec<NativeType>,
    pub optimization_passes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum NativeType {
    Integer(u8), // bit width
    Float(u8),
    String,
    Array,
    Struct,
}

pub struct CodegenContext {
    pub indent_level: usize,
    pub temp_counter: usize,
    pub label_counter: usize,
    pub target: TargetInfo,
    pub optimization_level: OptLevel,
    pub sections: HashMap<SectionType, Vec<String>>,
    pub dependencies: HashSet<String>,
    pub emit_debug_info: bool,
    pub emit_profiling: bool,
}

impl CodegenContext {
    pub fn new(target: TargetInfo) -> Self {
        Self {
            indent_level: 0,
            temp_counter: 0,
            label_counter: 0,
            target,
            optimization_level: OptLevel::Basic,
            sections: HashMap::new(),
            dependencies: HashSet::new(),
            emit_debug_info: false,
            emit_profiling: false,
        }
    }

    pub fn next_temp(&mut self) -> String {
        let temp = format!("temp_{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    pub fn next_label(&mut self) -> String {
        let label = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn add_dependency(&mut self, dep: String) {
        self.dependencies.insert(dep);
    }

    pub fn add_to_section(&mut self, section: SectionType, content: String) {
        self.sections.entry(section).or_insert_with(Vec::new).push(content);
    }
}

pub trait Backend {
    fn name(&self) -> &str;
    fn generate_program(&mut self, ir: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult;
    fn capabilities(&self) -> BackendCapabilities;
}

pub trait CodeEmitter {
    fn emit_line(&mut self, line: &str);
    fn emit_block<F>(&mut self, header: &str, body: F) where F: FnOnce(&mut Self);
    fn emit_function(&mut self, name: &str, params: &[&str], body: &str);
    fn emit_comment(&mut self, text: &str);
    fn push_indent(&mut self);
    fn pop_indent(&mut self);
    fn get_output(&self) -> String;
    fn clear(&mut self);
}

pub trait TargetLanguage {
    fn file_extension(&self) -> &str;
    fn comment_style(&self) -> CommentStyle;
    fn compile_command(&self, filename: &str) -> String;
    fn runtime_requirements(&self) -> Vec<String>;
    fn emit_header(&self, ctx: &CodegenContext) -> String;
    fn emit_footer(&self, ctx: &CodegenContext) -> String;
}

pub trait IRTranslator {
    fn translate_instruction(&mut self, instr: &IRInstruction, ctx: &mut CodegenContext) -> CodegenResult;
    fn translate_function(&mut self, func: &IRFunction, ctx: &mut CodegenContext) -> CodegenResult;
    fn translate_program(&mut self, program: &IRProgram, ctx: &mut CodegenContext) -> CodegenResult;
}

pub struct TemplateArgs {
    pub args: HashMap<String, String>,
}

impl TemplateArgs {
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }

    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.args.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.args.get(key)
    }
}

pub trait CodeTemplate {
    fn render(&self, ctx: &CodegenContext, args: &TemplateArgs) -> String;
}

pub struct TemplateRegistry {
    templates: HashMap<String, Box<dyn CodeTemplate>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register<T: CodeTemplate + 'static>(&mut self, name: String, template: T) {
        self.templates.insert(name, Box::new(template));
    }

    pub fn render(&self, name: &str, ctx: &CodegenContext, args: &TemplateArgs) -> Option<String> {
        self.templates.get(name).map(|t| t.render(ctx, args))
    }
}