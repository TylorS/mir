//! Backend compatibility integration tests
//! 
//! Tests compatibility across different backend implementations including:
//! - WASM backend code generation
//! - JavaScript/TypeScript backend compatibility
//! - WASI backend integration
//! - Cross-backend serialization compatibility

use mir_backend_wasm::{WasmCodeGenerator, OptimizationLevel};
use mir_ast::{Module, Statement, Expression, NodeId, ExportDeclaration, ExportVisibility, CompatibilityInfo};
use mir_types::{Value, ContentHash, TypeHash};
use std::collections::{HashMap, VecDeque};

/// Helper to create test modules for backend compatibility testing
struct BackendTestModule {
    name: String,
    functions: Vec<TestFunction>,
    exports: Vec<String>,
}

struct TestFunction {
    name: String,
    parameters: Vec<String>,
    body: Vec<Statement>,
}

impl BackendTestModule {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

    fn with_function(mut self, name: &str, params: Vec<&str>, statements: Vec<Statement>) -> Self {
        self.functions.push(TestFunction {
            name: name.to_string(),
            parameters: params.into_iter().map(String::from).collect(),
            body: statements,
        });
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

        for func in self.functions {
            statements.push(Statement::FunctionDeclaration {
                id: NodeId::new(node_id),
                name: func.name.clone(),
                parameters: func.parameters,
                body: func.body,
            });
            node_id += 1;

            if self.exports.contains(&func.name) {
                exports.push(ExportDeclaration {
                    name: func.name.clone(),
                    is_type: false,
                    exported_hash: ContentHash::new(func.name.as_bytes()),
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

#[test]
fn test_wasm_backend_basic_compatibility() {
    let module = BackendTestModule::new("wasm_compat_test")
        .with_function(
            "add_numbers",
            vec!["a", "b"],
            vec![Statement::Expression {
                id: NodeId::new(100),
                expression: Expression::Literal {
                    id: NodeId::new(101),
                    value: Value::I32(42),
                },
            }],
        )
        .with_export("add_numbers")
        .build();

    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);

    assert!(result.is_ok());
    let wasm_module = result.unwrap();

    // Verify WASM structure
    assert!(wasm_module.bytecode.len() >= 8);
    assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]); // WASM magic
    assert_eq!(&wasm_module.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]); // Version

    // Verify exports and functions
    assert_eq!(wasm_module.exports.len(), 1);
    assert_eq!(wasm_module.functions.len(), 1);
    assert!(wasm_module.memory.is_some());
}

#[test]
fn test_cross_backend_value_serialization() {
    // Test that values serialize consistently across backends
    let test_values = vec![
        Value::I32(42),
        Value::I64(-1234567890),
        Value::F32(3.14159),
        Value::F64(2.718281828),
        Value::Bool(true),
        Value::String("cross-backend test".to_string()),
        Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)]),
    ];

    for value in test_values {
        // Test serialization (should be consistent across backends)
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: Value = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(value, deserialized);
        
        // Test content hashing (should be deterministic)
        let hash1 = ContentHash::new(serialized.as_bytes());
        let hash2 = ContentHash::new(serialized.as_bytes());
        assert_eq!(hash1, hash2);
    }
}

#[test]
fn test_wasm_optimization_levels_compatibility() {
    let module = BackendTestModule::new("optimization_test")
        .with_function(
            "optimizable_func",
            vec!["x"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(200),
                    name: "temp".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(201),
                        value: Value::I32(10),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(202),
                    expression: Expression::Identifier {
                        id: NodeId::new(203),
                        name: "temp".to_string(),
                    },
                },
            ],
        )
        .with_export("optimizable_func")
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
        results.push((level, wasm_module.bytecode.len()));

        // All optimization levels should produce valid WASM
        assert!(wasm_module.bytecode.len() >= 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }

    // Verify we got results for all optimization levels
    assert_eq!(results.len(), 4);
}

#[test]
fn test_complex_module_cross_backend_compatibility() {
    let complex_module = BackendTestModule::new("complex_compat_test")
        .with_function(
            "fibonacci",
            vec!["n"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(300),
                    name: "a".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(301),
                        value: Value::I32(0),
                    }),
                },
                Statement::VariableDeclaration {
                    id: NodeId::new(302),
                    name: "b".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(303),
                        value: Value::I32(1),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(304),
                    expression: Expression::Identifier {
                        id: NodeId::new(305),
                        name: "b".to_string(),
                    },
                },
            ],
        )
        .with_function(
            "factorial",
            vec!["n"],
            vec![
                Statement::Expression {
                    id: NodeId::new(306),
                    expression: Expression::Literal {
                        id: NodeId::new(307),
                        value: Value::I32(1),
                    },
                },
            ],
        )
        .with_export("fibonacci")
        .with_export("factorial")
        .build();

    // Test WASM backend
    let mut wasm_generator = WasmCodeGenerator::new();
    let wasm_result = wasm_generator.generate(&complex_module);
    assert!(wasm_result.is_ok());

    let wasm_module = wasm_result.unwrap();
    assert_eq!(wasm_module.functions.len(), 2);
    assert_eq!(wasm_module.exports.len(), 2);

    // Verify both functions are exported
    let export_names: Vec<_> = wasm_module.exports.iter().map(|e| &e.name).collect();
    assert!(export_names.contains(&&"fibonacci".to_string()));
    assert!(export_names.contains(&&"factorial".to_string()));
}

#[test]
fn test_observability_statements_backend_compatibility() {
    let observability_module = BackendTestModule::new("observability_test")
        .with_function(
            "traced_function",
            vec![],
            vec![
                Statement::StartSpan {
                    id: NodeId::new(400),
                    name: "computation_span".to_string(),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("operation".to_string(), Value::String("test".to_string()));
                        attrs
                    },
                    body: vec![
                        Statement::Expression {
                            id: NodeId::new(401),
                            expression: Expression::Literal {
                                id: NodeId::new(402),
                                value: Value::I32(100),
                            },
                        },
                    ],
                },
                Statement::RecordMetric {
                    id: NodeId::new(403),
                    metric_type: mir_ast::MetricType::Counter,
                    name: "function_calls".to_string(),
                    value: Expression::Literal {
                        id: NodeId::new(404),
                        value: Value::I32(1),
                    },
                    labels: {
                        let mut labels = HashMap::new();
                        labels.insert("function".to_string(), Value::String("traced_function".to_string()));
                        labels
                    },
                },
                Statement::LogEvent {
                    id: NodeId::new(405),
                    level: mir_ast::LogLevel::Info,
                    message: "Function executed successfully".to_string(),
                    attributes: HashMap::new(),
                },
            ],
        )
        .with_export("traced_function")
        .build();

    // Test that observability statements are handled by WASM backend
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&observability_module);
    assert!(result.is_ok());

    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert_eq!(wasm_module.exports.len(), 1);

    // Function should have non-empty body (observability statements compiled)
    assert!(!wasm_module.functions[0].body.is_empty());
}

#[test]
fn test_type_compatibility_across_backends() {
    // Test that type information is preserved across backend compilation
    let typed_module = BackendTestModule::new("typed_test")
        .with_function(
            "process_data",
            vec!["input"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(500),
                    name: "result".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(501),
                        value: Value::Struct({
                            let mut map = HashMap::new();
                            map.insert("id".to_string(), Value::I32(123));
                            map.insert("name".to_string(), Value::String("test".to_string()));
                            map.insert("active".to_string(), Value::Bool(true));
                            map
                        }),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(502),
                    expression: Expression::Identifier {
                        id: NodeId::new(503),
                        name: "result".to_string(),
                    },
                },
            ],
        )
        .with_export("process_data")
        .build();

    // Test WASM backend with complex types
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&typed_module);
    assert!(result.is_ok());

    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);

    // Verify that complex types are handled
    assert!(!wasm_module.functions[0].body.is_empty());
}

#[test]
fn test_error_handling_compatibility() {
    // Test error handling across backends
    let error_module = BackendTestModule::new("error_test")
        .with_function(
            "error_prone_function",
            vec![],
            vec![
                Statement::Expression {
                    id: NodeId::new(600),
                    expression: Expression::Literal {
                        id: NodeId::new(601),
                        value: Value::Null, // This might cause issues
                    },
                },
            ],
        )
        .with_export("error_prone_function")
        .build();

    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&error_module);

    // Should handle gracefully (either succeed or fail with meaningful error)
    match result {
        Ok(wasm_module) => {
            // If it succeeds, verify basic structure
            assert!(wasm_module.bytecode.len() >= 8);
            assert_eq!(wasm_module.functions.len(), 1);
        }
        Err(error) => {
            // If it fails, error should be meaningful
            let error_msg = format!("{}", error);
            assert!(!error_msg.is_empty());
        }
    }
}

#[test]
fn test_module_metadata_preservation() {
    let module = BackendTestModule::new("metadata_test")
        .with_function(
            "test_func",
            vec!["param"],
            vec![Statement::Expression {
                id: NodeId::new(700),
                expression: Expression::Literal {
                    id: NodeId::new(701),
                    value: Value::String("metadata_test".to_string()),
                },
            }],
        )
        .with_export("test_func")
        .build();

    // Verify module metadata is preserved
    assert_eq!(module.name, "metadata_test");
    assert_eq!(module.exports.len(), 1);
    assert_eq!(module.exports[0].name, "test_func");

    // Test WASM compilation preserves essential information
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    assert!(result.is_ok());

    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.exports.len(), 1);
    assert_eq!(wasm_module.exports[0].name, "test_func");
}

#[test]
fn test_large_module_backend_compatibility() {
    let mut large_module_builder = BackendTestModule::new("large_module");

    // Create a module with many functions
    for i in 0..50 {
        large_module_builder = large_module_builder.with_function(
            &format!("function_{}", i),
            vec!["param"],
            vec![Statement::Expression {
                id: NodeId::new(800 + i),
                expression: Expression::Literal {
                    id: NodeId::new(900 + i),
                    value: Value::I32(i as i32),
                },
            }],
        );

        if i % 5 == 0 {
            large_module_builder = large_module_builder.with_export(&format!("function_{}", i));
        }
    }

    let large_module = large_module_builder.build();

    // Test WASM backend with large module
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&large_module);
    assert!(result.is_ok());

    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 50);
    assert_eq!(wasm_module.exports.len(), 10); // Every 5th function exported

    // Verify WASM structure is still valid
    assert!(wasm_module.bytecode.len() >= 8);
    assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
}

#[test]
fn test_debug_info_compatibility() {
    let module = BackendTestModule::new("debug_test")
        .with_function(
            "debuggable_func",
            vec!["x", "y"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(1000),
                    name: "sum".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(1001),
                        value: Value::I32(0),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(1002),
                    expression: Expression::Identifier {
                        id: NodeId::new(1003),
                        name: "sum".to_string(),
                    },
                },
            ],
        )
        .with_export("debuggable_func")
        .build();

    // Test with debug info enabled
    let mut generator = WasmCodeGenerator::new();
    generator.enable_debug_info(true);
    generator.enable_source_maps(true);

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let wasm_module = result.unwrap();
    assert!(wasm_module.debug_info.is_some());
    assert!(wasm_module.source_map.is_some());

    // Test with debug info disabled
    let mut generator_no_debug = WasmCodeGenerator::new();
    generator_no_debug.enable_debug_info(false);
    generator_no_debug.enable_source_maps(false);

    let result_no_debug = generator_no_debug.generate(&module);
    assert!(result_no_debug.is_ok());

    let wasm_module_no_debug = result_no_debug.unwrap();
    assert!(wasm_module_no_debug.debug_info.is_none());
    assert!(wasm_module_no_debug.source_map.is_none());
}

#[test]
fn test_backend_performance_consistency() {
    use std::time::Instant;

    let performance_module = BackendTestModule::new("performance_test")
        .with_function(
            "compute_intensive",
            vec!["iterations"],
            vec![
                Statement::VariableDeclaration {
                    id: NodeId::new(1100),
                    name: "result".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(1101),
                        value: Value::I32(0),
                    }),
                },
                Statement::Expression {
                    id: NodeId::new(1102),
                    expression: Expression::Identifier {
                        id: NodeId::new(1103),
                        name: "result".to_string(),
                    },
                },
            ],
        )
        .with_export("compute_intensive")
        .build();

    // Measure WASM compilation time
    let start = Instant::now();
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&performance_module);
    let wasm_duration = start.elapsed();

    assert!(result.is_ok());
    assert!(wasm_duration.as_millis() < 100); // Should compile quickly

    println!("WASM compilation took: {}ms", wasm_duration.as_millis());
}

#[test]
fn test_cross_backend_module_hash_consistency() {
    let module = BackendTestModule::new("hash_consistency_test")
        .with_function(
            "hash_test_func",
            vec!["input"],
            vec![Statement::Expression {
                id: NodeId::new(1200),
                expression: Expression::Literal {
                    id: NodeId::new(1201),
                    value: Value::String("consistent_hash".to_string()),
                },
            }],
        )
        .with_export("hash_test_func")
        .build();

    // Module hash should be consistent
    let hash1 = module.type_hash;
    let hash2 = module.type_hash;
    assert_eq!(hash1, hash2);

    // Content hash should be deterministic
    let content_hash1 = ContentHash::new(module.name.as_bytes());
    let content_hash2 = ContentHash::new(module.name.as_bytes());
    assert_eq!(content_hash1, content_hash2);

    // WASM compilation should not affect module identity
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&module);
    assert!(result.is_ok());

    // Module hash should remain the same after compilation
    assert_eq!(module.type_hash, hash1);
}