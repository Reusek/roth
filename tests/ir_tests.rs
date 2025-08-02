use roth::ir::{IRFunction, IRInstruction, IRProgram, IRValue, StackEffect};
use std::collections::HashMap;

#[test]
fn test_ir_program_creation() {
    let main_function = IRFunction {
        name: "main".to_string(),
        instructions: vec![
            IRInstruction::Push(IRValue::Constant(42)),
            IRInstruction::Push(IRValue::Constant(2)),
            IRInstruction::Add,
        ],
        stack_effect: StackEffect {
            consumes: 0,
            produces: 1,
        },
    };

    let program = IRProgram {
        functions: HashMap::new(),
        main: main_function,
    };

    assert_eq!(program.main.name, "main");
    assert_eq!(program.main.instructions.len(), 3);
    assert_eq!(program.functions.len(), 0);
}

#[test]
fn test_ir_function_with_stack_operations() {
    let function = IRFunction {
        name: "test_stack".to_string(),
        instructions: vec![
            IRInstruction::Push(IRValue::Constant(10)),
            IRInstruction::Dup,
            IRInstruction::Swap,
            IRInstruction::Over,
            IRInstruction::Drop,
        ],
        stack_effect: StackEffect {
            consumes: 0,
            produces: 2,
        },
    };

    assert_eq!(function.instructions.len(), 5);
    assert_eq!(function.stack_effect.consumes, 0);
    assert_eq!(function.stack_effect.produces, 2);
}

#[test]
fn test_ir_arithmetic_operations() {
    let instructions = vec![
        IRInstruction::Push(IRValue::Constant(10)),
        IRInstruction::Push(IRValue::Constant(5)),
        IRInstruction::Add,
        IRInstruction::Push(IRValue::Constant(2)),
        IRInstruction::Mul,
        IRInstruction::Push(IRValue::Constant(3)),
        IRInstruction::Sub,
        IRInstruction::Push(IRValue::Constant(2)),
        IRInstruction::Div,
    ];

    // This should compute: ((10 + 5) * 2 - 3) / 2 = (30 - 3) / 2 = 13.5 -> 13 (integer division)
    assert_eq!(instructions.len(), 9);

    // Verify instruction types
    match &instructions[2] {
        IRInstruction::Add => {}
        _ => panic!("Expected Add instruction"),
    }
    match &instructions[4] {
        IRInstruction::Mul => {}
        _ => panic!("Expected Mul instruction"),
    }
}

#[test]
fn test_ir_values() {
    let int_val = IRValue::Constant(42);
    let var_val = IRValue::Variable("hello".to_string());
    let stack_val = IRValue::StackTop;

    match int_val {
        IRValue::Constant(42) => {}
        _ => panic!("Expected constant value 42"),
    }

    match var_val {
        IRValue::Variable(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected variable value"),
    }

    match stack_val {
        IRValue::StackTop => {}
        _ => panic!("Expected stack top value"),
    }
}

#[test]
fn test_stack_effect_calculation() {
    let effect1 = StackEffect {
        consumes: 2,
        produces: 1,
    }; // Like ADD: takes 2, produces 1
    let effect2 = StackEffect {
        consumes: 1,
        produces: 2,
    }; // Like DUP: takes 1, produces 2
    let effect3 = StackEffect {
        consumes: 0,
        produces: 1,
    }; // Like literal: takes 0, produces 1

    assert_eq!(effect1.consumes, 2);
    assert_eq!(effect1.produces, 1);

    assert_eq!(effect2.consumes, 1);
    assert_eq!(effect2.produces, 2);

    assert_eq!(effect3.consumes, 0);
    assert_eq!(effect3.produces, 1);
}

#[test]
fn test_ir_program_with_functions() {
    let mut functions = HashMap::new();

    let square_function = IRFunction {
        name: "SQUARE".to_string(),
        instructions: vec![IRInstruction::Dup, IRInstruction::Mul],
        stack_effect: StackEffect {
            consumes: 1,
            produces: 1,
        },
    };

    functions.insert("SQUARE".to_string(), square_function);

    let main_function = IRFunction {
        name: "main".to_string(),
        instructions: vec![
            IRInstruction::Push(IRValue::Constant(5)),
            IRInstruction::Call("SQUARE".to_string()),
        ],
        stack_effect: StackEffect {
            consumes: 0,
            produces: 1,
        },
    };
    let program = IRProgram {
        functions,
        main: main_function,
    };

    assert_eq!(program.functions.len(), 1);
    assert!(program.functions.contains_key("SQUARE"));
    assert_eq!(program.main.instructions.len(), 2);
}

#[test]
fn test_ir_comparison_operations() {
    let instructions = vec![
        IRInstruction::Push(IRValue::Constant(10)),
        IRInstruction::Push(IRValue::Constant(5)),
        IRInstruction::Greater,
        IRInstruction::Push(IRValue::Constant(3)),
        IRInstruction::Push(IRValue::Constant(3)),
        IRInstruction::Equal,
    ];

    assert_eq!(instructions.len(), 6);

    match &instructions[2] {
        IRInstruction::Greater => {}
        _ => panic!("Expected Greater instruction"),
    }

    match &instructions[5] {
        IRInstruction::Equal => {}
        _ => panic!("Expected Equal instruction"),
    }
}

#[test]
fn test_ir_control_flow() {
    use roth::ir::IRLabel;

    let label1 = IRLabel {
        name: "label1".to_string(),
        id: 1,
    };
    let label2 = IRLabel {
        name: "label2".to_string(),
        id: 2,
    };

    let instructions = vec![
        IRInstruction::Push(IRValue::Constant(1)),
        IRInstruction::JumpIf(label1.clone()),
        IRInstruction::Push(IRValue::Constant(1)),
        IRInstruction::Jump(label2.clone()),
        IRInstruction::Push(IRValue::Constant(0)),
    ];

    assert_eq!(instructions.len(), 5);

    match &instructions[1] {
        IRInstruction::JumpIf(label) => assert_eq!(label.id, 1),
        _ => panic!("Expected JumpIf instruction"),
    }

    match &instructions[3] {
        IRInstruction::Jump(label) => assert_eq!(label.id, 2),
        _ => panic!("Expected Jump instruction"),
    }
}

#[test]
fn test_ir_io_operations() {
    let instructions = vec![
        IRInstruction::Push(IRValue::Constant(42)),
        IRInstruction::Print,
        IRInstruction::PrintStack,
        IRInstruction::PrintChar,
    ];

    assert_eq!(instructions.len(), 4);

    match &instructions[1] {
        IRInstruction::Print => {}
        _ => panic!("Expected Print instruction"),
    }

    match &instructions[2] {
        IRInstruction::PrintStack => {}
        _ => panic!("Expected PrintStack instruction"),
    }

    match &instructions[3] {
        IRInstruction::PrintChar => {}
        _ => panic!("Expected PrintChar instruction"),
    }
}

#[test]
fn test_complex_ir_program() {
    use roth::ir::IRLabel;

    let mut functions = HashMap::new();

    let continue_label = IRLabel {
        name: "continue".to_string(),
        id: 1,
    };
    let end_label = IRLabel {
        name: "end".to_string(),
        id: 2,
    };

    // Define a factorial function
    let factorial_function = IRFunction {
        name: "FACTORIAL".to_string(),
        instructions: vec![
            IRInstruction::Dup,                            // Duplicate n
            IRInstruction::Push(IRValue::Constant(1)),     // Push 1
            IRInstruction::Greater,                        // n > 1?
            IRInstruction::JumpIf(continue_label.clone()), // If true, continue
            IRInstruction::Drop,                           // Drop the original n
            IRInstruction::Push(IRValue::Constant(1)),     // Return 1
            IRInstruction::Jump(end_label.clone()),        // Jump to end
            IRInstruction::Label(continue_label),          // Continue label
            IRInstruction::Dup,                            // Duplicate n
            IRInstruction::Push(IRValue::Constant(1)),     // Push 1
            IRInstruction::Sub,                            // n - 1
            IRInstruction::Call("FACTORIAL".to_string()),  // Recursive call
            IRInstruction::Mul,                            // n * factorial(n-1)
            IRInstruction::Label(end_label),               // End label
        ],
        stack_effect: StackEffect {
            consumes: 1,
            produces: 1,
        },
    };

    functions.insert("FACTORIAL".to_string(), factorial_function);

    let main_function = IRFunction {
        name: "main".to_string(),
        instructions: vec![
            IRInstruction::Push(IRValue::Constant(5)),
            IRInstruction::Call("FACTORIAL".to_string()),
            IRInstruction::Print,
        ],
        stack_effect: StackEffect {
            consumes: 0,
            produces: 0,
        },
    };

    let program = IRProgram {
        functions,
        main: main_function,
    };

    assert_eq!(program.functions.len(), 1);
    assert!(program.functions.contains_key("FACTORIAL"));
    assert_eq!(program.main.instructions.len(), 3);
}
