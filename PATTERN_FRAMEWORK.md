# Pattern-Based Code Generation Framework

This document describes the new pattern-based code generation framework for the Roth Forth interpreter.

## Overview

The pattern framework allows you to define code generation rules using patterns that match AST nodes and templates that generate the corresponding output code. This makes it easy to create new code generators for different target languages without writing complex recursive traversal code.

## Architecture

### Core Components

1. **Pattern** - Defines what AST structures to match
2. **Template** - Defines how to generate code for matched patterns
3. **Rule** - Combines a pattern with a template and priority
4. **PatternMatcher** - Manages rules and performs pattern matching
5. **CodeGenerator** - Uses PatternMatcher to generate code

### Pattern Types

```rust
pub enum Pattern {
    Exact(AstNode),                                    // Match exact AST node
    AnyNumber,                                         // Match any number
    AnyWord,                                           // Match any word
    AnyDefinition,                                     // Match any definition
    Sequence(Vec<Pattern>),                            // Match sequence of patterns
    Optional(Box<Pattern>),                            // Optional pattern
    Repeat(Box<Pattern>),                              // Repeated pattern
    Named(String, Box<Pattern>),                       // Capture matched value
    Guard(Box<Pattern>, Box<dyn Fn(&AstNode) -> bool>), // Pattern with condition
}
```

### Template Parts

```rust
pub enum TemplatePart {
    Literal(String),                                   // Static text
    Variable(String),                                  // Insert captured variable
    Block(String, Vec<TemplatePart>),                  // Conditional block
    Conditional(String, Vec<TemplatePart>, Option<Vec<TemplatePart>>), // If-else
    Loop(String, Vec<TemplatePart>),                   // Loop over collection
    Indent,                                            // Increase indentation
    Dedent,                                            // Decrease indentation
    NewLine,                                           // New line with indentation
}
```

## Usage Examples

### Creating a Simple Pattern

```rust
use crate::codegen::generator::create_word_pattern;

// Match the "+" word
let pattern = create_word_pattern("+");
```

### Creating a Template

```rust
use crate::codegen::pattern::{Template, TemplatePart};

let template = Template {
    parts: vec![
        TemplatePart::Literal("stack.push(".to_string()),
        TemplatePart::Variable("number".to_string()),
        TemplatePart::Literal(");\n".to_string()),
    ],
};
```

### Creating a Rule

```rust
use crate::codegen::pattern::Rule;

let rule = Rule {
    pattern: create_number_pattern(),
    template: template,
    priority: 10,
};
```

### Adding Rules to a Generator

```rust
impl RustPatternGenerator {
    fn setup_rules(&mut self) {
        // Number pattern
        self.matcher.add_rule(Rule {
            pattern: create_number_pattern(),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("self.stack.push(".to_string()),
                    TemplatePart::Variable("number".to_string()),
                    TemplatePart::Literal(");\n".to_string()),
                ],
            },
            priority: 10,
        });

        // Addition pattern
        self.matcher.add_rule(Rule {
            pattern: create_word_pattern("+"),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("{ let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }\n".to_string()),
                ],
            },
            priority: 50,
        });
    }
}
```

## Creating New Backends

To create a new backend (e.g., JavaScript, Python, Assembly):

1. Create a new file in `src/codegen/` (e.g., `javascript_pattern.rs`)
2. Implement the `CodeGenerator` trait
3. Set up pattern rules in the constructor
4. Add the backend to the `Backend` enum in `mod.rs`
5. Update the factory function

### Example: JavaScript Generator

```rust
use crate::types::AstNode;
use crate::codegen::{CodeGenerator, pattern::{PatternMatcher, Rule, Pattern, Template, TemplatePart}};
use crate::codegen::generator::{create_word_pattern, create_number_pattern};

pub struct JavaScriptPatternGenerator {
    matcher: PatternMatcher,
}

impl JavaScriptPatternGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            matcher: PatternMatcher::new(),
        };
        generator.setup_rules();
        generator
    }

    fn setup_rules(&mut self) {
        // Number pattern
        self.matcher.add_rule(Rule {
            pattern: create_number_pattern(),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("stack.push(".to_string()),
                    TemplatePart::Variable("number".to_string()),
                    TemplatePart::Literal(");\n".to_string()),
                ],
            },
            priority: 10,
        });

        // Addition pattern
        self.matcher.add_rule(Rule {
            pattern: create_word_pattern("+"),
            template: Template {
                parts: vec![
                    TemplatePart::Literal("stack.push(stack.pop() + stack.pop());\n".to_string()),
                ],
            },
            priority: 50,
        });
    }
}

impl CodeGenerator for JavaScriptPatternGenerator {
    fn generate(&mut self, ast: &AstNode) -> String {
        // Implementation here
    }

    fn get_file_extension(&self) -> &str {
        "js"
    }

    fn get_compile_command(&self, filename: &str) -> String {
        format!("node {}", filename)
    }
}
```

## Advanced Features

### Pattern Priorities

Rules with higher priority values are matched first. This allows you to create specific patterns that override more general ones.

```rust
// Specific pattern for "DUP" word (high priority)
Rule { pattern: create_word_pattern("DUP"), template: dup_template, priority: 50 }

// General pattern for any word (low priority)
Rule { pattern: Pattern::AnyWord, template: generic_template, priority: 1 }
```

### Named Captures

Use named patterns to capture matched values:

```rust
Pattern::Named("number".to_string(), Box::new(Pattern::AnyNumber))
```

Then reference in templates:

```rust
TemplatePart::Variable("number".to_string())
```

### Guards

Add conditions to patterns:

```rust
Pattern::Guard(
    Box::new(Pattern::AnyWord),
    Box::new(|node| {
        if let AstNode::Word(w, _) = node {
            w.starts_with("CUSTOM_")
        } else {
            false
        }
    })
)
```

### Complex Templates

Use conditional blocks and loops:

```rust
Template {
    parts: vec![
        TemplatePart::Conditional("has_body".to_string(),
            vec![
                TemplatePart::Loop("body".to_string(), vec![
                    TemplatePart::Variable("item".to_string()),
                    TemplatePart::NewLine,
                ]),
            ],
            Some(vec![
                TemplatePart::Literal("// Empty function".to_string()),
            ])
        ),
    ],
}
```

## Testing

Test your patterns using the REPL:

```bash
cargo run
> gen rust-pattern 5 10 + .
> gen c-pattern 5 10 + .
> gen javascript-pattern 5 10 + .  # If you implement it
```

## Benefits

1. **Declarative** - Define what to match and how to generate, not how to traverse
2. **Extensible** - Easy to add new patterns and templates
3. **Maintainable** - Rules are isolated and can be modified independently
4. **Reusable** - Pattern and template components can be shared across backends
5. **Testable** - Individual patterns and templates can be unit tested

## Files

- `src/codegen/pattern.rs` - Core pattern matching types and logic
- `src/codegen/generator.rs` - Base generator trait and utilities
- `src/codegen/rust_pattern.rs` - Rust code generator using patterns
- `src/codegen/c_pattern.rs` - C code generator using patterns
- `src/codegen/examples.rs` - Example generators for JavaScript, Python, Assembly

## Future Enhancements

1. **Pattern Macros** - Simplify pattern creation with macros
2. **Template Inheritance** - Allow templates to extend base templates
3. **Pattern Composition** - Combine simple patterns into complex ones
4. **Dynamic Rules** - Load rules from configuration files
5. **Pattern Debugging** - Tools to visualize pattern matching process