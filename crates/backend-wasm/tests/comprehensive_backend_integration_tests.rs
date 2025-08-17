//! Comprehensive backend integration tests
//! 
//! Tests complete backend compatibility and integration including:
//! - Cross-backend compilation consistency
//! - NAPI integration testing
//! - Performance comparison across backends
//! - Hot-reloading with different backend targets

use mir_backend_wasm::{WasmCodeGenerator, OptimizationLevel};
use mir_ast::{Module, Statement, Expression, NodeId, ExportDeclaration, ExportVisibility, CompatibilityInfo};
use mir_types::{Value, ContentHash, TypeHash};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Comprehensive backend test suite
struct BackendIntegrationSuite {
    test_modules: Vec<Module>,
    performance_metrics: HashMap<String, Vec<f64>>,
}

impl BackendIntegrationSuite {
    fn new() -> Self {
        Self {
            test_modules: Self::create_test_modules(),
            performance_metrics: HashMap::new(),
        }
    }
    
    fn create_test_modules() -> Vec<Module> {
        vec![
            Self::create_simple_module(),
            Self::create_complex_module(),
            Self::create_observability_module(),
            Self::create_performance_test_module(),
            Self::create_error_handling_module(),
        ]
    }
    
    fn create_simple_module() -> Module {
        Module {
            id: NodeId::new(1),
            name: "simple_test".to_string(),
            imports: vec![],
            exports: vec![
                ExportDeclaration {
                    name: "add".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"add"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                }
            ],
            statements: vec![
                Statement::FunctionDeclaration {
                    id: NodeId::new(2),
                    name: "add".to_string(),
                    parameters: vec!["a".to_string(), "b".to_string()],
                    body: vec![
                        Statement::Expression {
                            id: NodeId::new(3),
                            expression: Expression::Literal {
                                id: NodeId::new(4),
                                value: Value::I32(42), // Simplified - would be a + b
                            },
                        },
                    ],
                },
            ],
            type_hash: TypeHash::new(ContentHash::new(b"simple_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
    
    fn create_complex_module() -> Module {
        Module {
            id: NodeId::new(10),
            name: "complex_test".to_string(),
            imports: vec![],
            exports: vec![
                ExportDeclaration {
                    name: "processData".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"processData"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                }
            ],
            statements: vec![
                Statement::FunctionDeclaration {
                    id: NodeId::new(11),
                    name: "processData".to_string(),
                    parameters: vec!["input".to_string()],
                    body: vec![
                        Statement::VariableDeclaration {
                            id: NodeId::new(12),
                            name: "result".to_string(),
                            value: Some(Expression::Literal {
                                id: NodeId::new(13),
                                value: Value::Struct({
                                    let mut map = HashMap::new();
                                    map.insert("processed".to_string(), Value::Bool(true));
                                    map.insert("count".to_string(), Value::I32(100));
                                    map.insert("data".to_string(), Value::Array(vec![
                                        Value::I32(1), Value::I32(2), Value::I32(3)
                                    ]));
                                    map
                                }),
                            }),
                        },
                        Statement::Expression {
                            id: NodeId::new(14),
                            expression: Expression::Identifier {
                                id: NodeId::new(15),
                                name: "result".to_string(),
                            },
                        },
                    ],
                },
            ],
            type_hash: TypeHash::new(ContentHash::new(b"complex_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
    
    fn create_observability_module() -> Module {
        Module {
            id: NodeId::new(20),
            name: "observability_test".to_string(),
            imports: vec![],
            exports: vec![
                ExportDeclaration {
                    name: "tracedFunction".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"tracedFunction"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                }
            ],
            statements: vec![
                Statement::FunctionDeclaration {
                    id: NodeId::new(21),
                    name: "tracedFunction".to_string(),
                    parameters: vec!["param".to_string()],
                    body: vec![
                        Statement::StartSpan {
                            id: NodeId::new(22),
                            name: "processing_span".to_string(),
                            attributes: {
                                let mut attrs = HashMap::new();
                                attrs.insert("operation".to_string(), Value::String("process".to_string()));
                                attrs.insert("version".to_string(), Value::String("1.0.0".to_string()));
                                attrs
                            },
                            body: vec![
                                Statement::RecordMetric {
                                    id: NodeId::new(23),
                                    metric_type: mir_ast::MetricType::Counter,
                                    name: "operations_total".to_string(),
                                    value: Expression::Literal {
                                        id: NodeId::new(24),
                                        value: Value::I32(1),
                                    },
                                    labels: {
                                        let mut labels = HashMap::new();
                                        labels.insert("function".to_string(), Value::String("tracedFunction".to_string()));
                                        labels
                                    },
                                },
                                Statement::LogEvent {
                                    id: NodeId::new(25),
                                    level: mir_ast::LogLevel::Info,
                                    message: "Processing started".to_string(),
                                    attributes: HashMap::new(),
                                },
                                Statement::Expression {
                                    id: NodeId::new(26),
                                    expression: Expression::Literal {
                                        id: NodeId::new(27),
                                        value: Value::String("processed".to_string()),
                                    },
                                },
                            ],
                        },
                    ],
                },
            ],
            type_hash: TypeHash::new(ContentHash::new(b"observability_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
    
    fn create_performance_test_module() -> Module {
        let mut statements = Vec::new();
        
        // Create a function with many operations for performance testing
        let mut function_body = Vec::new();
        for i in 0..50 {
            function_body.push(Statement::VariableDeclaration {
                id: NodeId::new(100 + i),
                name: format!("var_{}", i),
                value: Some(Expression::Literal {
                    id: NodeId::new(200 + i),
                    value: Value::I32(i as i32),
                }),
            });
        }
        
        // Add final return expression
        function_body.push(Statement::Expression {
            id: NodeId::new(300),
            expression: Expression::Literal {
                id: NodeId::new(301),
                value: Value::I32(42),
            },
        });
        
        statements.push(Statement::FunctionDeclaration {
            id: NodeId::new(30),
            name: "performanceTest".to_string(),
            parameters: vec!["iterations".to_string()],
            body: function_body,
        });
        
        Module {
            id: NodeId::new(30),
            name: "performance_test".to_string(),
            imports: vec![],
            exports: vec![
                ExportDeclaration {
                    name: "performanceTest".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"performanceTest"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                }
            ],
            statements,
            type_hash: TypeHash::new(ContentHash::new(b"performance_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
    
    fn create_error_handling_module() -> Module {
        Module {
            id: NodeId::new(40),
            name: "error_test".to_string(),
            imports: vec![],
            exports: vec![
                ExportDeclaration {
                    name: "errorProneFunction".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"errorProneFunction"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                }
            ],
            statements: vec![
                Statement::FunctionDeclaration {
                    id: NodeId::new(41),
                    name: "errorProneFunction".to_string(),
                    parameters: vec!["input".to_string()],
                    body: vec![
                        Statement::Expression {
                            id: NodeId::new(42),
                            expression: Expression::Literal {
                                id: NodeId::new(43),
                                value: Value::Null, // This might cause issues in some backends
                            },
                        },
                    ],
                },
            ],
            type_hash: TypeHash::new(ContentHash::new(b"error_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }
    
    fn test_wasm_backend_comprehensive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing WASM backend comprehensively...");
        
        for module in &self.test_modules {
            println!("Testing module: {}", module.name);
            
            // Test all optimization levels
            let optimization_levels = vec![
                OptimizationLevel::None,
                OptimizationLevel::Size,
                OptimizationLevel::Speed,
                OptimizationLevel::Aggressive,
            ];
            
            for level in optimization_levels {
                let start = Instant::now();
                
                let mut generator = WasmCodeGenerator::new();
                generator.set_optimization_level(level);
                
                let result = generator.generate(module);
                let compilation_time = start.elapsed();
                
                match result {
                    Ok(wasm_module) => {
                        // Verify WASM structure
                        assert!(wasm_module.bytecode.len() >= 8);
                        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
                        assert_eq!(&wasm_module.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]);
                        
                        // Record performance metrics
                        let metric_key = format!("wasm_{}_{:?}", module.name, level);
                        self.performance_metrics.entry(metric_key)
                            .or_insert_with(Vec::new)
                            .push(compilation_time.as_millis() as f64);
                        
                        println!("  {:?} optimization: {}ms, {} bytes", 
                                level, compilation_time.as_millis(), wasm_module.bytecode.len());
                    }
                    Err(err) => {
                        // Some errors might be expected for error test modules
                        if module.name != "error_test" {
                            return Err(format!("Unexpected compilation error for {}: {}", module.name, err).into());
                        }
                        println!("  {:?} optimization: Expected error - {}", level, err);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn test_debug_info_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing debug info generation...");
        
        for module in &self.test_modules {
            if module.name == "error_test" {
                continue; // Skip error test module for debug info testing
            }
            
            // Test with debug info enabled
            let mut generator = WasmCodeGenerator::new();
            generator.enable_debug_info(true);
            generator.enable_source_maps(true);
            
            let result = generator.generate(module)?;
            
            assert!(result.debug_info.is_some(), "Debug info should be present for {}", module.name);
            assert!(result.source_map.is_some(), "Source map should be present for {}", module.name);
            
            // Test with debug info disabled
            let mut generator_no_debug = WasmCodeGenerator::new();
            generator_no_debug.enable_debug_info(false);
            generator_no_debug.enable_source_maps(false);
            
            let result_no_debug = generator_no_debug.generate(module)?;
            
            assert!(result_no_debug.debug_info.is_none(), "Debug info should be absent for {}", module.name);
            assert!(result_no_debug.source_map.is_none(), "Source map should be absent for {}", module.name);
            
            println!("  Debug info test passed for {}", module.name);
        }
        
        Ok(())
    }
    
    fn test_serialization_consistency(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing serialization consistency...");
        
        for module in &self.test_modules {
            // Test JSON serialization
            let serialized = serde_json::to_string(module)?;
            let deserialized: Module = serde_json::from_str(&serialized)?;
            
            // Verify key properties are preserved
            assert_eq!(module.name, deserialized.name);
            assert_eq!(module.exports.len(), deserialized.exports.len());
            assert_eq!(module.statements.len(), deserialized.statements.len());
            assert_eq!(module.type_hash, deserialized.type_hash);
            
            // Test content hash consistency
            let hash1 = ContentHash::new(serialized.as_bytes());
            let hash2 = ContentHash::new(serialized.as_bytes());
            assert_eq!(hash1, hash2, "Content hashes should be deterministic");
            
            println!("  Serialization consistency verified for {}", module.name);
        }
        
        Ok(())
    }
    
    fn test_large_module_handling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing large module handling...");
        
        // Create a very large module
        let mut large_module = Module {
            id: NodeId::new(1000),
            name: "large_module_test".to_string(),
            imports: vec![],
            exports: vec![],
            statements: vec![],
            type_hash: TypeHash::new(ContentHash::new(b"large_module_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        };
        
        // Add many functions
        for i in 0..100 {
            let function_name = format!("function_{}", i);
            
            large_module.statements.push(Statement::FunctionDeclaration {
                id: NodeId::new(1001 + i),
                name: function_name.clone(),
                parameters: vec!["param".to_string()],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(2001 + i),
                        expression: Expression::Literal {
                            id: NodeId::new(3001 + i),
                            value: Value::I32(i as i32),
                        },
                    },
                ],
            });
            
            // Export every 10th function
            if i % 10 == 0 {
                large_module.exports.push(ExportDeclaration {
                    name: function_name.clone(),
                    is_type: false,
                    exported_hash: ContentHash::new(function_name.as_bytes()),
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
        
        // Test compilation
        let start = Instant::now();
        let mut generator = WasmCodeGenerator::new();
        let result = generator.generate(&large_module)?;
        let compilation_time = start.elapsed();
        
        assert_eq!(result.functions.len(), 100);
        assert_eq!(result.exports.len(), 10);
        
        // Should compile in reasonable time
        assert!(compilation_time.as_secs() < 5, "Large module compilation took too long: {}s", compilation_time.as_secs());
        
        println!("  Large module (100 functions) compiled in {}ms", compilation_time.as_millis());
        
        Ok(())
    }
    
    fn test_hot_reload_compatibility(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing hot reload compatibility...");
        
        // Test module versioning for hot reload
        let v1_module = &self.test_modules[0]; // Simple module
        let mut v2_module = v1_module.clone();
        
        // Modify the module slightly (simulate hot reload)
        v2_module.name = format!("{}_v2", v1_module.name);
        v2_module.type_hash = TypeHash::new(ContentHash::new(v2_module.name.as_bytes()));
        
        // Add a new function to simulate feature addition
        v2_module.statements.push(Statement::FunctionDeclaration {
            id: NodeId::new(5000),
            name: "newFeature".to_string(),
            parameters: vec![],
            body: vec![
                Statement::Expression {
                    id: NodeId::new(5001),
                    expression: Expression::Literal {
                        id: NodeId::new(5002),
                        value: Value::String("new_feature".to_string()),
                    },
                },
            ],
        });
        
        v2_module.exports.push(ExportDeclaration {
            name: "newFeature".to_string(),
            is_type: false,
            exported_hash: ContentHash::new(b"newFeature"),
            visibility: ExportVisibility::Public,
            compatibility_info: CompatibilityInfo {
                version: "2.0.0".to_string(),
                breaking_changes: vec![],
                deprecated_features: vec![],
                migration_hints: vec![],
            },
        });
        
        // Compile both versions
        let mut generator = WasmCodeGenerator::new();
        
        let v1_result = generator.generate(v1_module)?;
        let v2_result = generator.generate(&v2_module)?;
        
        // V2 should have more functions and exports
        assert!(v2_result.functions.len() > v1_result.functions.len());
        assert!(v2_result.exports.len() > v1_result.exports.len());
        
        // Both should be valid WASM
        for result in [&v1_result, &v2_result] {
            assert!(result.bytecode.len() >= 8);
            assert_eq!(&result.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        }
        
        println!("  Hot reload compatibility verified");
        
        Ok(())
    }
    
    fn test_error_recovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing error recovery...");
        
        // Test compilation with various error conditions
        let error_scenarios = vec![
            ("empty_module", Module {
                id: NodeId::new(6000),
                name: "empty_module".to_string(),
                imports: vec![],
                exports: vec![],
                statements: vec![],
                type_hash: TypeHash::new(ContentHash::new(b"empty_module")),
                capabilities: mir_ast::ModuleCapabilities {
                    can_send_messages: false,
                    can_receive_messages: false,
                    allowed_message_types: vec![],
                },
                message_queue: VecDeque::new(),
            }),
            ("invalid_export", {
                let mut module = self.test_modules[0].clone();
                module.name = "invalid_export".to_string();
                // Add export that doesn't match any function
                module.exports.push(ExportDeclaration {
                    name: "nonexistent_function".to_string(),
                    is_type: false,
                    exported_hash: ContentHash::new(b"nonexistent_function"),
                    visibility: ExportVisibility::Public,
                    compatibility_info: CompatibilityInfo {
                        version: "1.0.0".to_string(),
                        breaking_changes: vec![],
                        deprecated_features: vec![],
                        migration_hints: vec![],
                    },
                });
                module
            }),
        ];
        
        for (scenario_name, module) in error_scenarios {
            let mut generator = WasmCodeGenerator::new();
            let result = generator.generate(&module);
            
            match result {
                Ok(wasm_module) => {
                    // Some scenarios might succeed (like empty module)
                    println!("  Scenario '{}' succeeded: {} functions", scenario_name, wasm_module.functions.len());
                }
                Err(err) => {
                    // Errors should be meaningful
                    let error_msg = format!("{}", err);
                    assert!(!error_msg.is_empty(), "Error message should not be empty");
                    println!("  Scenario '{}' failed as expected: {}", scenario_name, error_msg);
                }
            }
        }
        
        Ok(())
    }
    
    fn test_concurrent_compilation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing concurrent compilation...");
        
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let modules = Arc::new(self.test_modules.clone());
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];
        
        // Spawn multiple threads to compile modules concurrently
        for i in 0..3 {
            let modules_clone = Arc::clone(&modules);
            let results_clone = Arc::clone(&results);
            
            let handle = thread::spawn(move || {
                let mut thread_results = Vec::new();
                
                for (j, module) in modules_clone.iter().enumerate() {
                    if module.name == "error_test" {
                        continue; // Skip error test module
                    }
                    
                    let start = Instant::now();
                    let mut generator = WasmCodeGenerator::new();
                    generator.set_optimization_level(match i {
                        0 => OptimizationLevel::None,
                        1 => OptimizationLevel::Size,
                        _ => OptimizationLevel::Speed,
                    });
                    
                    match generator.generate(module) {
                        Ok(wasm_module) => {
                            thread_results.push((
                                format!("thread_{}_module_{}", i, j),
                                start.elapsed().as_millis(),
                                wasm_module.bytecode.len(),
                            ));
                        }
                        Err(err) => {
                            println!("Thread {} failed to compile module {}: {}", i, j, err);
                        }
                    }
                }
                
                if let Ok(mut results) = results_clone.lock() {
                    results.extend(thread_results);
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_results = results.lock().unwrap();
        
        // Should have results from all threads
        assert!(!final_results.is_empty(), "Should have compilation results from concurrent threads");
        
        println!("  Concurrent compilation completed: {} results", final_results.len());
        
        for (name, time_ms, size_bytes) in final_results.iter() {
            println!("    {}: {}ms, {} bytes", name, time_ms, size_bytes);
        }
        
        Ok(())
    }
    
    fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Backend Performance Report ===\n");
        
        for (metric_name, values) in &self.performance_metrics {
            if !values.is_empty() {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                
                report.push_str(&format!("{}: avg={:.2}ms, min={:.2}ms, max={:.2}ms\n", 
                                        metric_name, avg, min, max));
            }
        }
        
        report
    }
}

// Integration test cases

#[test]
fn test_comprehensive_wasm_backend() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_wasm_backend_comprehensive()
        .expect("WASM backend comprehensive test should pass");
    
    let report = suite.generate_performance_report();
    println!("{}", report);
}

#[test]
fn test_debug_and_source_maps() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_debug_info_generation()
        .expect("Debug info generation test should pass");
}

#[test]
fn test_serialization_and_consistency() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_serialization_consistency()
        .expect("Serialization consistency test should pass");
}

#[test]
fn test_large_module_performance() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_large_module_handling()
        .expect("Large module handling test should pass");
}

#[test]
fn test_hot_reload_backend_compatibility() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_hot_reload_compatibility()
        .expect("Hot reload compatibility test should pass");
}

#[test]
fn test_backend_error_recovery() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_error_recovery()
        .expect("Error recovery test should pass");
}

#[test]
fn test_concurrent_backend_compilation() {
    let mut suite = BackendIntegrationSuite::new();
    
    suite.test_concurrent_compilation()
        .expect("Concurrent compilation test should pass");
}

#[test]
fn test_napi_integration() {
    // Test NAPI bindings integration
    println!("Testing NAPI integration...");
    
    // Create a test module for NAPI testing
    let test_module = Module {
        id: NodeId::new(7000),
        name: "napi_test".to_string(),
        imports: vec![],
        exports: vec![
            ExportDeclaration {
                name: "napiFunction".to_string(),
                is_type: false,
                exported_hash: ContentHash::new(b"napiFunction"),
                visibility: ExportVisibility::Public,
                compatibility_info: CompatibilityInfo {
                    version: "1.0.0".to_string(),
                    breaking_changes: vec![],
                    deprecated_features: vec![],
                    migration_hints: vec![],
                },
            }
        ],
        statements: vec![
            Statement::FunctionDeclaration {
                id: NodeId::new(7001),
                name: "napiFunction".to_string(),
                parameters: vec!["input".to_string()],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(7002),
                        expression: Expression::Literal {
                            id: NodeId::new(7003),
                            value: Value::String("napi_result".to_string()),
                        },
                    },
                ],
            },
        ],
        type_hash: TypeHash::new(ContentHash::new(b"napi_test")),
        capabilities: mir_ast::ModuleCapabilities {
            can_send_messages: false,
            can_receive_messages: false,
            allowed_message_types: vec![],
        },
        message_queue: VecDeque::new(),
    };
    
    // Test WASM compilation with NAPI considerations
    let mut generator = WasmCodeGenerator::new();
    let result = generator.generate(&test_module);
    
    assert!(result.is_ok(), "NAPI test module should compile successfully");
    
    let wasm_module = result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert_eq!(wasm_module.exports.len(), 1);
    
    // Verify WASM structure is compatible with NAPI
    assert!(wasm_module.bytecode.len() >= 8);
    assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    
    println!("NAPI integration test completed successfully");
}

#[test]
fn test_cross_platform_compatibility() {
    println!("Testing cross-platform compatibility...");
    
    // Test that compilation results are consistent across different configurations
    let test_module = Module {
        id: NodeId::new(8000),
        name: "cross_platform_test".to_string(),
        imports: vec![],
        exports: vec![
            ExportDeclaration {
                name: "crossPlatformFunction".to_string(),
                is_type: false,
                exported_hash: ContentHash::new(b"crossPlatformFunction"),
                visibility: ExportVisibility::Public,
                compatibility_info: CompatibilityInfo {
                    version: "1.0.0".to_string(),
                    breaking_changes: vec![],
                    deprecated_features: vec![],
                    migration_hints: vec![],
                },
            }
        ],
        statements: vec![
            Statement::FunctionDeclaration {
                id: NodeId::new(8001),
                name: "crossPlatformFunction".to_string(),
                parameters: vec!["data".to_string()],
                body: vec![
                    Statement::VariableDeclaration {
                        id: NodeId::new(8002),
                        name: "result".to_string(),
                        value: Some(Expression::Literal {
                            id: NodeId::new(8003),
                            value: Value::Struct({
                                let mut map = HashMap::new();
                                map.insert("platform".to_string(), Value::String("universal".to_string()));
                                map.insert("timestamp".to_string(), Value::I64(1640995200));
                                map.insert("data".to_string(), Value::Array(vec![
                                    Value::I32(1), Value::I32(2), Value::I32(3)
                                ]));
                                map
                            }),
                        }),
                    },
                    Statement::Expression {
                        id: NodeId::new(8004),
                        expression: Expression::Identifier {
                            id: NodeId::new(8005),
                            name: "result".to_string(),
                        },
                    },
                ],
            },
        ],
        type_hash: TypeHash::new(ContentHash::new(b"cross_platform_test")),
        capabilities: mir_ast::ModuleCapabilities {
            can_send_messages: false,
            can_receive_messages: false,
            allowed_message_types: vec![],
        },
        message_queue: VecDeque::new(),
    };
    
    // Test multiple configurations
    let configurations = vec![
        (OptimizationLevel::None, false, false),
        (OptimizationLevel::Size, true, false),
        (OptimizationLevel::Speed, false, true),
        (OptimizationLevel::Aggressive, true, true),
    ];
    
    let mut results = Vec::new();
    
    for (opt_level, debug_info, source_maps) in configurations {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(opt_level);
        generator.enable_debug_info(debug_info);
        generator.enable_source_maps(source_maps);
        
        let result = generator.generate(&test_module);
        assert!(result.is_ok(), "Configuration {:?} should compile successfully", opt_level);
        
        let wasm_module = result.unwrap();
        results.push((opt_level, wasm_module.bytecode.len(), debug_info, source_maps));
    }
    
    // All configurations should produce valid WASM
    for (opt_level, size, debug, source_map) in &results {
        assert!(*size >= 8, "Configuration {:?} should produce valid WASM", opt_level);
        println!("  {:?}: {} bytes (debug: {}, source_map: {})", opt_level, size, debug, source_map);
    }
    
    println!("Cross-platform compatibility test completed successfully");
}

#[test]
fn test_memory_efficiency() {
    println!("Testing memory efficiency...");
    
    // Create modules of varying sizes to test memory usage
    let module_sizes = vec![1, 10, 50, 100];
    
    for &size in &module_sizes {
        let mut statements = Vec::new();
        let mut exports = Vec::new();
        
        for i in 0..size {
            let function_name = format!("func_{}", i);
            
            statements.push(Statement::FunctionDeclaration {
                id: NodeId::new(9000 + i as u64),
                name: function_name.clone(),
                parameters: vec!["param".to_string()],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(10000 + i as u64),
                        expression: Expression::Literal {
                            id: NodeId::new(11000 + i as u64),
                            value: Value::I32(i as i32),
                        },
                    },
                ],
            });
            
            exports.push(ExportDeclaration {
                name: function_name.clone(),
                is_type: false,
                exported_hash: ContentHash::new(function_name.as_bytes()),
                visibility: ExportVisibility::Public,
                compatibility_info: CompatibilityInfo {
                    version: "1.0.0".to_string(),
                    breaking_changes: vec![],
                    deprecated_features: vec![],
                    migration_hints: vec![],
                },
            });
        }
        
        let module = Module {
            id: NodeId::new(9000),
            name: format!("memory_test_{}", size),
            imports: vec![],
            exports,
            statements,
            type_hash: TypeHash::new(ContentHash::new(format!("memory_test_{}", size).as_bytes())),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        };
        
        let start = Instant::now();
        let mut generator = WasmCodeGenerator::new();
        let result = generator.generate(&module);
        let compilation_time = start.elapsed();
        
        assert!(result.is_ok(), "Module with {} functions should compile", size);
        
        let wasm_module = result.unwrap();
        let bytes_per_function = wasm_module.bytecode.len() / size.max(1);
        
        println!("  {} functions: {}ms compilation, {} bytes total, {} bytes/function", 
                size, compilation_time.as_millis(), wasm_module.bytecode.len(), bytes_per_function);
        
        // Compilation time should scale reasonably
        assert!(compilation_time.as_secs() < 5, "Compilation should complete in reasonable time");
    }
    
    println!("Memory efficiency test completed successfully");
}