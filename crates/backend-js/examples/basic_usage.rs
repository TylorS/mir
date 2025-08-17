//! Basic usage example for the JavaScript/TypeScript backend

use mir_backend_js::{JsBackend, BackendOptions, JsTarget};
use mir_ast::{Module, NodeId, Statement, Expression};
use mir_types::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple MIR module
    let mut module = Module::new(NodeId::new(1), "example_module".to_string());
    
    // Add a function declaration
    module.statements.push(Statement::FunctionDeclaration {
        id: NodeId::new(2),
        name: "greet".to_string(),
        parameters: vec!["name".to_string()],
        body: vec![
            Statement::Expression {
                id: NodeId::new(3),
                expression: Expression::Literal {
                    id: NodeId::new(4),
                    value: Value::String("Hello, World!".to_string()),
                },
            },
        ],
    });
    
    // Add a variable declaration
    module.statements.push(Statement::VariableDeclaration {
        id: NodeId::new(5),
        name: "version".to_string(),
        value: Some(Expression::Literal {
            id: NodeId::new(6),
            value: Value::String("1.0.0".to_string()),
        }),
    });
    
    // Create backend with custom options
    let mut options = BackendOptions::default();
    options.js_options.target = JsTarget::ES2020;
    options.js_options.minify = false;
    options.js_options.optimize = true;
    options.emit_typescript = true;
    options.emit_source_maps = true;
    
    let backend = JsBackend::with_options(options);
    
    // Compile the module
    let output = backend.compile(&module)?;
    
    // Print the generated JavaScript
    println!("Generated JavaScript:");
    println!("{}", output.javascript.code);
    println!();
    
    // Print the generated TypeScript definitions
    if let Some(ref ts_output) = output.typescript {
        println!("Generated TypeScript definitions:");
        println!("{}", ts_output.declarations);
        println!();
    }
    
    // Print metadata
    println!("Compilation metadata:");
    println!("  Module: {}", output.metadata.module_name);
    println!("  Target: {:?}", output.metadata.target);
    println!("  Has TypeScript: {}", output.metadata.has_typescript);
    println!("  Has Source Maps: {}", output.metadata.has_source_maps);
    println!("  Compilation time: {}ms", output.metadata.compilation_time_ms);
    println!("  Optimizations: {:?}", output.metadata.optimizations_applied);
    
    Ok(())
}