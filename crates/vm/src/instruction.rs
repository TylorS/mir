//! VM instruction set that maps to WASM operations

use mir_types::{ContentHash, TypeHash, Value};
use serde::{Deserialize, Serialize};

/// VM instruction identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstructionId(u64);

impl InstructionId {
    pub fn new(id: u64) -> Self {
        InstructionId(id)
    }
}

/// VM instruction set that maps cleanly to WASM operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Control flow
    Nop,
    Block {
        instructions: Vec<Instruction>,
    },
    Loop {
        instructions: Vec<Instruction>,
    },
    If {
        condition: Box<Instruction>,
        then_branch: Box<Instruction>,
        else_branch: Option<Box<Instruction>>,
    },
    Branch {
        target: u32,
    },
    BranchIf {
        target: u32,
        condition: Box<Instruction>,
    },
    Return {
        value: Option<Box<Instruction>>,
    },

    // Function calls
    Call {
        function_id: u64,
        args: Vec<Instruction>,
    },
    CallIndirect {
        table_index: u32,
        type_index: u32,
        args: Vec<Instruction>,
    },

    // Local variables
    LocalGet {
        index: u32,
    },
    LocalSet {
        index: u32,
        value: Box<Instruction>,
    },
    LocalTee {
        index: u32,
        value: Box<Instruction>,
    },

    // Global variables
    GlobalGet {
        index: u32,
    },
    GlobalSet {
        index: u32,
        value: Box<Instruction>,
    },

    // Memory operations
    Load {
        offset: u32,
        align: u32,
    },
    Store {
        offset: u32,
        align: u32,
        value: Box<Instruction>,
    },
    MemorySize,
    MemoryGrow {
        pages: Box<Instruction>,
    },

    // Constants
    I32Const {
        value: i32,
    },
    I64Const {
        value: i64,
    },
    F32Const {
        value: f32,
    },
    F64Const {
        value: f64,
    },

    // Arithmetic operations (i32)
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    // Arithmetic operations (i64)
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,

    // Arithmetic operations (f32)
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,

    // Arithmetic operations (f64)
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,

    // Comparison operations (i32)
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    // Comparison operations (i64)
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    // Comparison operations (f32)
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    // Comparison operations (f64)
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    // Type conversions
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    // MIR-specific instructions
    TypeCheck {
        value: Box<Instruction>,
        expected_type: TypeHash,
    },
    TypeCast {
        value: Box<Instruction>,
        target_type: TypeHash,
    },
    StructGet {
        struct_value: Box<Instruction>,
        field_name: String,
    },
    StructSet {
        struct_value: Box<Instruction>,
        field_name: String,
        value: Box<Instruction>,
    },
    ArrayGet {
        array_value: Box<Instruction>,
        index: Box<Instruction>,
    },
    ArraySet {
        array_value: Box<Instruction>,
        index: Box<Instruction>,
        value: Box<Instruction>,
    },
    ArrayLen {
        array_value: Box<Instruction>,
    },

    // Hot-reloading specific
    StateSnapshot {
        module_hash: ContentHash,
    },
    StateRestore {
        snapshot_id: u64,
    },
    ModuleReload {
        old_hash: ContentHash,
        new_hash: ContentHash,
    },

    // Observability and OpenTelemetry
    SpanStart {
        name: String,
        attributes: Vec<(String, String)>,
    },
    SpanEnd,
    SpanCreate {
        name: String,
        attributes: Vec<(String, Value)>,
        parent_span: Option<Box<Instruction>>,
    },
    SpanSetAttribute {
        span: Box<Instruction>,
        key: String,
        value: Value,
    },
    SpanAddEvent {
        span: Box<Instruction>,
        name: String,
        attributes: Vec<(String, Value)>,
    },
    SpanSetStatus {
        span: Box<Instruction>,
        status: SpanStatus,
        message: Option<String>,
    },
    TraceContextGet,
    TraceContextSet {
        context: Box<Instruction>,
    },
    TraceContextPropagate {
        target_module: ContentHash,
        context: Box<Instruction>,
    },
    MetricRecord {
        metric_type: MetricType,
        name: String,
        value: Box<Instruction>,
        labels: Vec<(String, Value)>,
    },
    LogEvent {
        level: LogLevel,
        message: String,
        fields: Vec<(String, Value)>,
    },
}

/// Log levels for observability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Span status for OpenTelemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

/// Metric types for OpenTelemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Instruction sequence with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSequence {
    pub instructions: Vec<Instruction>,
    pub locals: Vec<LocalVariable>,
    pub source_map: Option<SourceMap>,
}

/// Local variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVariable {
    pub index: u32,
    pub name: Option<String>,
    pub type_hash: TypeHash,
}

/// Source mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub file: String,
    pub mappings: Vec<SourceMapping>,
}

/// Individual source mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    pub instruction_index: usize,
    pub line: u32,
    pub column: u32,
}
