use crate::codegen::pattern::{PatternMatcher, Rule, Pattern, Template, TemplatePart};
use crate::codegen::generator::{create_word_pattern, create_number_pattern, create_simple_template, create_complex_template};

pub fn create_example_javascript_generator() -> PatternMatcher {
    let mut matcher = PatternMatcher::new();

    // Number pattern for JavaScript
    matcher.add_rule(Rule {
        pattern: create_number_pattern(),
        template: create_simple_template("stack.push(${number});\n"),
        priority: 10,
    });

    // Addition pattern
    matcher.add_rule(Rule {
        pattern: create_word_pattern("+"),
        template: create_simple_template("stack.push(stack.pop() + stack.pop());\n"),
        priority: 50,
    });

    // Print pattern
    matcher.add_rule(Rule {
        pattern: create_word_pattern("."),
        template: create_simple_template("console.log(stack.pop());\n"),
        priority: 50,
    });

    // Complex template example with conditionals and loops
    matcher.add_rule(Rule {
        pattern: Pattern::Named("definition".to_string(), Box::new(Pattern::AnyDefinition)),
        template: create_complex_template(vec![
            TemplatePart::Literal("// Definition: ".to_string()),
            TemplatePart::Variable("definition".to_string()),
            TemplatePart::NewLine,
            TemplatePart::Literal("function ".to_string()),
            TemplatePart::Variable("definition".to_string()),
            TemplatePart::Literal("() {".to_string()),
            TemplatePart::Indent,
            TemplatePart::NewLine,
            TemplatePart::Conditional("body".to_string(), 
                vec![
                    TemplatePart::Loop("body".to_string(), vec![
                        TemplatePart::Variable("item".to_string()),
                        TemplatePart::NewLine,
                    ]),
                ],
                Some(vec![
                    TemplatePart::Literal("// Empty function".to_string()),
                    TemplatePart::NewLine,
                ])
            ),
            TemplatePart::Dedent,
            TemplatePart::NewLine,
            TemplatePart::Literal("}\n\n".to_string()),
        ]),
        priority: 20,
    });

    matcher
}

pub fn create_example_python_generator() -> PatternMatcher {
    let mut matcher = PatternMatcher::new();

    // Number pattern for Python
    matcher.add_rule(Rule {
        pattern: create_number_pattern(),
        template: create_simple_template("stack.append(${number})\n"),
        priority: 10,
    });

    // Addition pattern
    matcher.add_rule(Rule {
        pattern: create_word_pattern("+"),
        template: create_simple_template("stack.append(stack.pop() + stack.pop())\n"),
        priority: 50,
    });

    // Print pattern
    matcher.add_rule(Rule {
        pattern: create_word_pattern("."),
        template: create_simple_template("print(stack.pop())\n"),
        priority: 50,
    });

    // Definition pattern for Python
    matcher.add_rule(Rule {
        pattern: Pattern::Named("definition".to_string(), Box::new(Pattern::AnyDefinition)),
        template: create_complex_template(vec![
            TemplatePart::Literal("# Definition: ".to_string()),
            TemplatePart::Variable("definition".to_string()),
            TemplatePart::NewLine,
            TemplatePart::Literal("def ".to_string()),
            TemplatePart::Variable("definition".to_string()),
            TemplatePart::Literal("():".to_string()),
            TemplatePart::Indent,
            TemplatePart::NewLine,
            TemplatePart::Block("body".to_string(), vec![
                TemplatePart::Variable("body".to_string()),
            ]),
            TemplatePart::Dedent,
            TemplatePart::NewLine,
            TemplatePart::NewLine,
        ]),
        priority: 20,
    });

    matcher
}

pub fn create_example_assembly_generator() -> PatternMatcher {
    let mut matcher = PatternMatcher::new();

    // Number pattern for x86 assembly
    matcher.add_rule(Rule {
        pattern: create_number_pattern(),
        template: create_complex_template(vec![
            TemplatePart::Literal("    push ".to_string()),
            TemplatePart::Variable("number".to_string()),
            TemplatePart::NewLine,
        ]),
        priority: 10,
    });

    // Addition pattern
    matcher.add_rule(Rule {
        pattern: create_word_pattern("+"),
        template: create_complex_template(vec![
            TemplatePart::Literal("    pop eax".to_string()),
            TemplatePart::NewLine,
            TemplatePart::Literal("    pop ebx".to_string()),
            TemplatePart::NewLine,
            TemplatePart::Literal("    add eax, ebx".to_string()),
            TemplatePart::NewLine,
            TemplatePart::Literal("    push eax".to_string()),
            TemplatePart::NewLine,
        ]),
        priority: 50,
    });

    matcher
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AstNode, Position};

    #[test]
    fn test_javascript_pattern_matching() {
        let matcher = create_example_javascript_generator();
        let nodes = vec![
            AstNode::Number(42, Position { line: 1, column: 1, offset: 0 }),
            AstNode::Number(10, Position { line: 1, column: 4, offset: 3 }),
            AstNode::Word("+".to_string(), Position { line: 1, column: 7, offset: 6 }),
        ];

        // Test number pattern
        let result = matcher.match_pattern(&create_number_pattern(), &nodes, 0);
        assert!(result.matched);
        assert_eq!(result.consumed, 1);

        // Test word pattern
        let result = matcher.match_pattern(&create_word_pattern("+"), &nodes, 2);
        assert!(result.matched);
        assert_eq!(result.consumed, 1);
    }

    #[test]
    fn test_pattern_priority() {
        let mut matcher = PatternMatcher::new();
        
        // Add rules with different priorities
        matcher.add_rule(Rule {
            pattern: create_word_pattern("+"),
            template: create_simple_template("LOW PRIORITY"),
            priority: 1,
        });
        
        matcher.add_rule(Rule {
            pattern: create_word_pattern("+"),
            template: create_simple_template("HIGH PRIORITY"),
            priority: 100,
        });

        let nodes = vec![AstNode::Word("+".to_string(), Position { line: 1, column: 1, offset: 0 })];
        
        if let Some((rule, _)) = matcher.find_matching_rule(&nodes, 0) {
            let output = matcher.render_template(&rule.template, &std::collections::HashMap::new());
            assert_eq!(output, "HIGH PRIORITY");
        } else {
            panic!("No matching rule found");
        }
    }
}