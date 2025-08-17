//! TypeScript type definition generation from MIR IR
//!
//! This module generates TypeScript type definitions (.d.ts files) from MIR modules,
//! providing type safety and IDE support for JavaScript consumers.

use mir_ast::{Module, Statement, Expression};
use mir_types::Value;
use std::collections::HashMap;

/// TypeScript type definition generator
pub struct TypeScriptGenerator {
    emit_comments: bool,
    emit_jsdoc: bool,
    strict_mode: bool,
    emit_runtime_types: bool,
}

/// TypeScript generation options
#[derive(Debug, Clone)]
pub struct TypeScriptOptions {
    pub emit_comments: bool,
    pub emit_jsdoc: bool,
    pub strict_mode: bool,
    pub emit_runtime_types: bool,
    pub module_resolution: ModuleResolution,
}

/// Module resolution strategies for TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleResolution {
    Node,
    Classic,
    Bundler,
}

/// Generated TypeScript output
#[derive(Debug, Clone)]
pub struct TypeScriptOutput {
    pub declarations: String,
    pub metadata: TypeScriptMetadata,
}

/// Metadata about generated TypeScript definitions
#[derive(Debug, Clone)]
pub struct TypeScriptMetadata {
    pub module_name: String,
    pub exported_types: Vec<String>,
    pub exported_values: Vec<String>,
    pub imported_modules: Vec<String>,
    pub has_generics: bool,
    pub has_unions: bool,
    pub has_functions: bool,
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptGenerator {
    /// Create a new TypeScript generator with default settings
    pub fn new() -> Self {
        TypeScriptGenerator {
            emit_comments: true,
            emit_jsdoc: true,
            strict_mode: true,
            emit_runtime_types: false,
        }
    }

    /// Create a new generator with custom options
    pub fn with_options(options: TypeScriptOptions) -> Self {
        TypeScriptGenerator {
            emit_comments: options.emit_comments,
            emit_jsdoc: options.emit_jsdoc,
            strict_mode: options.strict_mode,
            emit_runtime_types: options.emit_runtime_types,
        }
    }

    /// Generate TypeScript type definitions from a MIR module
    pub fn generate_types(&self, module: &Module) -> Result<TypeScriptOutput, TypeScriptError> {
        let mut context = TypeGenContext::new(module);
        
        let mut declarations = String::new();
        
        // Generate file header
        declarations.push_str(&self.generate_header(&context)?);
        
        // Generate imports
        declarations.push_str(&self.generate_import_types(module, &mut context)?);
        
        // Generate type definitions from module content
        declarations.push_str(&self.generate_module_types(module, &mut context)?);
        
        // Generate export declarations
        declarations.push_str(&self.generate_export_types(module, &mut context)?);
        
        let metadata = TypeScriptMetadata {
            module_name: module.name.clone(),
            exported_types: context.exported_types,
            exported_values: context.exported_values,
            imported_modules: context.imported_modules,
            has_generics: context.has_generics,
            has_unions: context.has_unions,
            has_functions: context.has_functions,
        };
        
        Ok(TypeScriptOutput {
            declarations,
            metadata,
        })
    }

    /// Generate file header with module information
    fn generate_header(&self, context: &TypeGenContext) -> Result<String, TypeScriptError> {
        let mut header = String::new();
        
        if self.emit_comments {
            header.push_str(&format!(
                "/**\n * TypeScript definitions for MIR module: {}\n * Generated at: {}\n */\n\n",
                context.module_name,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }
        
        // Add MIR runtime type definitions
        header.push_str(&self.generate_mir_runtime_types()?);
        
        Ok(header)
    }

    /// Generate MIR runtime type definitions
    fn generate_mir_runtime_types(&self) -> Result<String, TypeScriptError> {
        let mut types = String::new();
        
        if self.emit_runtime_types {
            types.push_str(r#"
// MIR Runtime Types
declare namespace MIR {
  type ContentHash = string;
  type TypeHash = string;
  type SchemaVersion = string;
  
  interface Value {
    readonly typeHash: TypeHash;
    readonly schemaVersion: SchemaVersion;
  }
  
  interface ScalarValue<T> extends Value {
    readonly value: T;
  }
  
  interface StructValue extends Value {
    readonly fields: Record<string, Value>;
  }
  
  interface ArrayValue extends Value {
    readonly elements: Value[];
    readonly elementType: TypeHash;
  }
  
  interface FunctionValue extends Value {
    readonly functionId: number;
    (...args: Value[]): Value;
  }
  
  interface ClosureValue extends Value {
    readonly closureId: number;
    readonly capturedValues: Record<string, Value>;
    (...args: Value[]): Value;
  }
  
  // OpenTelemetry Integration Types
  interface Span {
    setAttributes(attributes: Record<string, any>): void;
    addEvent(name: string, attributes?: Record<string, any>): void;
    end(): void;
    recordException(exception: Error): void;
  }
  
  interface TraceContext {
    readonly traceId: string;
    readonly spanId: string;
  }
  
  type MetricType = 'counter' | 'gauge' | 'histogram';
  type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';
}

"#);
        }
        
        Ok(types)
    }

    /// Generate import type declarations
    fn generate_import_types(&self, module: &Module, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        let mut imports = String::new();
        
        for import in &module.imports {
            context.imported_modules.push(import.module_path.clone());
            
            // Generate type-only imports for TypeScript
            let type_imports: Vec<String> = import.imported_items.iter()
                .filter(|item| item.is_type)
                .map(|item| {
                    if let Some(alias) = &item.alias {
                        format!("type {} as {}", item.name, alias)
                    } else {
                        format!("type {}", item.name)
                    }
                })
                .collect();
            
            let value_imports: Vec<String> = import.imported_items.iter()
                .filter(|item| !item.is_type)
                .map(|item| {
                    if let Some(alias) = &item.alias {
                        format!("{} as {}", item.name, alias)
                    } else {
                        item.name.clone()
                    }
                })
                .collect();
            
            if !type_imports.is_empty() {
                imports.push_str(&format!(
                    "import {{ {} }} from '{}';\n",
                    type_imports.join(", "),
                    import.module_path
                ));
            }
            
            if !value_imports.is_empty() {
                imports.push_str(&format!(
                    "import {{ {} }} from '{}';\n",
                    value_imports.join(", "),
                    import.module_path
                ));
            }
        }
        
        if !imports.is_empty() {
            imports.push('\n');
        }
        
        Ok(imports)
    }

    /// Generate type definitions from module statements
    fn generate_module_types(&self, module: &Module, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        let mut types = String::new();
        
        for statement in &module.statements {
            types.push_str(&self.generate_statement_types(statement, context)?);
        }
        
        Ok(types)
    }

    /// Generate type definitions for a statement
    fn generate_statement_types(&self, statement: &Statement, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        match statement {
            Statement::VariableDeclaration { name, value, .. } => {
                let ts_type = if let Some(val_expr) = value {
                    self.infer_expression_type(val_expr, context)?
                } else {
                    "unknown".to_string()
                };
                
                Ok(format!("declare const {}: {};\n", name, ts_type))
            }
            
            Statement::FunctionDeclaration { name, parameters, .. } => {
                context.has_functions = true;
                
                let param_types = parameters.iter()
                    .map(|param| format!("{}: unknown", param))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let _func_type = format!("({}) => unknown", param_types);
                
                if self.emit_jsdoc {
                    Ok(format!(
                        "/**\n * Function: {}\n */\ndeclare function {}({});\n",
                        name, name, param_types
                    ))
                } else {
                    Ok(format!("declare function {}({});\n", name, param_types))
                }
            }
            
            _ => Ok(String::new()) // Other statements don't generate type declarations
        }
    }

    /// Infer TypeScript type from an expression
    fn infer_expression_type(&self, expression: &Expression, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        match expression {
            Expression::Literal { value, .. } => {
                self.value_to_typescript_type(value, context)
            }
            
            Expression::Identifier { .. } => {
                Ok("unknown".to_string())
            }
            
            Expression::FunctionCall { .. } => {
                Ok("unknown".to_string())
            }
            
            Expression::CreateSpan { .. } => {
                Ok("MIR.Span".to_string())
            }
            
            Expression::GetTraceContext { .. } => {
                Ok("MIR.TraceContext".to_string())
            }
            
            _ => Ok("unknown".to_string())
        }
    }

    /// Convert a MIR Value to a TypeScript type
    fn value_to_typescript_type(&self, value: &Value, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        match value {
            Value::I32(_) | Value::I64(_) | Value::U32(_) | Value::U64(_) => Ok("number".to_string()),
            Value::I128(_) | Value::U128(_) => Ok("bigint".to_string()),
            Value::F32(_) | Value::F64(_) => Ok("number".to_string()),
            Value::Bool(_) => Ok("boolean".to_string()),
            Value::String(_) => Ok("string".to_string()),
            Value::Null => Ok("null".to_string()),
            
            Value::Struct(fields) => {
                let field_types: Result<Vec<String>, TypeScriptError> = fields.iter()
                    .map(|(name, val)| {
                        let field_type = self.value_to_typescript_type(val, context)?;
                        Ok(format!("{}: {}", name, field_type))
                    })
                    .collect();
                
                Ok(format!("{{ {} }}", field_types?.join("; ")))
            }
            
            Value::Array(elements) => {
                if elements.is_empty() {
                    Ok("unknown[]".to_string())
                } else {
                    // Infer array element type from first element
                    let element_type = self.value_to_typescript_type(&elements[0], context)?;
                    Ok(format!("{}[]", element_type))
                }
            }
            
            Value::Union { .. } => {
                Ok("MIR.UnionValue".to_string())
            }
            
            Value::Record(_) => {
                Ok("MIR.RecordValue".to_string())
            }
            
            Value::Undefined => {
                Ok("undefined".to_string())
            }
        }
    }

    /// Generate export type declarations
    fn generate_export_types(&self, module: &Module, context: &mut TypeGenContext) -> Result<String, TypeScriptError> {
        let mut exports = String::new();
        
        for export in &module.exports {
            if export.is_type {
                context.exported_types.push(export.name.clone());
                exports.push_str(&format!("export type {};\n", export.name));
            } else {
                context.exported_values.push(export.name.clone());
                exports.push_str(&format!("export declare const {}: unknown;\n", export.name));
            }
        }
        
        Ok(exports)
    }

    /// Set whether to emit comments
    pub fn set_emit_comments(&mut self, emit: bool) {
        self.emit_comments = emit;
    }

    /// Set whether to emit JSDoc comments
    pub fn set_emit_jsdoc(&mut self, emit: bool) {
        self.emit_jsdoc = emit;
    }

    /// Set strict mode
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Set whether to emit runtime types
    pub fn set_emit_runtime_types(&mut self, emit: bool) {
        self.emit_runtime_types = emit;
    }
}

/// Context for TypeScript type generation
struct TypeGenContext {
    module_name: String,
    exported_types: Vec<String>,
    exported_values: Vec<String>,
    imported_modules: Vec<String>,
    has_generics: bool,
    has_unions: bool,
    has_functions: bool,
    #[allow(dead_code)]
    type_aliases: HashMap<String, String>,
}

impl TypeGenContext {
    fn new(module: &Module) -> Self {
        TypeGenContext {
            module_name: module.name.clone(),
            exported_types: Vec::new(),
            exported_values: Vec::new(),
            imported_modules: Vec::new(),
            has_generics: false,
            has_unions: false,
            has_functions: false,
            type_aliases: HashMap::new(),
        }
    }
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        TypeScriptOptions {
            emit_comments: true,
            emit_jsdoc: true,
            strict_mode: true,
            emit_runtime_types: false,
            module_resolution: ModuleResolution::Node,
        }
    }
}

/// Errors that can occur during TypeScript generation
#[derive(Debug, Clone)]
pub enum TypeScriptError {
    UnsupportedType(String),
    InvalidModule(String),
    TypeInferenceError(String),
    GenerationError(String),
}

impl std::fmt::Display for TypeScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeScriptError::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
            TypeScriptError::InvalidModule(msg) => write!(f, "Invalid module: {}", msg),
            TypeScriptError::TypeInferenceError(msg) => write!(f, "Type inference error: {}", msg),
            TypeScriptError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
        }
    }
}

impl std::error::Error for TypeScriptError {}