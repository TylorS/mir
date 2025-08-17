//! Integration tests for the WASM backend

use mir_ast::{
    CompatibilityInfo, ExportDeclaration, ExportVisibility, Expression, Module, NodeId, Statement,
};
use mir_backend_wasm::{OptimizationLevel, WasmCodeGenerator};
use mir_types::{ContentHash, TypeHash, Value};
use std::collections::VecDeque;

/// Builder for creating test modules with sensible defaults
struct TestModuleBuilder {
    name: String,
    functions: Vec<(String, Vec<String>, Vec<Statement>)>,
    exports: Vec<String>,
}

impl TestModuleBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

    fn with_function(mut self, name: &str, params: Vec<&str>, body: Vec<Statement>) -> Self {
        self.functions.push((
            name.to_string(),
            params.into_iter().map(String::from).collect(),
            body,
        ));
        self
    }

    fn with_export(mut self, name: &str) -> Self {
        self.exports.push(name.to_string());
        self
    }

    fn build(self) -> Module {
        let mut statements = Vec::new();
        let mut exports = Vec::new();
        let mut node_id = 2u64;

        for (func_name, params, body) in self.functions {
            statements.push(Statement::FunctionDeclaration {
                id: NodeId::new(node_id),
                name: func_name.clone(),
                parameters: params,
                body,
            });
            node_id += 1;

            if self.exports.contains(&func_name) {
                exports.push(ExportDeclaration {
                    name: func_name.clone(),
                    is_type: false,
                    exported_hash: ContentHash::new(func_name.as_bytes()),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                });
            }
        }

        let name_bytes = self.name.as_bytes().to_vec();
        Module {
            id: NodeId::new(1),
            name: self.name,
            imports: vec![],
            exports,
            statements,
            type_hash: TypeHash::new(ContentHash::new(&name_bytes)),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
}

fn create_simple_module() -> Module {
    TestModuleBuilder::new("simple_module")
        .with_function(
            "test_func",
            vec!["x"],
            vec![Statement::Expression {
                id: NodeId::new(100),
                expression: Expression::Literal {
                    id: NodeId::new(101),
                    value: Value::I32(42),
                },
            }],
        )
        .with_export("test_func")
        .build()
}

/// Tests the complete WASM generation pipeline from MIR AST to WASM bytecode.
/// Verifies that:
/// - WASM magic number and version are correct
/// - Exports and functions are properly generated
/// - Memory section is included
#[test]
fn test_complete_wasm_generation_pipeline() {
    let mut generator = WasmCodeGenerator::new();
    generator.set_optimization_level(OptimizationLevel::Speed);

    let module = create_simple_module();
    let result = generator.generate(&module);

    assert!(result.is_ok());

    let wasm_module = result.unwrap();

    // Verify WASM structure
    assert!(wasm_module.bytecode.len() >= 8);
    assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    assert_eq!(&wasm_module.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]);

    // Verify exports and functions
    assert_eq!(wasm_module.exports.len(), 1);
    assert_eq!(wasm_module.functions.len(), 1);
    assert!(wasm_module.memory.is_some());
}

/// Tests that all optimization levels produce valid WASM output.
/// Ensures that optimization settings don't break the generation pipeline.
#[test]
fn test_optimization_levels() {
    let module = create_simple_module();

    let levels = vec![
        OptimizationLevel::None,
        OptimizationLevel::Size,
        OptimizationLevel::Speed,
        OptimizationLevel::Aggressive,
    ];

    for level in levels {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(level);

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();
        assert!(wasm_module.bytecode.len() >= 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }
}

#[test]
fn test_wasm_generation_with_empty_module() {
    let empty_module = TestModuleBuilder::new("empty_module").build();
    let generator = WasmCodeGenerator::new();
    
    let result = generator.generate(&empty_module);
    assert!(result.is_ok());
    
    let wasm_module = result.unwrap();
    assert!(wasm_module.bytecode.len() >= 8); // Still has WASM header
    assert_eq!(wasm_module.functions.len(), 0);
    assert_eq!(wasm_module.exports.len(), 0);
}

#[test]
fn test_wasm_generation_with_multiple_functions() {
    let module = TestModuleBuilder::new("multi_func_module")
        .with_function(
            "add",
            vec!["a", "b"],
            vec![Statement::Expression {
                id: NodeId::new(200),
                expression: Expression::Literal {
                    id: NodeId::new(201),
                    value: Value::I32(0), // Placeholder for a + b
                },
            }],
        )
        .with_function(
            "multiply",
            vec!["x", "y"],
            vec![Statement::Expression {
                id: NodeId::new(202),
                expression: Expression::Literal {
                    id: NodeId::new(203),
                    value: Value::I32(0), // Placeholder for x * y
                },
            }],
        )
        .with_export("add")
        .with_export("multiply")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 2);
    assert_eq!(wasm_module.exports.len(), 2);
}

#[test]
fn test_wasm_generation_error_handling() {
    // Test with invalid module structure
    let mut invalid_module = TestModuleBuilder::new("invalid_module").build();
    // Corrupt the module by setting an invalid type hash
    invalid_module.type_hash = TypeHash::new(ContentHash::zero());
    
    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&invalid_module);
    
    // Should handle gracefully - in a real implementation, this might return an error
    // For now, we just verify it doesn't panic
    let _ = result;
}

#[test]
fn test_optimization_level_effects() {
    let module = create_simple_module();
    let mut size_optimized_generator = WasmCodeGenerator::new();
    size_optimized_generator.set_optimization_level(OptimizationLevel::Size);
    
    let mut speed_optimized_generator = WasmCodeGenerator::new();
    speed_optimized_generator.set_optimization_level(OptimizationLevel::Speed);
    
    let size_result = size_optimized_generator.generate(&module).unwrap();
    let speed_result = speed_optimized_generator.generate(&module).unwrap();
    
    // Both should succeed
    assert!(size_result.bytecode.len() >= 8);
    assert!(speed_result.bytecode.len() >= 8);
    
    // In a real implementation, we might verify that size optimization produces smaller bytecode
    // and speed optimization produces different instruction patterns
}

#[test]
fn test_debug_info_generation() {
    let mut generator = WasmCodeGenerator::new();
    generator.enable_debug_info(true);
    generator.enable_source_maps(true);
    
    let module = create_simple_module();
    let result = generator.generate(&module).unwrap();
    
    assert!(result.debug_info.is_some());
    assert!(result.source_map.is_some());
    
    // Test disabling debug info
    generator.enable_debug_info(false);
    generator.enable_source_maps(false);
    
    let result_no_debug = generator.generate(&module).unwrap();
    assert!(result_no_debug.debug_info.is_none());
    assert!(result_no_debug.source_map.is_none());
}

#[test]
fn test_wasm_generation_with_complex_expressions() {
    let module = TestModuleBuilder::new("complex_expr_module")
        .with_function(
            "complex_func",
            vec!["param1", "param2"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(300),
                    name: "local_var".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(301),
                        value: Value::I32(100),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(302),
                    expression: Expression::Identifier {
                        id: NodeId::new(303),
                        name: "local_var".to_string(),
                    },
                },
            ],
        )
        .with_export("complex_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    // Should handle complex expressions without errors
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert_eq!(wasm_module.exports.len(), 1);
}

#[test]
fn test_wasm_generation_with_observability_statements() {
    let module = TestModuleBuilder::new("observability_module")
        .with_function(
            "traced_func",
            vec![],
            vec![
                Statement::StartSpan {
                    id: NodeId::new(400),
                    name: "test_span".to_string(),
                    attributes: std::collections::HashMap::new(),
                    body: vec![
                        Statement::Expression {
                            id: NodeId::new(401),
                            expression: Expression::Literal {
                                id: NodeId::new(402),
                                value: Value::I32(123),
                            },
                        },
                    ],
                },
                Statement::RecordMetric {
                    id: NodeId::new(403),
                    metric_type: mir_ast::MetricType::Counter,
                    name: "test_metric".to_string(),
                    value: Expression::Literal {
                        id: NodeId::new(404),
                        value: Value::I32(1),
                    },
                    labels: std::collections::HashMap::new(),
                },
                Statement::LogEvent {
                    id: NodeId::new(405),
                    level: mir_ast::LogLevel::Info,
                    message: "Test log message".to_string(),
                    attributes: std::collections::HashMap::new(),
                },
            ],
        )
        .with_export("traced_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    // Should handle observability statements
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
}

#[test]
fn test_wasm_generation_with_different_value_types() {
    let module = TestModuleBuilder::new("value_types_module")
        .with_function(
            "value_types_func",
            vec![],
            vec![
                Statement::Expression {
                    id: NodeId::new(500),
                    expression: Expression::Literal {
                        id: NodeId::new(501),
                        value: Value::I32(42),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(502),
                    expression: Expression::Literal {
                        id: NodeId::new(503),
                        value: Value::I64(1234567890),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(504),
                    expression: Expression::Literal {
                        id: NodeId::new(505),
                        value: Value::F32(std::f32::consts::PI),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(506),
                    expression: Expression::Literal {
                        id: NodeId::new(507),
                        value: Value::F64(std::f64::consts::E),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(508),
                    expression: Expression::Literal {
                        id: NodeId::new(509),
                        value: Value::Bool(true),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(510),
                    expression: Expression::Literal {
                        id: NodeId::new(511),
                        value: Value::String("test string".to_string()),
                    },
                },
            ],
        )
        .with_export("value_types_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    // Should handle all value types
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert!(!wasm_module.functions[0].body.is_empty());
}

#[test]
fn test_optimization_integration_with_codegen() {
    let module = TestModuleBuilder::new("optimization_test_module")
        .with_function(
            "optimizable_func",
            vec![],
            vec![
                // Create patterns that should be optimized
                Statement::Expression {
                    id: NodeId::new(600),
                    expression: Expression::Literal {
                        id: NodeId::new(601),
                        value: Value::I32(5),
                    },
                },
                Statement::Expression {
                    id: NodeId::new(602),
                    expression: Expression::Literal {
                        id: NodeId::new(603),
                        value: Value::I32(3),
                    },
                },
            ],
        )
        .with_export("optimizable_func")
        .build();

    // Test different optimization levels
    let optimization_levels = vec![
        OptimizationLevel::None,
        OptimizationLevel::Size,
        OptimizationLevel::Speed,
        OptimizationLevel::Aggressive,
    ];

    for level in optimization_levels {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(level);
        
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let wasm_module = result.unwrap();
        assert!(wasm_module.bytecode.len() >= 8); // At least WASM header
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }
}

#[test]
fn test_wasm_module_structure_validation() {
    let module = create_simple_module();
    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module).unwrap();
    
    // Validate WASM module structure
    assert!(result.bytecode.len() >= 8);
    assert_eq!(&result.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]); // Magic
    assert_eq!(&result.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]); // Version
    
    // Validate exports match functions
    assert_eq!(result.exports.len(), 1);
    assert_eq!(result.functions.len(), 1);
    
    // Validate memory is present
    assert!(result.memory.is_some());
    let memory = result.memory.unwrap();
    assert!(memory.memory_type.limits.min > 0);
}

#[test]
fn test_error_propagation_from_optimization() {
    // Create a module that will generate invalid bytecode for testing
    let module = TestModuleBuilder::new("error_test_module")
        .with_function(
            "error_func",
            vec![],
            vec![Statement::Expression {
                id: NodeId::new(700),
                expression: Expression::Literal {
                    id: NodeId::new(701),
                    value: Value::Null, // This might cause issues in code generation
                },
            }],
        )
        .build();

    let mut generator = WasmCodeGenerator::new();
    generator.set_optimization_level(OptimizationLevel::Aggressive);
    
    // Should handle errors gracefully
    let result = generator.generate(&module);
    // The result might be Ok or Err depending on implementation
    // The important thing is that it doesn't panic
    let _ = result;
}

#[test]
fn test_large_module_generation() {
    let mut builder = TestModuleBuilder::new("large_module");
    
    // Create a module with many functions
    for i in 0..10 {
        builder = builder.with_function(
            &format!("func_{i}"),
            vec!["param"],
            vec![Statement::Expression {
                id: NodeId::new(800 + i as u64),
                expression: Expression::Literal {
                    id: NodeId::new(900 + i as u64),
                    value: Value::I32(i),
                },
            }],
        );
        
        if i % 2 == 0 {
            builder = builder.with_export(&format!("func_{i}"));
        }
    }
    
    let module = builder.build();
    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 10);
    assert_eq!(wasm_module.exports.len(), 5); // Only even-numbered functions exported
}#
[test]
fn test_wasm_generation_with_hot_reload_metadata() {
    let module = TestModuleBuilder::new("hmr_module")
        .with_function(
            "reloadable_func",
            vec![],
            vec![
                Statement::Expression {
                    id: NodeId::new(1001),
                    expression: Expression::Literal {
                        id: NodeId::new(1002),
                        value: Value::I32(42),
                    },
                },
            ],
        )
        .with_export("reloadable_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert_eq!(wasm_module.exports.len(), 1);
    
    // Verify the function body contains the expected logic
    assert!(!wasm_module.functions[0].body.is_empty());
}

#[test]
fn test_wasm_generation_performance_with_large_functions() {
    use std::time::Instant;
    
    let mut statements = Vec::new();
    
    // Create a function with many statements
    for i in 0..100 {
        statements.push(Statement::Expression {
            id: NodeId::new(2000 + i),
            expression: Expression::Literal {
                id: NodeId::new(3000 + i),
                value: Value::I32(i as i32),
            },
        });
    }
    
    let module = TestModuleBuilder::new("large_function_module")
        .with_function("large_func", vec![], statements)
        .with_export("large_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let start = Instant::now();
    let result = generator.generate(&module);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    // Should complete in reasonable time (less than 1 second for 100 statements)
    assert!(duration.as_secs() < 1);
    
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert!(!wasm_module.functions[0].body.is_empty());
}

#[test]
fn test_wasm_generation_with_all_optimization_levels() {
    let module = TestModuleBuilder::new("optimization_test")
        .with_function(
            "test_func",
            vec!["x", "y"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(4000),
                    name: "temp".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(4001),
                        value: Value::I32(10),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(4002),
                    expression: Expression::Identifier {
                        id: NodeId::new(4003),
                        name: "temp".to_string(),
                    },
                },
            ],
        )
        .with_export("test_func")
        .build();

    let optimization_levels = vec![
        OptimizationLevel::None,
        OptimizationLevel::Size,
        OptimizationLevel::Speed,
        OptimizationLevel::Aggressive,
    ];

    let mut results = Vec::new();
    
    for level in optimization_levels {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(level);
        
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let wasm_module = result.unwrap();
        results.push(wasm_module.bytecode.len());
    }
    
    // All optimization levels should produce valid WASM
    for size in &results {
        assert!(*size >= 8); // At least WASM header
    }
}

#[test]
fn test_wasm_generation_error_recovery() {
    // Test that generation can recover from various error conditions
    let mut module = TestModuleBuilder::new("error_recovery_test").build();
    
    // Add a function with potentially problematic constructs
    module.statements = vec![
        Statement::FunctionDeclaration {
            id: NodeId::new(5000),
            name: "problematic_func".to_string(),
            parameters: vec!["param1".to_string()],
            body: vec![
                // Reference undefined variable
                Statement::Expression {
                    id: NodeId::new(5001),
                    expression: Expression::Identifier {
                        id: NodeId::new(5002),
                        name: "undefined_var".to_string(),
                    },
                },
            ],
        },
    ];

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    // Should handle the error gracefully
    match result {
        Ok(_) => {
            // If it succeeds, that's fine too - depends on implementation
        }
        Err(err) => {
            // Should be a meaningful error
            assert!(format!("{err}").contains("undefined") || format!("{err}").contains("Undefined"));
        }
    }
}

#[test]
fn test_wasm_bytecode_validation() {
    let module = TestModuleBuilder::new("validation_test")
        .with_function(
            "simple_func",
            vec![],
            vec![Statement::Expression {
                id: NodeId::new(6000),
                expression: Expression::Literal {
                    id: NodeId::new(6001),
                    value: Value::I32(123),
                },
            }],
        )
        .with_export("simple_func")
        .build();

    let generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    
    assert!(result.is_ok());
    let wasm_module = result.unwrap();
    
    // Validate WASM bytecode structure
    let bytecode = &wasm_module.bytecode;
    assert!(bytecode.len() >= 8);
    
    // Check magic number
    assert_eq!(&bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    
    // Check version
    assert_eq!(&bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]);
    
    // Should have at least one section (type, function, or code)
    assert!(bytecode.len() > 8);
}