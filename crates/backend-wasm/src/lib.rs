//! MIR Backend WASM - WebAssembly code generation
//! 
//! This crate provides WebAssembly compilation from MIR IR, including:
//! - WASM module generation
//! - NAPI bindings for Node.js
//! - Optimization passes

pub mod codegen;
pub mod napi;
pub mod optimization;

pub use codegen::{WasmCodeGenerator, WasmModule, OptimizationLevel, CodegenError};
pub use napi::WasmNapiBindings;
pub use optimization::{WasmOptimizer, OptimizationStats};

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{Module, Statement, Expression, NodeId};
    use mir_types::Value;

    #[test]
    fn test_basic_wasm_generation() {
        let generator = WasmCodeGenerator::new();
        
        // Create a simple module with a function
        let module = Module {
            id: NodeId::new(1),
            name: "test_module".to_string(),
            imports: vec![],
            exports: vec![],
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
                                value: Value::I32(42),
                            },
                        },
                    ],
                },
            ],
            type_hash: mir_types::TypeHash::new(mir_types::ContentHash::new(b"test_module")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: std::collections::VecDeque::new(),
        };
        
        // Generate WASM module
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let wasm_module = result.unwrap();
        
        // Verify WASM magic number
        assert!(wasm_module.bytecode.len() >= 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]); // WASM magic
        assert_eq!(&wasm_module.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]); // version 1
        
        // Verify we have at least one function
        assert!(!wasm_module.functions.is_empty());
        assert_eq!(wasm_module.functions[0].name, Some("add".to_string()));
    }
    
    #[test]
    fn test_optimization_levels() {
        let mut generator = WasmCodeGenerator::new();
        
        // Test different optimization levels
        generator.set_optimization_level(OptimizationLevel::None);
        generator.set_optimization_level(OptimizationLevel::Size);
        generator.set_optimization_level(OptimizationLevel::Speed);
        generator.set_optimization_level(OptimizationLevel::Aggressive);
        
        // Should not panic
    }
    
    #[test]
    fn test_debug_info_and_source_maps() {
        let mut generator = WasmCodeGenerator::new();
        
        generator.enable_debug_info(true);
        generator.enable_source_maps(true);
        
        let module = Module {
            id: NodeId::new(1),
            name: "debug_test".to_string(),
            imports: vec![],
            exports: vec![],
            statements: vec![],
            type_hash: mir_types::TypeHash::new(mir_types::ContentHash::new(b"debug_test")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: std::collections::VecDeque::new(),
        };
        
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let wasm_module = result.unwrap();
        
        // Debug info and source maps should be present when enabled
        assert!(wasm_module.debug_info.is_some());
        assert!(wasm_module.source_map.is_some());
    }
    
    #[test]
    fn test_optimizer() {
        let optimizer = WasmOptimizer::new();
        
        // Test with simple bytecode
        let bytecode = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // WASM header
        
        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Speed);
        assert!(result.is_ok());
        
        let optimized = result.unwrap();
        assert!(!optimized.is_empty());
        
        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }
}