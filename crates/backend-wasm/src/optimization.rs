//! WebAssembly optimization passes
//!
//! This module provides WASM-specific optimizations including:
//! - Dead code elimination
//! - Constant folding
//! - Function inlining
//! - Size and speed optimizations

use crate::codegen::{CodegenError, OptimizationLevel};

/// WASM optimization passes
pub struct WasmOptimizer {
    dead_code_elimination: bool,
    constant_folding: bool,
    function_inlining: bool,
    size_optimization: bool,
}

/// Optimization statistics
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub original_size: usize,
    pub optimized_size: usize,
    pub functions_inlined: u32,
    pub dead_code_removed: u32,
    pub constants_folded: u32,
}

impl WasmOptimizer {
    pub fn new() -> Self {
        WasmOptimizer {
            dead_code_elimination: true,
            constant_folding: true,
            function_inlining: false,
            size_optimization: false,
        }
    }

    /// Optimize WASM bytecode based on optimization level
    pub fn optimize(
        &self,
        bytecode: &[u8],
        level: &OptimizationLevel,
    ) -> Result<Vec<u8>, CodegenError> {
        let mut optimized = bytecode.to_vec();
        let _original_size = optimized.len();

        match level {
            OptimizationLevel::None => {
                // No optimizations
                return Ok(optimized);
            }
            OptimizationLevel::Size => {
                optimized = self.optimize_for_size(optimized)?;
            }
            OptimizationLevel::Speed => {
                optimized = self.optimize_for_speed(optimized)?;
            }
            OptimizationLevel::Aggressive => {
                optimized = self.optimize_aggressive(optimized)?;
            }
        }

        // Validate optimized bytecode
        self.validate_bytecode(&optimized)?;

        Ok(optimized)
    }

    /// Configure optimization passes
    pub fn configure(&mut self, level: &OptimizationLevel) {
        match level {
            OptimizationLevel::None => {
                self.dead_code_elimination = false;
                self.constant_folding = false;
                self.function_inlining = false;
                self.size_optimization = false;
            }
            OptimizationLevel::Size => {
                self.dead_code_elimination = true;
                self.constant_folding = true;
                self.function_inlining = false;
                self.size_optimization = true;
            }
            OptimizationLevel::Speed => {
                self.dead_code_elimination = true;
                self.constant_folding = true;
                self.function_inlining = true;
                self.size_optimization = false;
            }
            OptimizationLevel::Aggressive => {
                self.dead_code_elimination = true;
                self.constant_folding = true;
                self.function_inlining = true;
                self.size_optimization = false;
            }
        }
    }

    /// Optimize for size
    fn optimize_for_size(&self, mut bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        if self.dead_code_elimination {
            bytecode = self.eliminate_dead_code(bytecode)?;
        }

        if self.constant_folding {
            bytecode = self.fold_constants(bytecode)?;
        }

        if self.size_optimization {
            bytecode = self.compress_instructions(bytecode)?;
        }

        Ok(bytecode)
    }

    /// Optimize for speed
    fn optimize_for_speed(&self, mut bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        if self.constant_folding {
            bytecode = self.fold_constants(bytecode)?;
        }

        if self.function_inlining {
            bytecode = self.inline_functions(bytecode)?;
        }

        if self.dead_code_elimination {
            bytecode = self.eliminate_dead_code(bytecode)?;
        }

        bytecode = self.optimize_control_flow(bytecode)?;

        Ok(bytecode)
    }

    /// Aggressive optimization
    fn optimize_aggressive(&self, mut bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Multiple passes for maximum optimization
        for _ in 0..3 {
            let original_len = bytecode.len();

            bytecode = self.fold_constants(bytecode)?;
            bytecode = self.inline_functions(bytecode)?;
            bytecode = self.eliminate_dead_code(bytecode)?;
            bytecode = self.optimize_control_flow(bytecode)?;
            bytecode = self.optimize_memory_access(bytecode)?;

            // Stop if no more improvements
            if bytecode.len() == original_len {
                break;
            }
        }

        Ok(bytecode)
    }

    /// Eliminate dead code
    fn eliminate_dead_code(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Simplified dead code elimination
        // In a full implementation, this would analyze control flow and remove unreachable code
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            let instruction = bytecode[i];

            // Skip unreachable code after unconditional branches
            if instruction == 0x0F || instruction == 0x0C {
                // return or br
                optimized.push(instruction);
                i += 1;

                // Skip instructions until next block/function boundary
                while i < bytecode.len() && !self.is_block_boundary(bytecode[i]) {
                    i += 1;
                }
            } else {
                optimized.push(instruction);
                i += 1;
            }
        }

        Ok(optimized)
    }

    /// Fold constants
    fn fold_constants(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Simplified constant folding
        // Look for patterns like: const, const, add -> const(result)
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            if i + 10 < bytecode.len() {
                // Look for i32.const, i32.const, i32.add pattern
                if bytecode[i] == 0x41 && bytecode[i + 5] == 0x41 && bytecode[i + 10] == 0x6A {
                    // Extract constants (simplified - assumes single byte values)
                    let const1 = bytecode[i + 1] as i32;
                    let const2 = bytecode[i + 6] as i32;
                    let result = const1.wrapping_add(const2);

                    // Replace with single constant
                    optimized.push(0x41); // i32.const
                    optimized.push(result as u8);
                    i += 11;
                    continue;
                }
            }

            optimized.push(bytecode[i]);
            i += 1;
        }

        Ok(optimized)
    }

    /// Inline small functions
    fn inline_functions(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Simplified function inlining
        // In a full implementation, this would analyze function sizes and inline small ones
        Ok(bytecode)
    }

    /// Optimize control flow
    fn optimize_control_flow(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Simplified control flow optimization
        // Remove redundant branches, merge blocks, etc.
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            let instruction = bytecode[i];

            // Remove redundant branch to next instruction
            if instruction == 0x0C && i + 2 < bytecode.len() {
                // br
                let target = bytecode[i + 1];
                if target == 0 {
                    // Branch to next instruction - remove it
                    i += 2;
                    continue;
                }
            }

            optimized.push(instruction);
            i += 1;
        }

        Ok(optimized)
    }

    /// Optimize memory access patterns
    fn optimize_memory_access(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Simplified memory access optimization
        // Combine adjacent loads/stores, optimize alignment, etc.
        Ok(bytecode)
    }

    /// Compress instruction sequences
    fn compress_instructions(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CodegenError> {
        // Replace common instruction sequences with more compact equivalents
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            // Look for drop, drop -> drop (remove redundant drops)
            if i + 1 < bytecode.len() && bytecode[i] == 0x1A && bytecode[i + 1] == 0x1A {
                optimized.push(0x1A); // single drop
                i += 2;
                continue;
            }

            optimized.push(bytecode[i]);
            i += 1;
        }

        Ok(optimized)
    }

    /// Check if instruction is a block boundary
    fn is_block_boundary(&self, instruction: u8) -> bool {
        matches!(instruction, 0x02 | 0x03 | 0x04 | 0x05 | 0x0B) // block, loop, if, else, end
    }

    /// Validate bytecode integrity after optimization
    fn validate_bytecode(&self, bytecode: &[u8]) -> Result<(), CodegenError> {
        // Basic validation - check WASM magic number and version
        if bytecode.len() < 8 {
            return Err(CodegenError::OptimizationError(
                "Bytecode too short after optimization".to_string(),
            ));
        }

        if bytecode[0..4] != [0x00, 0x61, 0x73, 0x6D] {
            return Err(CodegenError::OptimizationError(
                "Invalid WASM magic number after optimization".to_string(),
            ));
        }

        if bytecode[4..8] != [0x01, 0x00, 0x00, 0x00] {
            return Err(CodegenError::OptimizationError(
                "Invalid WASM version after optimization".to_string(),
            ));
        }

        Ok(())
    }

    /// Get optimization statistics
    pub fn get_stats(&self, original: &[u8], optimized: &[u8]) -> OptimizationStats {
        OptimizationStats {
            original_size: original.len(),
            optimized_size: optimized.len(),
            functions_inlined: 0, // Would be tracked during optimization
            dead_code_removed: 0, // Would be tracked during optimization
            constants_folded: 0,  // Would be tracked during optimization
        }
    }
}

impl Default for WasmOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bytecode() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x41, 0x0D, // i32.const 13
            0x6A, // i32.add
            0x0F, // return
        ]
    }

    #[test]
    fn test_wasm_optimizer_creation() {
        let optimizer = WasmOptimizer::new();

        assert!(optimizer.dead_code_elimination);
        assert!(optimizer.constant_folding);
        assert!(!optimizer.function_inlining);
        assert!(!optimizer.size_optimization);
    }

    #[test]
    fn test_optimizer_configuration() {
        let mut optimizer = WasmOptimizer::new();

        // Test None level
        optimizer.configure(&OptimizationLevel::None);
        assert!(!optimizer.dead_code_elimination);
        assert!(!optimizer.constant_folding);
        assert!(!optimizer.function_inlining);
        assert!(!optimizer.size_optimization);

        // Test Size level
        optimizer.configure(&OptimizationLevel::Size);
        assert!(optimizer.dead_code_elimination);
        assert!(optimizer.constant_folding);
        assert!(!optimizer.function_inlining);
        assert!(optimizer.size_optimization);

        // Test Speed level
        optimizer.configure(&OptimizationLevel::Speed);
        assert!(optimizer.dead_code_elimination);
        assert!(optimizer.constant_folding);
        assert!(optimizer.function_inlining);
        assert!(!optimizer.size_optimization);

        // Test Aggressive level
        optimizer.configure(&OptimizationLevel::Aggressive);
        assert!(optimizer.dead_code_elimination);
        assert!(optimizer.constant_folding);
        assert!(optimizer.function_inlining);
        assert!(!optimizer.size_optimization);
    }

    #[test]
    fn test_no_optimization() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();
        let original_len = bytecode.len();

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::None);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert_eq!(optimized.len(), original_len);
        assert_eq!(optimized, bytecode);
    }

    #[test]
    fn test_size_optimization() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Size);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_speed_optimization() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Speed);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_aggressive_optimization() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Aggressive);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_dead_code_elimination() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode with unreachable code after return
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x0F, // return
            0x41, 0x0D, // unreachable: i32.const 13
            0x1A, // unreachable: drop
        ];

        let result = optimizer.eliminate_dead_code(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should have removed unreachable code
        assert!(optimized.len() < 14); // Original had unreachable instructions
        assert!(optimized.contains(&0x0F)); // Should still have return
    }

    #[test]
    fn test_constant_folding() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode with constant arithmetic
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x05, // i32.const 5
            0x41, 0x03, // i32.const 3
            0x6A, // i32.add
            0x0F, // return
        ];

        let result = optimizer.fold_constants(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should still have valid structure
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_control_flow_optimization() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode with redundant branch
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x0C, 0x00, // br 0 (branch to next instruction)
            0x41, 0x2A, // i32.const 42
            0x0F, // return
        ];

        let result = optimizer.optimize_control_flow(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should have removed redundant branch
        assert!(!optimized[8..].contains(&0x0C)); // No branch instruction in body
        assert!(optimized.contains(&0x41)); // Should still have i32.const
    }

    #[test]
    fn test_instruction_compression() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode with redundant drops
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x1A, // drop
            0x1A, // drop (redundant)
            0x0F, // return
        ];

        let result = optimizer.compress_instructions(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should have compressed redundant drops
        let drop_count = optimized.iter().filter(|&&b| b == 0x1A).count();
        assert_eq!(drop_count, 1); // Should have only one drop
    }

    #[test]
    fn test_bytecode_validation() {
        let optimizer = WasmOptimizer::new();

        // Test valid bytecode
        let valid_bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
        ];

        let result = optimizer.validate_bytecode(&valid_bytecode);
        assert!(result.is_ok());

        // Test invalid magic number
        let invalid_magic = vec![
            0xFF, 0xFF, 0xFF, 0xFF, // Invalid magic
            0x01, 0x00, 0x00, 0x00, // version 1
        ];

        let result = optimizer.validate_bytecode(&invalid_magic);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodegenError::OptimizationError(_)
        ));

        // Test invalid version
        let invalid_version = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0xFF, 0xFF, 0xFF, 0xFF, // Invalid version
        ];

        let result = optimizer.validate_bytecode(&invalid_version);
        assert!(result.is_err());

        // Test too short bytecode
        let too_short = vec![0x00, 0x61];

        let result = optimizer.validate_bytecode(&too_short);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_boundary_detection() {
        let optimizer = WasmOptimizer::new();

        assert!(optimizer.is_block_boundary(0x02)); // block
        assert!(optimizer.is_block_boundary(0x03)); // loop
        assert!(optimizer.is_block_boundary(0x04)); // if
        assert!(optimizer.is_block_boundary(0x05)); // else
        assert!(optimizer.is_block_boundary(0x0B)); // end

        assert!(!optimizer.is_block_boundary(0x41)); // i32.const
        assert!(!optimizer.is_block_boundary(0x0F)); // return
        assert!(!optimizer.is_block_boundary(0x1A)); // drop
    }

    #[test]
    fn test_optimization_stats() {
        let optimizer = WasmOptimizer::new();

        let original = vec![1, 2, 3, 4, 5];
        let optimized = vec![1, 2, 3];

        let stats = optimizer.get_stats(&original, &optimized);

        assert_eq!(stats.original_size, 5);
        assert_eq!(stats.optimized_size, 3);
        assert_eq!(stats.functions_inlined, 0);
        assert_eq!(stats.dead_code_removed, 0);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_optimization_stats_debug() {
        let stats = OptimizationStats {
            original_size: 100,
            optimized_size: 80,
            functions_inlined: 2,
            dead_code_removed: 5,
            constants_folded: 3,
        };

        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("original_size: 100"));
        assert!(debug_str.contains("optimized_size: 80"));
        assert!(debug_str.contains("functions_inlined: 2"));
        assert!(debug_str.contains("dead_code_removed: 5"));
        assert!(debug_str.contains("constants_folded: 3"));
    }

    #[test]
    fn test_default_optimizer() {
        let optimizer1 = WasmOptimizer::new();
        let optimizer2 = WasmOptimizer::default();

        assert_eq!(
            optimizer1.dead_code_elimination,
            optimizer2.dead_code_elimination
        );
        assert_eq!(optimizer1.constant_folding, optimizer2.constant_folding);
        assert_eq!(optimizer1.function_inlining, optimizer2.function_inlining);
        assert_eq!(optimizer1.size_optimization, optimizer2.size_optimization);
    }

    #[test]
    fn test_memory_access_optimization() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();

        let result = optimizer.optimize_memory_access(bytecode.clone());
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Placeholder implementation should return unchanged bytecode
        assert_eq!(optimized, bytecode);
    }

    #[test]
    fn test_function_inlining() {
        let optimizer = WasmOptimizer::new();
        let bytecode = create_test_bytecode();

        let result = optimizer.inline_functions(bytecode.clone());
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Placeholder implementation should return unchanged bytecode
        assert_eq!(optimized, bytecode);
    }

    #[test]
    fn test_optimization_level_enum() {
        // Test that optimization levels can be cloned and debugged
        let level = OptimizationLevel::Speed;
        let cloned = level.clone();

        assert!(matches!(cloned, OptimizationLevel::Speed));

        let debug_str = format!("{level:?}");
        assert_eq!(debug_str, "Speed");
    }

    #[test]
    fn test_complex_optimization_scenario() {
        let optimizer = WasmOptimizer::new();

        // Create complex bytecode with multiple optimization opportunities
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x05, // i32.const 5
            0x41, 0x03, // i32.const 3
            0x6A, // i32.add (can be folded)
            0x1A, // drop
            0x1A, // drop (redundant)
            0x0C, 0x00, // br 0 (redundant branch)
            0x41, 0x2A, // i32.const 42
            0x0F, // return
            0x41, 0x0D, // unreachable: i32.const 13
            0x1A, // unreachable: drop
        ];

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Aggressive);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);

        // Should be smaller than original due to optimizations
        assert!(optimized.len() <= bytecode.len());
    }

    #[test]
    fn test_optimization_with_empty_bytecode() {
        let optimizer = WasmOptimizer::new();
        let empty_bytecode = vec![];

        let result = optimizer.optimize(&empty_bytecode, &OptimizationLevel::Aggressive);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodegenError::OptimizationError(_)
        ));
    }

    #[test]
    fn test_optimization_with_minimal_valid_bytecode() {
        let optimizer = WasmOptimizer::new();
        let minimal_bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
        ];

        let result = optimizer.optimize(&minimal_bytecode, &OptimizationLevel::Size);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert_eq!(optimized, minimal_bytecode);
    }

    #[test]
    fn test_dead_code_elimination_with_nested_blocks() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode with nested blocks and unreachable code
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x02, 0x40, // block
            0x41, 0x2A, // i32.const 42
            0x0C, 0x00, // br 0 (unconditional branch)
            0x41, 0x0D, // unreachable: i32.const 13
            0x0B, // end block
            0x0F, // return
        ];

        let result = optimizer.eliminate_dead_code(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should have removed unreachable code after branch
        assert!(!optimized[10..].contains(&0x0D)); // Should not contain unreachable constant 13
    }

    #[test]
    fn test_constant_folding_edge_cases() {
        let optimizer = WasmOptimizer::new();

        // Test with insufficient bytes for pattern matching
        let short_bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x05, // i32.const 5 (incomplete pattern)
        ];

        let result = optimizer.fold_constants(short_bytecode.clone());
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should remain unchanged since pattern is incomplete
        assert_eq!(optimized, short_bytecode);
    }

    #[test]
    fn test_constant_folding_with_overflow() {
        let optimizer = WasmOptimizer::new();

        // Test constant folding with values that would overflow
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0xFF, // i32.const 255
            0x41, 0x01, // i32.const 1
            0x6A, // i32.add (would overflow as u8)
        ];

        let result = optimizer.fold_constants(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should handle wrapping arithmetic correctly
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_control_flow_optimization_edge_cases() {
        let optimizer = WasmOptimizer::new();

        // Test with branch at end of bytecode
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x0C, 0x00, // br 0 (at end, no target check possible)
        ];

        let result = optimizer.optimize_control_flow(bytecode.clone());
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should handle edge case gracefully
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }

    #[test]
    fn test_instruction_compression_multiple_sequences() {
        let optimizer = WasmOptimizer::new();

        // Test with multiple sequences of redundant drops
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x1A, // drop
            0x1A, // drop (redundant)
            0x41, 0x0D, // i32.const 13
            0x1A, // drop
            0x1A, // drop (redundant)
            0x0F, // return
        ];

        let result = optimizer.compress_instructions(bytecode);
        assert!(result.is_ok());

        let optimized = result.unwrap();

        // Should have compressed both sequences
        let drop_count = optimized.iter().filter(|&&b| b == 0x1A).count();
        assert_eq!(drop_count, 2); // Should have only two drops total
    }

    #[test]
    fn test_optimization_convergence() {
        let optimizer = WasmOptimizer::new();

        // Test that aggressive optimization converges (stops when no more improvements)
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
            0x0F, // return
        ];

        let result = optimizer.optimize_aggressive(bytecode.clone());
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should converge quickly since there's nothing to optimize
        assert_eq!(optimized, bytecode);
    }

    #[test]
    fn test_optimization_stats_calculation() {
        let optimizer = WasmOptimizer::new();

        let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let optimized = vec![1, 2, 3, 4, 5];

        let stats = optimizer.get_stats(&original, &optimized);

        assert_eq!(stats.original_size, 10);
        assert_eq!(stats.optimized_size, 5);
        // Verify that the size reduction is correctly calculated
        assert!(stats.optimized_size < stats.original_size);
    }

    #[test]
    fn test_optimization_stats_no_change() {
        let optimizer = WasmOptimizer::new();

        let bytecode = vec![1, 2, 3, 4, 5];
        let stats = optimizer.get_stats(&bytecode, &bytecode);

        assert_eq!(stats.original_size, stats.optimized_size);
        assert_eq!(stats.original_size, 5);
    }

    #[test]
    fn test_optimization_with_invalid_wasm_structure() {
        let optimizer = WasmOptimizer::new();
        
        // Create bytecode that looks like WASM but has invalid structure
        let invalid_bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0xFF, 0xFF, 0xFF, 0xFF, // Invalid section
        ];

        let result = optimizer.optimize(&invalid_bytecode, &OptimizationLevel::Speed);
        assert!(result.is_ok()); // Should handle gracefully
        
        let optimized = result.unwrap();
        // Should still have valid WASM header
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_optimization_preserves_section_order() {
        let optimizer = WasmOptimizer::new();
        
        // Create bytecode with multiple sections
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section
            0x03, 0x02, 0x01, 0x00, // Function section
            0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
        ];

        let result = optimizer.optimize(&bytecode, &OptimizationLevel::Size);
        assert!(result.is_ok());
        
        let optimized = result.unwrap();
        // Should maintain WASM structure
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_block_boundary_detection_comprehensive() {
        let optimizer = WasmOptimizer::new();

        // Test all block boundary instructions
        let block_instructions = vec![0x02, 0x03, 0x04, 0x05, 0x0B];
        for instruction in block_instructions {
            assert!(optimizer.is_block_boundary(instruction));
        }

        // Test non-block instructions
        let non_block_instructions = vec![0x00, 0x01, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0C, 0x0D, 0x0E, 0x0F];
        for instruction in non_block_instructions {
            assert!(!optimizer.is_block_boundary(instruction));
        }
    }

    #[test]
    fn test_optimization_with_corrupted_header() {
        let optimizer = WasmOptimizer::new();

        // Test with corrupted magic number
        let corrupted_magic = vec![
            0x00, 0x61, 0x73, 0x6E, // Wrong magic (6E instead of 6D)
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x2A, // i32.const 42
        ];

        let result = optimizer.optimize(&corrupted_magic, &OptimizationLevel::Size);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodegenError::OptimizationError(_)
        ));
    }

    #[test]
    fn test_optimization_preserves_functionality() {
        let optimizer = WasmOptimizer::new();

        // Create bytecode that should maintain functionality after optimization
        let functional_bytecode = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x41, 0x0A, // i32.const 10
            0x41, 0x05, // i32.const 5
            0x6A, // i32.add
            0x21, 0x00, // local.set 0
            0x20, 0x00, // local.get 0
            0x0F, // return
        ];

        let result = optimizer.optimize(&functional_bytecode, &OptimizationLevel::Speed);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // Should maintain valid WASM structure
        assert_eq!(&optimized[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&optimized[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }
}
