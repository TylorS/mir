//! WebAssembly code generation from MIR IR
//!
//! This module implements WASM compilation from MIR IR, including:
//! - WASM module generation with proper exports
//! - Instruction translation from MIR to WASM
//! - Type mapping and function signatures
//! - Debugging information preservation

use crate::optimization::WasmOptimizer;
use mir_ast::{Expression, Module, Statement};
use mir_types::{TypeHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebAssembly code generator
pub struct WasmCodeGenerator {
    optimization_level: OptimizationLevel,
    optimizer: WasmOptimizer,
    debug_info: bool,
    source_maps: bool,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Size,
    Speed,
    Aggressive,
}

/// Generated WebAssembly module with complete metadata
#[derive(Debug, Clone)]
pub struct WasmModule {
    pub bytecode: Vec<u8>,
    pub exports: Vec<WasmExport>,
    pub imports: Vec<WasmImport>,
    pub functions: Vec<WasmFunction>,
    pub types: Vec<WasmFunctionType>,
    pub globals: Vec<WasmGlobal>,
    pub memory: Option<WasmMemory>,
    pub debug_info: Option<WasmDebugInfo>,
    pub source_map: Option<WasmSourceMap>,
}

/// WASM export declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExport {
    pub name: String,
    pub kind: WasmExportKind,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmExportKind {
    Function,
    Table,
    Memory,
    Global,
}

/// WASM import declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmImport {
    pub module: String,
    pub name: String,
    pub kind: WasmImportKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmImportKind {
    Function(u32), // type index
    Table(WasmTableType),
    Memory(WasmMemoryType),
    Global(WasmGlobalType),
}

/// WASM function definition
#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub type_index: u32,
    pub locals: Vec<WasmValueType>,
    pub body: Vec<u8>, // WASM bytecode
    pub name: Option<String>,
    pub source_location: Option<SourceLocation>,
}

/// WASM function type signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunctionType {
    pub params: Vec<WasmValueType>,
    pub results: Vec<WasmValueType>,
}

/// WASM value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

/// WASM global variable
#[derive(Debug, Clone)]
pub struct WasmGlobal {
    pub global_type: WasmGlobalType,
    pub init_expr: Vec<u8>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmGlobalType {
    pub value_type: WasmValueType,
    pub mutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTableType {
    pub element_type: WasmValueType,
    pub limits: WasmLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMemoryType {
    pub limits: WasmLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLimits {
    pub min: u32,
    pub max: Option<u32>,
}

/// WASM memory configuration
#[derive(Debug, Clone)]
pub struct WasmMemory {
    pub memory_type: WasmMemoryType,
    pub name: Option<String>,
}

/// Debug information for WASM module
#[derive(Debug, Clone)]
pub struct WasmDebugInfo {
    pub dwarf_sections: HashMap<String, Vec<u8>>,
    pub function_names: HashMap<u32, String>,
    pub local_names: HashMap<u32, HashMap<u32, String>>,
}

/// Source map for debugging
#[derive(Debug, Clone)]
pub struct WasmSourceMap {
    pub version: u32,
    pub sources: Vec<String>,
    pub mappings: String,
    pub names: Vec<String>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Code generation context
struct CodegenContext {
    type_indices: HashMap<TypeHash, u32>,
    function_indices: HashMap<String, u32>,
    global_indices: HashMap<String, u32>,
    local_indices: HashMap<String, u32>,
    current_function_locals: Vec<WasmValueType>,
    label_stack: Vec<u32>,
    next_label: u32,
}

impl WasmCodeGenerator {
    pub fn new() -> Self {
        WasmCodeGenerator {
            optimization_level: OptimizationLevel::Speed,
            optimizer: WasmOptimizer::new(),
            debug_info: true,
            source_maps: true,
        }
    }

    /// Generate WASM module from MIR IR
    pub fn generate(&self, module: &Module) -> Result<WasmModule, CodegenError> {
        let mut context = CodegenContext::new();

        // Analyze module and build type information
        self.analyze_module(module, &mut context)?;

        // Generate WASM sections
        let types = self.generate_types(&context)?;
        let imports = self.generate_imports(module, &context)?;
        let functions = self.generate_functions(module, &mut context)?;
        let exports = self.generate_exports(module, &context)?;
        let globals = self.generate_globals(module, &context)?;
        let memory = self.generate_memory(module)?;

        // Generate debug information if enabled
        let debug_info = if self.debug_info {
            Some(self.generate_debug_info(module, &functions)?)
        } else {
            None
        };

        // Generate source maps if enabled
        let source_map = if self.source_maps {
            Some(self.generate_source_map(module, &functions)?)
        } else {
            None
        };

        // Assemble WASM bytecode
        let mut bytecode =
            self.assemble_wasm_module(&types, &imports, &functions, &exports, &globals, &memory)?;

        // Apply optimizations
        if !matches!(self.optimization_level, OptimizationLevel::None) {
            bytecode = self
                .optimizer
                .optimize(&bytecode, &self.optimization_level)?;
        }

        Ok(WasmModule {
            bytecode,
            exports,
            imports,
            functions,
            types,
            globals,
            memory,
            debug_info,
            source_map,
        })
    }

    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }

    pub fn enable_debug_info(&mut self, enabled: bool) {
        self.debug_info = enabled;
    }

    pub fn enable_source_maps(&mut self, enabled: bool) {
        self.source_maps = enabled;
    }

    /// Analyze module to build type and function indices
    fn analyze_module(
        &self,
        module: &Module,
        context: &mut CodegenContext,
    ) -> Result<(), CodegenError> {
        // Analyze imports first
        for import in &module.imports {
            for item in &import.imported_items {
                if !item.is_type {
                    // Function import
                    let index = context.function_indices.len() as u32;
                    context.function_indices.insert(item.name.clone(), index);
                }
            }
        }

        // Analyze module statements to find function declarations
        for statement in &module.statements {
            if let Statement::FunctionDeclaration { name, .. } = statement {
                let index = context.function_indices.len() as u32;
                context.function_indices.insert(name.clone(), index);
            }
        }

        Ok(())
    }

    /// Generate WASM type section
    fn generate_types(
        &self,
        _context: &CodegenContext,
    ) -> Result<Vec<WasmFunctionType>, CodegenError> {
        let mut types = Vec::new();

        // Add common function types
        types.push(WasmFunctionType {
            params: vec![],
            results: vec![],
        });

        types.push(WasmFunctionType {
            params: vec![WasmValueType::I32],
            results: vec![WasmValueType::I32],
        });

        types.push(WasmFunctionType {
            params: vec![WasmValueType::I32, WasmValueType::I32],
            results: vec![WasmValueType::I32],
        });

        Ok(types)
    }

    /// Generate WASM import section
    fn generate_imports(
        &self,
        module: &Module,
        _context: &CodegenContext,
    ) -> Result<Vec<WasmImport>, CodegenError> {
        let mut imports = Vec::new();

        for import_decl in &module.imports {
            for item in &import_decl.imported_items {
                if !item.is_type {
                    imports.push(WasmImport {
                        module: import_decl.module_path.clone(),
                        name: item.name.clone(),
                        kind: WasmImportKind::Function(0), // Default to type 0
                    });
                }
            }
        }

        Ok(imports)
    }

    /// Generate WASM function section
    fn generate_functions(
        &self,
        module: &Module,
        context: &mut CodegenContext,
    ) -> Result<Vec<WasmFunction>, CodegenError> {
        let mut functions = Vec::new();

        for statement in &module.statements {
            if let Statement::FunctionDeclaration {
                name,
                parameters,
                body,
                ..
            } = statement
            {
                let function = self.generate_function(name, parameters, body, context)?;
                functions.push(function);
            }
        }

        Ok(functions)
    }

    /// Generate a single WASM function
    fn generate_function(
        &self,
        name: &str,
        parameters: &[String],
        body: &[Statement],
        context: &mut CodegenContext,
    ) -> Result<WasmFunction, CodegenError> {
        // Reset local context for this function
        context.local_indices.clear();
        context.current_function_locals.clear();
        context.label_stack.clear();
        context.next_label = 0;

        // Add parameters as locals (assuming i32 for now)
        for (i, param_name) in parameters.iter().enumerate() {
            context.local_indices.insert(param_name.clone(), i as u32);
            context.current_function_locals.push(WasmValueType::I32);
        }

        // Generate function body bytecode
        let mut bytecode = Vec::new();
        for stmt in body {
            self.generate_statement(stmt, &mut bytecode, context)?;
        }

        // Add implicit return if needed
        if bytecode.is_empty() || !self.ends_with_return(&bytecode) {
            bytecode.push(0x0F); // return
        }

        Ok(WasmFunction {
            type_index: 0, // Will be determined during assembly
            locals: context.current_function_locals.clone(),
            body: bytecode,
            name: Some(name.to_string()),
            source_location: None,
        })
    }

    /// Generate WASM export section
    fn generate_exports(
        &self,
        module: &Module,
        context: &CodegenContext,
    ) -> Result<Vec<WasmExport>, CodegenError> {
        let mut exports = Vec::new();

        for export_decl in &module.exports {
            if !export_decl.is_type {
                if let Some(&index) = context.function_indices.get(&export_decl.name) {
                    exports.push(WasmExport {
                        name: export_decl.name.clone(),
                        kind: WasmExportKind::Function,
                        index,
                    });
                }
            }
        }

        Ok(exports)
    }

    /// Generate WASM global section
    fn generate_globals(
        &self,
        _module: &Module,
        _context: &CodegenContext,
    ) -> Result<Vec<WasmGlobal>, CodegenError> {
        let globals = Vec::new();

        // For now, no global variables in the simplified AST
        // In a full implementation, this would analyze variable declarations
        // that are marked as global

        Ok(globals)
    }

    /// Generate WASM memory section
    fn generate_memory(&self, _module: &Module) -> Result<Option<WasmMemory>, CodegenError> {
        // Default memory configuration
        Ok(Some(WasmMemory {
            memory_type: WasmMemoryType {
                limits: WasmLimits {
                    min: 1,        // 1 page (64KB)
                    max: Some(16), // 16 pages (1MB)
                },
            },
            name: Some("memory".to_string()),
        }))
    }

    /// Generate statement bytecode
    fn generate_statement(
        &self,
        stmt: &Statement,
        bytecode: &mut Vec<u8>,
        context: &mut CodegenContext,
    ) -> Result<(), CodegenError> {
        match stmt {
            Statement::Expression { expression, .. } => {
                self.generate_expression(expression, bytecode, context)?;
                bytecode.push(0x1A); // drop
            }
            Statement::VariableDeclaration { name, value, .. } => {
                // Allocate local variable
                let local_index = context.current_function_locals.len() as u32;
                context.local_indices.insert(name.clone(), local_index);
                context.current_function_locals.push(WasmValueType::I32); // Default to i32

                // Initialize if value provided
                if let Some(init_expr) = value {
                    self.generate_expression(init_expr, bytecode, context)?;
                    bytecode.push(0x21); // local.set
                    self.encode_u32(local_index, bytecode);
                }
            }
            Statement::FunctionDeclaration { .. } => {
                // Function declarations are handled separately
                // This is just a placeholder for nested functions
            }
            Statement::StartSpan {  body, .. } => {
                // Generate span start (simplified)
                for stmt in body {
                    self.generate_statement(stmt, bytecode, context)?;
                }
            }
            Statement::EndSpan { .. } => {
                // Generate span end (simplified)
            }
            Statement::WithSpan { body, .. } => {
                // Generate statements within span context
                for stmt in body {
                    self.generate_statement(stmt, bytecode, context)?;
                }
            }
            Statement::RecordMetric { .. } => {
                // Generate metric recording (simplified)
            }
            Statement::LogEvent { .. } => {
                // Generate log event (simplified)
            }
        }

        Ok(())
    }

    /// Generate expression bytecode
    fn generate_expression(
        &self,
        expr: &Expression,
        bytecode: &mut Vec<u8>,
        context: &CodegenContext,
    ) -> Result<(), CodegenError> {
        match expr {
            Expression::Literal { value, .. } => {
                self.generate_literal(value, bytecode)?;
            }
            Expression::Identifier { name, .. } => {
                if let Some(&index) = context.local_indices.get(name) {
                    bytecode.push(0x20); // local.get
                    self.encode_u32(index, bytecode);
                } else if let Some(&index) = context.global_indices.get(name) {
                    bytecode.push(0x23); // global.get
                    self.encode_u32(index, bytecode);
                } else {
                    return Err(CodegenError::InvalidModule(format!(
                        "Undefined variable: {name}"
                    )));
                }
            }
            Expression::FunctionCall {
                function,
                arguments,
                ..
            } => {
                // Generate arguments
                for arg in arguments {
                    self.generate_expression(arg, bytecode, context)?;
                }

                // For now, assume function is an identifier
                // In a full implementation, this would handle complex function expressions
                if let Expression::Identifier { name, .. } = function.as_ref() {
                    if let Some(&index) = context.function_indices.get(name) {
                        bytecode.push(0x10); // call
                        self.encode_u32(index, bytecode);
                    } else {
                        return Err(CodegenError::InvalidModule(format!(
                            "Undefined function: {name}"
                        )));
                    }
                } else {
                    return Err(CodegenError::UnsupportedFeature(
                        "Complex function expressions not yet supported".to_string(),
                    ));
                }
            }
            Expression::CreateSpan { .. } => {
                // Generate span creation (simplified - return span ID as i32)
                bytecode.push(0x41); // i32.const
                self.encode_i32(1, bytecode); // dummy span ID
            }
            Expression::SetSpanAttribute { span, .. } => {
                // Generate span attribute setting (simplified - return the span)
                self.generate_expression(span, bytecode, context)?;
            }
            Expression::AddSpanEvent { span, .. } => {
                // Generate span event (simplified - return the span)
                self.generate_expression(span, bytecode, context)?;
            }
            Expression::GetTraceContext { .. } => {
                // Generate trace context retrieval (simplified - return dummy context)
                bytecode.push(0x41); // i32.const
                self.encode_i32(0, bytecode); // dummy context
            }
            Expression::SetTraceContext { context: ctx, .. } => {
                // Generate trace context setting (simplified - return the context)
                self.generate_expression(ctx, bytecode, context)?;
            }
        }

        Ok(())
    }

    /// Generate literal value bytecode
    fn generate_literal(&self, value: &Value, bytecode: &mut Vec<u8>) -> Result<(), CodegenError> {
        match value {
            Value::I32(n) => {
                bytecode.push(0x41); // i32.const
                self.encode_i32(*n, bytecode);
            }
            Value::I64(n) => {
                bytecode.push(0x42); // i64.const
                self.encode_i64(*n, bytecode);
            }
            Value::F32(f) => {
                bytecode.push(0x43); // f32.const
                bytecode.extend_from_slice(&f.to_le_bytes());
            }
            Value::F64(f) => {
                bytecode.push(0x44); // f64.const
                bytecode.extend_from_slice(&f.to_le_bytes());
            }
            Value::Bool(b) => {
                bytecode.push(0x41); // i32.const
                self.encode_i32(if *b { 1 } else { 0 }, bytecode);
            }
            Value::String(s) => {
                // For now, strings are not directly supported in WASM
                // In a full implementation, this would store the string in memory
                // and return a pointer
                bytecode.push(0x41); // i32.const
                self.encode_i32(s.len() as i32, bytecode); // return string length as placeholder
            }
            Value::Null => {
                bytecode.push(0x41); // i32.const
                self.encode_i32(0, bytecode); // null as 0
            }
            _ => {
                return Err(CodegenError::UnsupportedFeature(format!(
                    "Literal type not yet implemented: {value:?}"
                )));
            }
        }

        Ok(())
    }

    /// Check if bytecode ends with a return instruction
    fn ends_with_return(&self, bytecode: &[u8]) -> bool {
        bytecode.last() == Some(&0x0F)
    }

    /// Encode unsigned 32-bit integer in LEB128 format
    fn encode_u32(&self, mut value: u32, bytecode: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                bytecode.push(byte);
                break;
            } else {
                bytecode.push(byte | 0x80);
            }
        }
    }

    /// Encode signed 32-bit integer in LEB128 format
    fn encode_i32(&self, mut value: i32, bytecode: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0) {
                bytecode.push(byte);
                break;
            } else {
                bytecode.push(byte | 0x80);
            }
        }
    }

    /// Encode signed 64-bit integer in LEB128 format
    fn encode_i64(&self, mut value: i64, bytecode: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0) {
                bytecode.push(byte);
                break;
            } else {
                bytecode.push(byte | 0x80);
            }
        }
    }

    /// Assemble complete WASM module bytecode
    fn assemble_wasm_module(
        &self,
        types: &[WasmFunctionType],
        imports: &[WasmImport],
        functions: &[WasmFunction],
        exports: &[WasmExport],
        globals: &[WasmGlobal],
        memory: &Option<WasmMemory>,
    ) -> Result<Vec<u8>, CodegenError> {
        let mut bytecode = Vec::new();

        // WASM magic number and version
        bytecode.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
        bytecode.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // Type section
        if !types.is_empty() {
            self.write_section(1, &self.encode_type_section(types)?, &mut bytecode);
        }

        // Import section
        if !imports.is_empty() {
            self.write_section(2, &self.encode_import_section(imports)?, &mut bytecode);
        }

        // Function section (function type indices)
        if !functions.is_empty() {
            self.write_section(3, &self.encode_function_section(functions)?, &mut bytecode);
        }

        // Memory section
        if let Some(mem) = memory {
            self.write_section(5, &self.encode_memory_section(mem)?, &mut bytecode);
        }

        // Global section
        if !globals.is_empty() {
            self.write_section(6, &self.encode_global_section(globals)?, &mut bytecode);
        }

        // Export section
        if !exports.is_empty() {
            self.write_section(7, &self.encode_export_section(exports)?, &mut bytecode);
        }

        // Code section (function bodies)
        if !functions.is_empty() {
            self.write_section(10, &self.encode_code_section(functions)?, &mut bytecode);
        }

        Ok(bytecode)
    }

    /// Write a WASM section with header
    fn write_section(&self, section_id: u8, content: &[u8], bytecode: &mut Vec<u8>) {
        bytecode.push(section_id);
        self.encode_u32(content.len() as u32, bytecode);
        bytecode.extend_from_slice(content);
    }

    /// Encode type section
    fn encode_type_section(&self, types: &[WasmFunctionType]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(types.len() as u32, &mut section);

        for func_type in types {
            section.push(0x60); // func type
            self.encode_u32(func_type.params.len() as u32, &mut section);
            for param in &func_type.params {
                section.push(self.value_type_to_byte(*param));
            }
            self.encode_u32(func_type.results.len() as u32, &mut section);
            for result in &func_type.results {
                section.push(self.value_type_to_byte(*result));
            }
        }

        Ok(section)
    }

    /// Encode import section
    fn encode_import_section(&self, imports: &[WasmImport]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(imports.len() as u32, &mut section);

        for import in imports {
            // Module name
            self.encode_string(&import.module, &mut section);
            // Import name
            self.encode_string(&import.name, &mut section);
            // Import kind
            match &import.kind {
                WasmImportKind::Function(type_idx) => {
                    section.push(0x00); // function
                    self.encode_u32(*type_idx, &mut section);
                }
                _ => {
                    return Err(CodegenError::UnsupportedFeature(
                        "Non-function imports not yet implemented".to_string(),
                    ));
                }
            }
        }

        Ok(section)
    }

    /// Encode function section
    fn encode_function_section(&self, functions: &[WasmFunction]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(functions.len() as u32, &mut section);

        for function in functions {
            self.encode_u32(function.type_index, &mut section);
        }

        Ok(section)
    }

    /// Encode memory section
    fn encode_memory_section(&self, memory: &WasmMemory) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        section.push(0x01); // one memory

        // Limits
        if memory.memory_type.limits.max.is_some() {
            section.push(0x01); // has max
            self.encode_u32(memory.memory_type.limits.min, &mut section);
            self.encode_u32(memory.memory_type.limits.max.unwrap(), &mut section);
        } else {
            section.push(0x00); // no max
            self.encode_u32(memory.memory_type.limits.min, &mut section);
        }

        Ok(section)
    }

    /// Encode global section
    fn encode_global_section(&self, globals: &[WasmGlobal]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(globals.len() as u32, &mut section);

        for global in globals {
            section.push(self.value_type_to_byte(global.global_type.value_type));
            section.push(if global.global_type.mutable {
                0x01
            } else {
                0x00
            });
            section.extend_from_slice(&global.init_expr);
        }

        Ok(section)
    }

    /// Encode export section
    fn encode_export_section(&self, exports: &[WasmExport]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(exports.len() as u32, &mut section);

        for export in exports {
            self.encode_string(&export.name, &mut section);
            match export.kind {
                WasmExportKind::Function => section.push(0x00),
                WasmExportKind::Table => section.push(0x01),
                WasmExportKind::Memory => section.push(0x02),
                WasmExportKind::Global => section.push(0x03),
            }
            self.encode_u32(export.index, &mut section);
        }

        Ok(section)
    }

    /// Encode code section
    fn encode_code_section(&self, functions: &[WasmFunction]) -> Result<Vec<u8>, CodegenError> {
        let mut section = Vec::new();
        self.encode_u32(functions.len() as u32, &mut section);

        for function in functions {
            let mut func_body = Vec::new();

            // Encode locals
            self.encode_u32(function.locals.len() as u32, &mut func_body);
            for local_type in &function.locals {
                func_body.push(0x01); // count
                func_body.push(self.value_type_to_byte(*local_type));
            }

            // Function body
            func_body.extend_from_slice(&function.body);

            // Encode function size and body
            self.encode_u32(func_body.len() as u32, &mut section);
            section.extend_from_slice(&func_body);
        }

        Ok(section)
    }

    /// Generate debug information
    fn generate_debug_info(
        &self,
        _module: &Module,
        _functions: &[WasmFunction],
    ) -> Result<WasmDebugInfo, CodegenError> {
        // Placeholder implementation
        Ok(WasmDebugInfo {
            dwarf_sections: HashMap::new(),
            function_names: HashMap::new(),
            local_names: HashMap::new(),
        })
    }

    /// Generate source map
    fn generate_source_map(
        &self,
        _module: &Module,
        _functions: &[WasmFunction],
    ) -> Result<WasmSourceMap, CodegenError> {
        // Placeholder implementation
        Ok(WasmSourceMap {
            version: 3,
            sources: Vec::new(),
            mappings: String::new(),
            names: Vec::new(),
        })
    }

    /// Convert WASM value type to byte representation
    fn value_type_to_byte(&self, value_type: WasmValueType) -> u8 {
        match value_type {
            WasmValueType::I32 => 0x7F,
            WasmValueType::I64 => 0x7E,
            WasmValueType::F32 => 0x7D,
            WasmValueType::F64 => 0x7C,
            WasmValueType::V128 => 0x7B,
            WasmValueType::FuncRef => 0x70,
            WasmValueType::ExternRef => 0x6F,
        }
    }

    /// Encode string with length prefix
    fn encode_string(&self, s: &str, bytecode: &mut Vec<u8>) {
        let bytes = s.as_bytes();
        self.encode_u32(bytes.len() as u32, bytecode);
        bytecode.extend_from_slice(bytes);
    }
}

impl CodegenContext {
    fn new() -> Self {
        CodegenContext {
            type_indices: HashMap::new(),
            function_indices: HashMap::new(),
            global_indices: HashMap::new(),
            local_indices: HashMap::new(),
            current_function_locals: Vec::new(),
            label_stack: Vec::new(),
            next_label: 0,
        }
    }
}

impl Default for WasmCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    UnsupportedFeature(String),
    InvalidModule(String),
    OptimizationError(String),
    AssemblyError(String),
}
impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {msg}"),
            CodegenError::InvalidModule(msg) => write!(f, "Invalid module: {msg}"),
            CodegenError::OptimizationError(msg) => write!(f, "Optimization error: {msg}"),
            CodegenError::AssemblyError(msg) => write!(f, "Assembly error: {msg}"),
        }
    }
}

impl std::error::Error for CodegenError {}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{
        ExportDeclaration, Expression, ImportDeclaration, ImportItem, Module, NodeId, Statement,
    };
    use mir_types::{ContentHash, TypeHash, Value};
    use std::collections::VecDeque;

    fn create_test_module(name: &str) -> Module {
        Module {
            id: NodeId::new(1),
            name: name.to_string(),
            imports: vec![],
            exports: vec![],
            statements: vec![],
            type_hash: TypeHash::new(ContentHash::new(name.as_bytes())),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }

    fn create_simple_function_module() -> Module {
        Module {
            id: NodeId::new(1),
            name: "test_module".to_string(),
            imports: vec![],
            exports: vec![ExportDeclaration {
                name: "add".to_string(),
                is_type: false,
                exported_hash: ContentHash::new(b"add"),
                visibility: mir_ast::ExportVisibility::Public,
                compatibility_info: mir_ast::CompatibilityInfo {
                    version: "1.0.0".to_string(),
                    breaking_changes: vec![],
                    deprecated_features: vec![],
                    migration_hints: vec![],
                },
            }],
            statements: vec![Statement::FunctionDeclaration {
                id: NodeId::new(2),
                name: "add".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Expression {
                    id: NodeId::new(3),
                    expression: Expression::Literal {
                        id: NodeId::new(4),
                        value: Value::I32(42),
                    },
                }],
            }],
            type_hash: TypeHash::new(ContentHash::new(b"test_module")),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: vec![],
            },
            message_queue: VecDeque::new(),
        }
    }

    #[test]
    fn test_wasm_code_generator_creation() {
        let generator = WasmCodeGenerator::new();

        // Verify default settings
        assert!(matches!(
            generator.optimization_level,
            OptimizationLevel::Speed
        ));
        assert!(generator.debug_info);
        assert!(generator.source_maps);
    }

    #[test]
    fn test_wasm_code_generator_configuration() {
        let mut generator = WasmCodeGenerator::new();

        generator.set_optimization_level(OptimizationLevel::Size);
        generator.enable_debug_info(false);
        generator.enable_source_maps(false);

        assert!(matches!(
            generator.optimization_level,
            OptimizationLevel::Size
        ));
        assert!(!generator.debug_info);
        assert!(!generator.source_maps);
    }

    #[test]
    fn test_empty_module_generation() {
        let generator = WasmCodeGenerator::new();
        let module = create_test_module("empty");

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();

        // Should have valid WASM header
        assert!(wasm_module.bytecode.len() >= 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&wasm_module.bytecode[4..8], &[0x01, 0x00, 0x00, 0x00]);

        // Should have basic structure
        assert!(wasm_module.functions.is_empty());
        assert!(wasm_module.imports.is_empty());
        assert!(wasm_module.exports.is_empty());
        assert!(wasm_module.memory.is_some());
    }

    #[test]
    fn test_simple_function_generation() {
        let generator = WasmCodeGenerator::new();
        let module = create_simple_function_module();

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();

        // Should have one function
        assert_eq!(wasm_module.functions.len(), 1);
        assert_eq!(wasm_module.functions[0].name, Some("add".to_string()));
        assert_eq!(wasm_module.functions[0].locals.len(), 2); // Two parameters
        assert!(!wasm_module.functions[0].body.is_empty());

        // Should have one export
        assert_eq!(wasm_module.exports.len(), 1);
        assert_eq!(wasm_module.exports[0].name, "add");
        assert!(matches!(
            wasm_module.exports[0].kind,
            WasmExportKind::Function
        ));
    }

    #[test]
    fn test_module_with_imports() {
        let mut module = create_test_module("with_imports");
        module.imports = vec![ImportDeclaration {
            module_path: "env".to_string(),
            module_hash: Some(ContentHash::new(b"env")),
            imported_items: vec![ImportItem {
                name: "print".to_string(),
                alias: None,
                is_type: false,
                expected_hash: Some(ContentHash::new(b"print")),
                compatibility_version: Some("1.0.0".to_string()),
            }],
            import_hash: ContentHash::new(b"env_import"),
        }];

        let generator = WasmCodeGenerator::new();
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();

        // Should have one import
        assert_eq!(wasm_module.imports.len(), 1);
        assert_eq!(wasm_module.imports[0].module, "env");
        assert_eq!(wasm_module.imports[0].name, "print");
        assert!(matches!(
            wasm_module.imports[0].kind,
            WasmImportKind::Function(_)
        ));
    }

    #[test]
    fn test_literal_generation() {
        let generator = WasmCodeGenerator::new();
        let mut bytecode = Vec::new();

        // Test i32 literal
        generator
            .generate_literal(&Value::I32(42), &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x41); // i32.const

        bytecode.clear();

        // Test i64 literal
        generator
            .generate_literal(&Value::I64(123), &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x42); // i64.const

        bytecode.clear();

        // Test f32 literal
        generator
            .generate_literal(&Value::F32(std::f32::consts::PI), &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x43); // f32.const

        bytecode.clear();

        // Test f64 literal
        generator
            .generate_literal(&Value::F64(std::f64::consts::E), &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x44); // f64.const

        bytecode.clear();

        // Test boolean literal
        generator
            .generate_literal(&Value::Bool(true), &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x41); // i32.const
        assert_eq!(bytecode[1], 1); // true as 1

        bytecode.clear();

        // Test null literal
        generator
            .generate_literal(&Value::Null, &mut bytecode)
            .unwrap();
        assert_eq!(bytecode[0], 0x41); // i32.const
        assert_eq!(bytecode[1], 0); // null as 0
    }

    #[test]
    fn test_leb128_encoding() {
        let generator = WasmCodeGenerator::new();
        let mut bytecode = Vec::new();

        // Test small positive number
        generator.encode_u32(42, &mut bytecode);
        assert_eq!(bytecode, vec![42]);

        bytecode.clear();

        // Test larger number requiring multiple bytes
        generator.encode_u32(300, &mut bytecode);
        assert_eq!(bytecode, vec![0xAC, 0x02]); // 300 in LEB128

        bytecode.clear();

        // Test signed encoding
        generator.encode_i32(-1, &mut bytecode);
        assert_eq!(bytecode, vec![0x7F]); // -1 in signed LEB128

        bytecode.clear();

        // Test signed positive
        generator.encode_i32(42, &mut bytecode);
        assert_eq!(bytecode, vec![42]);
    }

    #[test]
    fn test_value_type_conversion() {
        let generator = WasmCodeGenerator::new();

        assert_eq!(generator.value_type_to_byte(WasmValueType::I32), 0x7F);
        assert_eq!(generator.value_type_to_byte(WasmValueType::I64), 0x7E);
        assert_eq!(generator.value_type_to_byte(WasmValueType::F32), 0x7D);
        assert_eq!(generator.value_type_to_byte(WasmValueType::F64), 0x7C);
        assert_eq!(generator.value_type_to_byte(WasmValueType::V128), 0x7B);
        assert_eq!(generator.value_type_to_byte(WasmValueType::FuncRef), 0x70);
        assert_eq!(generator.value_type_to_byte(WasmValueType::ExternRef), 0x6F);
    }

    #[test]
    fn test_string_encoding() {
        let generator = WasmCodeGenerator::new();
        let mut bytecode = Vec::new();

        generator.encode_string("hello", &mut bytecode);

        // Should start with length (5)
        assert_eq!(bytecode[0], 5);
        // Followed by string bytes
        assert_eq!(&bytecode[1..6], b"hello");
    }

    #[test]
    fn test_function_type_generation() {
        let generator = WasmCodeGenerator::new();
        let context = CodegenContext::new();

        let result = generator.generate_types(&context);
        assert!(result.is_ok());

        let types = result.unwrap();

        // Should have at least basic function types
        assert!(!types.is_empty());

        // First type should be void -> void
        assert!(types[0].params.is_empty());
        assert!(types[0].results.is_empty());

        // Second type should be i32 -> i32
        assert_eq!(types[1].params, vec![WasmValueType::I32]);
        assert_eq!(types[1].results, vec![WasmValueType::I32]);
    }

    #[test]
    fn test_memory_generation() {
        let generator = WasmCodeGenerator::new();
        let module = create_test_module("memory_test");

        let result = generator.generate_memory(&module);
        assert!(result.is_ok());

        let memory = result.unwrap();
        assert!(memory.is_some());

        let mem = memory.unwrap();
        assert_eq!(mem.memory_type.limits.min, 1);
        assert_eq!(mem.memory_type.limits.max, Some(16));
        assert_eq!(mem.name, Some("memory".to_string()));
    }

    #[test]
    fn test_section_encoding() {
        let generator = WasmCodeGenerator::new();

        // Test type section encoding
        let types = vec![WasmFunctionType {
            params: vec![WasmValueType::I32],
            results: vec![WasmValueType::I32],
        }];

        let result = generator.encode_type_section(&types);
        assert!(result.is_ok());

        let section = result.unwrap();
        assert!(!section.is_empty());

        // Should start with count (1)
        assert_eq!(section[0], 1);
        // Followed by function type marker
        assert_eq!(section[1], 0x60);
    }

    #[test]
    fn test_export_section_encoding() {
        let generator = WasmCodeGenerator::new();

        let exports = vec![WasmExport {
            name: "test_func".to_string(),
            kind: WasmExportKind::Function,
            index: 0,
        }];

        let result = generator.encode_export_section(&exports);
        assert!(result.is_ok());

        let section = result.unwrap();
        assert!(!section.is_empty());

        // Should start with count (1)
        assert_eq!(section[0], 1);
    }

    #[test]
    fn test_optimization_integration() {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(OptimizationLevel::Size);

        let module = create_simple_function_module();
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();

        // Should still have valid WASM
        assert!(wasm_module.bytecode.len() >= 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }

    #[test]
    fn test_debug_info_generation() {
        let mut generator = WasmCodeGenerator::new();
        generator.enable_debug_info(true);

        let module = create_simple_function_module();
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();
        assert!(wasm_module.debug_info.is_some());

        let debug_info = wasm_module.debug_info.unwrap();
        // Debug info structure should be initialized
        assert!(debug_info.dwarf_sections.is_empty()); // Placeholder implementation
        assert!(debug_info.function_names.is_empty());
        assert!(debug_info.local_names.is_empty());
    }

    #[test]
    fn test_source_map_generation() {
        let mut generator = WasmCodeGenerator::new();
        generator.enable_source_maps(true);

        let module = create_simple_function_module();
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let wasm_module = result.unwrap();
        assert!(wasm_module.source_map.is_some());

        let source_map = wasm_module.source_map.unwrap();
        assert_eq!(source_map.version, 3);
        assert!(source_map.sources.is_empty()); // Placeholder implementation
        assert!(source_map.mappings.is_empty());
        assert!(source_map.names.is_empty());
    }

    #[test]
    fn test_complex_expression_generation() {
        let generator = WasmCodeGenerator::new();
        let mut context = CodegenContext::new();
        context.local_indices.insert("x".to_string(), 0);
        context.function_indices.insert("add".to_string(), 0);

        let mut bytecode = Vec::new();

        // Test identifier expression
        let expr = Expression::Identifier {
            id: NodeId::new(1),
            name: "x".to_string(),
        };

        let result = generator.generate_expression(&expr, &mut bytecode, &context);
        assert!(result.is_ok());
        assert_eq!(bytecode[0], 0x20); // local.get
        assert_eq!(bytecode[1], 0); // local index 0

        bytecode.clear();

        // Test function call expression
        let call_expr = Expression::FunctionCall {
            id: NodeId::new(2),
            function: Box::new(Expression::Identifier {
                id: NodeId::new(3),
                name: "add".to_string(),
            }),
            arguments: vec![
                Expression::Literal {
                    id: NodeId::new(4),
                    value: Value::I32(1),
                },
                Expression::Literal {
                    id: NodeId::new(5),
                    value: Value::I32(2),
                },
            ],
        };

        let result = generator.generate_expression(&call_expr, &mut bytecode, &context);
        assert!(result.is_ok());

        // Should generate: i32.const 1, i32.const 2, call 0
        assert!(bytecode.contains(&0x41)); // i32.const
        assert!(bytecode.contains(&0x10)); // call
    }

    #[test]
    fn test_variable_declaration_statement() {
        let generator = WasmCodeGenerator::new();
        let mut context = CodegenContext::new();
        let mut bytecode = Vec::new();

        let stmt = Statement::VariableDeclaration {
            id: NodeId::new(1),
            name: "x".to_string(),
            value: Some(Expression::Literal {
                id: NodeId::new(2),
                value: Value::I32(42),
            }),
        };

        let result = generator.generate_statement(&stmt, &mut bytecode, &mut context);
        assert!(result.is_ok());

        // Should have allocated a local variable
        assert_eq!(context.local_indices.get("x"), Some(&0));
        assert_eq!(context.current_function_locals.len(), 1);
        assert_eq!(context.current_function_locals[0], WasmValueType::I32);

        // Should generate initialization code
        assert!(bytecode.contains(&0x41)); // i32.const
        assert!(bytecode.contains(&0x21)); // local.set
    }

    #[test]
    fn test_error_handling() {
        let generator = WasmCodeGenerator::new();
        let context = CodegenContext::new();
        let mut bytecode = Vec::new();

        // Test undefined variable error
        let expr = Expression::Identifier {
            id: NodeId::new(1),
            name: "undefined_var".to_string(),
        };

        let result = generator.generate_expression(&expr, &mut bytecode, &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodegenError::InvalidModule(_)
        ));

        // Test undefined function error
        let call_expr = Expression::FunctionCall {
            id: NodeId::new(2),
            function: Box::new(Expression::Identifier {
                id: NodeId::new(3),
                name: "undefined_func".to_string(),
            }),
            arguments: vec![],
        };

        let result = generator.generate_expression(&call_expr, &mut bytecode, &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodegenError::InvalidModule(_)
        ));
    }

    #[test]
    fn test_wasm_function_struct() {
        let func = WasmFunction {
            type_index: 0,
            locals: vec![WasmValueType::I32, WasmValueType::F64],
            body: vec![0x41, 0x2A, 0x0F], // i32.const 42, return
            name: Some("test_function".to_string()),
            source_location: Some(SourceLocation {
                file: "test.mir".to_string(),
                line: 10,
                column: 5,
            }),
        };

        assert_eq!(func.type_index, 0);
        assert_eq!(func.locals.len(), 2);
        assert_eq!(func.body.len(), 3);
        assert_eq!(func.name, Some("test_function".to_string()));
        assert!(func.source_location.is_some());

        let source_loc = func.source_location.unwrap();
        assert_eq!(source_loc.file, "test.mir");
        assert_eq!(source_loc.line, 10);
        assert_eq!(source_loc.column, 5);
    }

    #[test]
    fn test_codegen_context() {
        let mut context = CodegenContext::new();

        // Test initial state
        assert!(context.type_indices.is_empty());
        assert!(context.function_indices.is_empty());
        assert!(context.global_indices.is_empty());
        assert!(context.local_indices.is_empty());
        assert!(context.current_function_locals.is_empty());
        assert!(context.label_stack.is_empty());
        assert_eq!(context.next_label, 0);

        // Test adding indices
        context.function_indices.insert("test_func".to_string(), 0);
        context.local_indices.insert("test_var".to_string(), 1);
        context.current_function_locals.push(WasmValueType::I32);

        assert_eq!(context.function_indices.get("test_func"), Some(&0));
        assert_eq!(context.local_indices.get("test_var"), Some(&1));
        assert_eq!(context.current_function_locals.len(), 1);
    }

    #[test]
    fn test_wasm_module_structure() {
        let wasm_module = WasmModule {
            bytecode: vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00],
            exports: vec![],
            imports: vec![],
            functions: vec![],
            types: vec![],
            globals: vec![],
            memory: None,
            debug_info: None,
            source_map: None,
        };

        assert_eq!(wasm_module.bytecode.len(), 8);
        assert_eq!(&wasm_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert!(wasm_module.exports.is_empty());
        assert!(wasm_module.imports.is_empty());
        assert!(wasm_module.functions.is_empty());
        assert!(wasm_module.debug_info.is_none());
        assert!(wasm_module.source_map.is_none());
    }

    #[test]
    fn test_codegen_error_display() {
        let error1 = CodegenError::UnsupportedFeature("test feature".to_string());
        assert_eq!(format!("{error1}"), "Unsupported feature: test feature");

        let error2 = CodegenError::InvalidModule("test module".to_string());
        assert_eq!(format!("{error2}"), "Invalid module: test module");

        let error3 = CodegenError::OptimizationError("test optimization".to_string());
        assert_eq!(
            format!("{error3}"),
            "Optimization error: test optimization"
        );

        let error4 = CodegenError::AssemblyError("test assembly".to_string());
        assert_eq!(format!("{error4}"), "Assembly error: test assembly");
    }

    #[test]
    fn test_wasm_generation_with_circular_function_calls() {
        let mut module = create_test_module("circular_module");
        
        // Add functions that call each other
        module.statements = vec![
            Statement::FunctionDeclaration {
                id: NodeId::new(1),
                name: "func_a".to_string(),
                parameters: vec![],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(2),
                        expression: Expression::FunctionCall {
                            id: NodeId::new(3),
                            function: Box::new(Expression::Identifier {
                                id: NodeId::new(4),
                                name: "func_b".to_string(),
                            }),
                            arguments: vec![],
                        },
                    },
                ],
            },
            Statement::FunctionDeclaration {
                id: NodeId::new(5),
                name: "func_b".to_string(),
                parameters: vec![],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(6),
                        expression: Expression::FunctionCall {
                            id: NodeId::new(7),
                            function: Box::new(Expression::Identifier {
                                id: NodeId::new(8),
                                name: "func_a".to_string(),
                            }),
                            arguments: vec![],
                        },
                    },
                ],
            },
        ];

        let generator = WasmCodeGenerator::new();
        let result = generator.generate(&module);
        
        // Should handle circular calls without infinite loops
        assert!(result.is_ok());
        let wasm_module = result.unwrap();
        assert_eq!(wasm_module.functions.len(), 2);
    }

    #[test]
    fn test_wasm_generation_with_deeply_nested_expressions() {
        let mut module = create_test_module("nested_module");
        
        // Create deeply nested function calls
        let mut nested_expr = Expression::Literal {
            id: NodeId::new(100),
            value: Value::I32(1),
        };
        
        for i in 0..10 {
            nested_expr = Expression::FunctionCall {
                id: NodeId::new(101 + i),
                function: Box::new(Expression::Identifier {
                    id: NodeId::new(201 + i),
                    name: "identity".to_string(),
                }),
                arguments: vec![nested_expr],
            };
        }

        module.statements = vec![
            Statement::FunctionDeclaration {
                id: NodeId::new(1),
                name: "identity".to_string(),
                parameters: vec!["x".to_string()],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(2),
                        expression: Expression::Identifier {
                            id: NodeId::new(3),
                            name: "x".to_string(),
                        },
                    },
                ],
            },
            Statement::FunctionDeclaration {
                id: NodeId::new(4),
                name: "nested_test".to_string(),
                parameters: vec![],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(5),
                        expression: nested_expr,
                    },
                ],
            },
        ];

        let generator = WasmCodeGenerator::new();
        let result = generator.generate(&module);
        
        // Should handle deeply nested expressions
        assert!(result.is_ok());
        let wasm_module = result.unwrap();
        assert_eq!(wasm_module.functions.len(), 2);
    }

    #[test]
    fn test_wasm_generation_with_large_constants() {
        let generator = WasmCodeGenerator::new();
        let mut bytecode = Vec::new();

        // Test with maximum values
        generator.generate_literal(&Value::I32(i32::MAX), &mut bytecode).unwrap();
        assert_eq!(bytecode[0], 0x41); // i32.const
        
        bytecode.clear();
        generator.generate_literal(&Value::I32(i32::MIN), &mut bytecode).unwrap();
        assert_eq!(bytecode[0], 0x41); // i32.const
        
        bytecode.clear();
        generator.generate_literal(&Value::I64(i64::MAX), &mut bytecode).unwrap();
        assert_eq!(bytecode[0], 0x42); // i64.const
        
        bytecode.clear();
        generator.generate_literal(&Value::I64(i64::MIN), &mut bytecode).unwrap();
        assert_eq!(bytecode[0], 0x42); // i64.const
    }

    #[test]
    fn test_wasm_generation_with_unicode_strings() {
        let generator = WasmCodeGenerator::new();
        let mut bytecode = Vec::new();

        // Test with Unicode strings
        let unicode_string = "Hello, 世界! 🦀";
        generator.generate_literal(&Value::String(unicode_string.to_string()), &mut bytecode).unwrap();
        
        assert_eq!(bytecode[0], 0x41); // i32.const (string length placeholder)
        // In a full implementation, this would handle Unicode properly
    }

    #[test]
    fn test_wasm_generation_memory_management() {
        let generator = WasmCodeGenerator::new();
        let module = create_test_module("memory_test");
        
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let wasm_module = result.unwrap();
        assert!(wasm_module.memory.is_some());
        
        let memory = wasm_module.memory.unwrap();
        assert!(memory.memory_type.limits.min > 0);
        assert!(memory.memory_type.limits.max.is_some());
        assert!(memory.memory_type.limits.max.unwrap() >= memory.memory_type.limits.min);
    }
}
