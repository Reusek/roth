use crate::codegen::framework::{CodeTemplate, CodegenContext, TemplateArgs};
use std::collections::HashMap;

pub struct SimpleTemplate {
    template: String,
}

impl SimpleTemplate {
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }
}

impl CodeTemplate for SimpleTemplate {
    fn render(&self, _ctx: &CodegenContext, args: &TemplateArgs) -> String {
        let mut result = self.template.clone();
        
        for (key, value) in &args.args {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
}

pub struct ConditionalTemplate {
    condition_key: String,
    true_template: String,
    false_template: String,
}

impl ConditionalTemplate {
    pub fn new(condition_key: &str, true_template: &str, false_template: &str) -> Self {
        Self {
            condition_key: condition_key.to_string(),
            true_template: true_template.to_string(),
            false_template: false_template.to_string(),
        }
    }
}

impl CodeTemplate for ConditionalTemplate {
    fn render(&self, ctx: &CodegenContext, args: &TemplateArgs) -> String {
        let condition = args.get(&self.condition_key)
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);
        
        let template = if condition {
            &self.true_template
        } else {
            &self.false_template
        };
        
        let simple = SimpleTemplate::new(template);
        simple.render(ctx, args)
    }
}

pub struct RustTemplates;

impl RustTemplates {
    pub fn create_registry() -> HashMap<String, Box<dyn CodeTemplate>> {
        let mut templates = HashMap::new();
        
        templates.insert(
            "rust.function_header".to_string(),
            Box::new(SimpleTemplate::new("pub fn {name}({params}) -> {return_type} {")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "rust.stack_push".to_string(),
            Box::new(SimpleTemplate::new("self.stack.push({value});")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "rust.stack_pop".to_string(),
            Box::new(SimpleTemplate::new("self.stack.pop().unwrap_or(0)")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "rust.binary_op".to_string(),
            Box::new(SimpleTemplate::new(
                "if self.stack.len() >= 2 {\n    let b = self.stack.pop().unwrap();\n    let a = self.stack.pop().unwrap();\n    self.stack.push(a {op} b);\n}"
            )) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "rust.loop_construct".to_string(),
            Box::new(SimpleTemplate::new("for {var} in {start}..{end} {")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "rust.debug_print".to_string(),
            Box::new(ConditionalTemplate::new(
                "debug",
                "println!(\"[DEBUG] {message}\");",
                "// debug disabled"
            )) as Box<dyn CodeTemplate>
        );
        
        templates
    }
}

pub struct CTemplates;

impl CTemplates {
    pub fn create_registry() -> HashMap<String, Box<dyn CodeTemplate>> {
        let mut templates = HashMap::new();
        
        templates.insert(
            "c.function_header".to_string(),
            Box::new(SimpleTemplate::new("{return_type} {name}({params}) {")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "c.stack_push".to_string(),
            Box::new(SimpleTemplate::new("push(vm, {value});")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "c.stack_pop".to_string(),
            Box::new(SimpleTemplate::new("pop(vm)")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "c.binary_op".to_string(),
            Box::new(SimpleTemplate::new(
                "if (vm->stack.top >= 2) {\n    int b = pop(vm);\n    int a = pop(vm);\n    push(vm, a {op} b);\n}"
            )) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "c.loop_construct".to_string(),
            Box::new(SimpleTemplate::new("for (int {var} = {start}; {var} < {end}; {var}++) {")) as Box<dyn CodeTemplate>
        );
        
        templates.insert(
            "c.debug_print".to_string(),
            Box::new(ConditionalTemplate::new(
                "debug",
                "DEBUG_PRINT(\"{message}\");",
                "/* debug disabled */"
            )) as Box<dyn CodeTemplate>
        );
        
        templates
    }
}

pub struct TemplateEngine {
    templates: HashMap<String, Box<dyn CodeTemplate>>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn with_rust_templates(mut self) -> Self {
        self.templates.extend(RustTemplates::create_registry());
        self
    }

    pub fn with_c_templates(mut self) -> Self {
        self.templates.extend(CTemplates::create_registry());
        self
    }

    pub fn register_template<T: CodeTemplate + 'static>(&mut self, name: String, template: T) {
        self.templates.insert(name, Box::new(template));
    }

    pub fn render(&self, name: &str, ctx: &CodegenContext, args: &TemplateArgs) -> Option<String> {
        self.templates.get(name).map(|t| t.render(ctx, args))
    }

    pub fn render_with_fallback(&self, name: &str, ctx: &CodegenContext, args: &TemplateArgs, fallback: &str) -> String {
        self.render(name, ctx, args).unwrap_or_else(|| fallback.to_string())
    }
}