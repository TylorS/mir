//! Value Converter for JavaScript-WASM interop

use super::{JsValueConversion, JsBatchConversionResult, JsConversionStats};
use napi::{Result as NapiResult, Error as NapiError, Status};
use napi_derive::napi;
use std::collections::HashMap;
use std::time::{Instant, SystemTime};

/// Type mapping configuration
#[derive(Clone)]
#[napi(object)]
pub struct JsTypeMapping {
    pub source_type: String,
    pub target_type: String,
    pub conversion_function: String,
    pub is_lossy: bool,
    pub validation_required: bool,
}

/// Conversion cache entry
#[derive(Debug, Clone)]
struct ConversionCacheEntry {
    source_value: String,
    source_type: String,
    target_type: String,
    converted_value: String,
    created_at: SystemTime,
    access_count: u64,
}

/// Internal conversion statistics
#[derive(Debug, Clone, Default)]
struct InternalConversionStats {
    total_conversions: u64,
    successful_conversions: u64,
    failed_conversions: u64,
    cache_hits: u64,
    cache_misses: u64,
    total_conversion_time_ms: f64,
}

/// Value Converter for Node.js
#[napi]
pub struct ValueConverter {
    type_mappings: HashMap<String, JsTypeMapping>,
    conversion_cache: HashMap<String, ConversionCacheEntry>,
    statistics: InternalConversionStats,
    cache_enabled: bool,
    max_cache_size: usize,
}

#[napi]
impl ValueConverter {
    /// Create new value converter
    #[napi(constructor)]
    pub fn new() -> Self {
        let mut converter = ValueConverter {
            type_mappings: HashMap::new(),
            conversion_cache: HashMap::new(),
            statistics: InternalConversionStats::default(),
            cache_enabled: true,
            max_cache_size: 1000,
        };

        // Register default type mappings
        converter.register_default_mappings();
        converter
    }

    /// Convert JavaScript value to WASM type
    #[napi]
    pub fn js_to_wasm(
        &mut self,
        source_value: String,
        target_type: String,
    ) -> NapiResult<String> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("{}:{}:{}", source_value, "javascript", target_type);
        if self.cache_enabled {
            if let Some(cached) = self.conversion_cache.get_mut(&cache_key) {
                cached.access_count += 1;
                self.statistics.cache_hits += 1;
                self.statistics.total_conversions += 1;
                return Ok(cached.converted_value.clone());
            }
            self.statistics.cache_misses += 1;
        }

        // Perform conversion
        let result = self.convert_value_internal(&source_value, "javascript", &target_type);
        let conversion_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Update statistics
        self.statistics.total_conversions += 1;
        self.statistics.total_conversion_time_ms += conversion_time;
        
        match result {
            Ok(converted_value) => {
                self.statistics.successful_conversions += 1;
                
                // Cache the result
                if self.cache_enabled && self.conversion_cache.len() < self.max_cache_size {
                    let cache_entry = ConversionCacheEntry {
                        source_value: source_value.clone(),
                        source_type: "javascript".to_string(),
                        target_type: target_type.clone(),
                        converted_value: converted_value.clone(),
                        created_at: SystemTime::now(),
                        access_count: 1,
                    };
                    self.conversion_cache.insert(cache_key, cache_entry);
                }
                
                Ok(converted_value)
            }
            Err(e) => {
                self.statistics.failed_conversions += 1;
                Err(e)
            }
        }
    }

    /// Convert WASM value to JavaScript type
    #[napi]
    pub fn wasm_to_js(
        &mut self,
        source_value: String,
        source_type: String,
    ) -> NapiResult<String> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("{}:{}:javascript", source_value, source_type);
        if self.cache_enabled {
            if let Some(cached) = self.conversion_cache.get_mut(&cache_key) {
                cached.access_count += 1;
                self.statistics.cache_hits += 1;
                self.statistics.total_conversions += 1;
                return Ok(cached.converted_value.clone());
            }
            self.statistics.cache_misses += 1;
        }

        // Perform conversion
        let result = self.convert_value_internal(&source_value, &source_type, "javascript");
        let conversion_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Update statistics
        self.statistics.total_conversions += 1;
        self.statistics.total_conversion_time_ms += conversion_time;
        
        match result {
            Ok(converted_value) => {
                self.statistics.successful_conversions += 1;
                
                // Cache the result
                if self.cache_enabled && self.conversion_cache.len() < self.max_cache_size {
                    let cache_entry = ConversionCacheEntry {
                        source_value: source_value.clone(),
                        source_type: source_type.clone(),
                        target_type: "javascript".to_string(),
                        converted_value: converted_value.clone(),
                        created_at: SystemTime::now(),
                        access_count: 1,
                    };
                    self.conversion_cache.insert(cache_key, cache_entry);
                }
                
                Ok(converted_value)
            }
            Err(e) => {
                self.statistics.failed_conversions += 1;
                Err(e)
            }
        }
    }

    /// Batch convert multiple values
    #[napi]
    pub fn batch_convert(
        &mut self,
        conversions: Vec<JsValueConversion>,
    ) -> NapiResult<JsBatchConversionResult> {
        let start_time = Instant::now();
        let mut converted_values = Vec::new();
        let mut errors = Vec::new();
        let mut successful_conversions = 0;

        for conversion in conversions {
            let result = self.convert_value_internal(
                &conversion.source_value,
                &conversion.source_type,
                &conversion.target_type,
            );

            match result {
                Ok(converted) => {
                    converted_values.push(converted);
                    successful_conversions += 1;
                }
                Err(e) => {
                    converted_values.push(String::new());
                    errors.push(e.to_string());
                }
            }
        }

        let total_conversions = converted_values.len();
        let success_rate = if total_conversions > 0 {
            successful_conversions as f64 / total_conversions as f64
        } else {
            0.0
        };

        let total_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(JsBatchConversionResult {
            success_rate,
            total_time_ms,
            converted_values,
            errors,
        })
    }

    /// Register custom type mapping
    #[napi]
    pub fn register_type_mapping(
        &mut self,
        mapping_name: String,
        source_type: String,
        target_type: String,
        conversion_function: String,
    ) -> NapiResult<bool> {
        let mapping = JsTypeMapping {
            source_type,
            target_type,
            conversion_function,
            is_lossy: false,
            validation_required: true,
        };

        self.type_mappings.insert(mapping_name, mapping);
        Ok(true)
    }

    /// Get conversion statistics
    #[napi]
    pub fn get_conversion_stats(&self) -> JsConversionStats {
        let average_conversion_time = if self.statistics.total_conversions > 0 {
            self.statistics.total_conversion_time_ms / self.statistics.total_conversions as f64
        } else {
            0.0
        };

        JsConversionStats {
            total_conversions: self.statistics.total_conversions as f64,
            cache_hits: self.statistics.cache_hits as f64,
            cache_misses: self.statistics.cache_misses as f64,
            successful_conversions: self.statistics.successful_conversions as f64,
            failed_conversions: self.statistics.failed_conversions as f64,
            average_conversion_time_ms: average_conversion_time,
        }
    }

    /// Clear conversion cache
    #[napi]
    pub fn clear_cache(&mut self) -> u32 {
        let cleared_count = self.conversion_cache.len() as u32;
        self.conversion_cache.clear();
        cleared_count
    }

    /// Set cache configuration
    #[napi]
    pub fn configure_cache(&mut self, enabled: bool, max_size: u32) {
        self.cache_enabled = enabled;
        self.max_cache_size = max_size as usize;
        
        // Trim cache if new max size is smaller
        if self.conversion_cache.len() > self.max_cache_size {
            // Remove least recently used entries
            let keys_to_remove: Vec<String> = {
                let mut entries: Vec<_> = self.conversion_cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.access_count);
                
                let to_remove = self.conversion_cache.len() - self.max_cache_size;
                entries.iter().take(to_remove).map(|(key, _)| (*key).clone()).collect()
            };
            
            for key in keys_to_remove {
                self.conversion_cache.remove(&key);
            }
        }
    }

    /// Get cache statistics
    #[napi]
    pub fn get_cache_stats(&self) -> NapiResult<napi::JsObject> {
        // This would return cache-specific statistics in a full implementation
        // For now, return basic info as a placeholder
        // Return empty object for now - would contain cache stats in full implementation
        Err(NapiError::new(Status::GenericFailure, "Cache stats not implemented"))
    }

    /// Validate conversion compatibility
    #[napi]
    pub fn validate_conversion(
        &self,
        source_type: String,
        target_type: String,
    ) -> NapiResult<bool> {
        // Check if conversion is supported
        let is_supported = self.is_conversion_supported(&source_type, &target_type);
        Ok(is_supported)
    }

    /// List supported conversions
    #[napi]
    pub fn list_supported_conversions(&self) -> Vec<String> {
        let mut conversions = Vec::new();
        
        // Add built-in conversions
        let builtin_types = vec!["i32", "i64", "f32", "f64", "string", "boolean"];
        for source in &builtin_types {
            for target in &builtin_types {
                if source != target {
                    conversions.push(format!("{} -> {}", source, target));
                }
            }
        }
        
        // Add custom mappings
        for (name, mapping) in &self.type_mappings {
            conversions.push(format!("{}: {} -> {}", name, mapping.source_type, mapping.target_type));
        }
        
        conversions
    }

    /// Reset conversion statistics
    #[napi]
    pub fn reset_statistics(&mut self) {
        self.statistics = InternalConversionStats::default();
    }

    /// Get type mapping information
    #[napi]
    pub fn get_type_mapping(&self, mapping_name: String) -> NapiResult<JsTypeMapping> {
        self.type_mappings.get(&mapping_name)
            .cloned()
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Type mapping not found"))
    }

    /// Remove type mapping
    #[napi]
    pub fn remove_type_mapping(&mut self, mapping_name: String) -> bool {
        self.type_mappings.remove(&mapping_name).is_some()
    }

    /// List all type mappings
    #[napi]
    pub fn list_type_mappings(&self) -> Vec<String> {
        self.type_mappings.keys().cloned().collect()
    }
}

// Implementation helper methods
impl ValueConverter {
    /// Register default type mappings
    fn register_default_mappings(&mut self) {
        let default_mappings = vec![
            ("number_to_i32", "number", "i32", "parseInt"),
            ("number_to_i64", "number", "i64", "parseInt64"),
            ("number_to_f32", "number", "f32", "parseFloat32"),
            ("number_to_f64", "number", "f64", "parseFloat64"),
            ("string_to_number", "string", "number", "parseNumber"),
            ("boolean_to_i32", "boolean", "i32", "boolToInt"),
        ];

        for (name, source, target, func) in default_mappings {
            let mapping = JsTypeMapping {
                source_type: source.to_string(),
                target_type: target.to_string(),
                conversion_function: func.to_string(),
                is_lossy: false,
                validation_required: true,
            };
            self.type_mappings.insert(name.to_string(), mapping);
        }
    }

    /// Internal conversion implementation
    fn convert_value_internal(
        &self,
        source_value: &str,
        source_type: &str,
        target_type: &str,
    ) -> NapiResult<String> {
        // Handle same-type conversions
        if source_type == target_type {
            return Ok(source_value.to_string());
        }

        // Handle built-in conversions
        match (source_type, target_type) {
            ("javascript", "i32") | ("number", "i32") => {
                let value: i32 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to i32"))?;
                Ok(value.to_string())
            }
            ("javascript", "i64") | ("number", "i64") => {
                let value: i64 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to i64"))?;
                Ok(value.to_string())
            }
            ("javascript", "f32") | ("number", "f32") => {
                let value: f32 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to f32"))?;
                Ok(value.to_string())
            }
            ("javascript", "f64") | ("number", "f64") => {
                let value: f64 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to f64"))?;
                Ok(value.to_string())
            }
            ("javascript", "string") | (_, "string") => {
                Ok(source_value.to_string())
            }
            ("javascript", "boolean") | ("string", "boolean") => {
                let value = match source_value.to_lowercase().as_str() {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => return Err(NapiError::new(Status::InvalidArg, "Cannot convert to boolean")),
                };
                Ok(value.to_string())
            }
            ("i32", "javascript") | ("i32", "number") => {
                // Validate it's a valid i32
                let _: i32 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Invalid i32 value"))?;
                Ok(source_value.to_string())
            }
            ("i64", "javascript") | ("i64", "number") => {
                // Validate it's a valid i64
                let _: i64 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Invalid i64 value"))?;
                Ok(source_value.to_string())
            }
            ("f32", "javascript") | ("f32", "number") => {
                // Validate it's a valid f32
                let _: f32 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Invalid f32 value"))?;
                Ok(source_value.to_string())
            }
            ("f64", "javascript") | ("f64", "number") => {
                // Validate it's a valid f64
                let _: f64 = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Invalid f64 value"))?;
                Ok(source_value.to_string())
            }
            ("boolean", "i32") => {
                let bool_val: bool = source_value.parse()
                    .map_err(|_| NapiError::new(Status::InvalidArg, "Invalid boolean value"))?;
                Ok(if bool_val { "1" } else { "0" }.to_string())
            }
            _ => {
                // Check custom type mappings
                for mapping in self.type_mappings.values() {
                    if mapping.source_type == source_type && mapping.target_type == target_type {
                        // In a full implementation, this would execute the conversion function
                        return Ok(format!("converted_{}_{}", source_value, target_type));
                    }
                }
                
                Err(NapiError::new(
                    Status::InvalidArg,
                    format!("Unsupported conversion: {} -> {}", source_type, target_type),
                ))
            }
        }
    }

    /// Check if conversion is supported
    fn is_conversion_supported(&self, source_type: &str, target_type: &str) -> bool {
        // Same type is always supported
        if source_type == target_type {
            return true;
        }

        // Check built-in conversions
        let builtin_conversions = vec![
            ("javascript", "i32"), ("javascript", "i64"), ("javascript", "f32"), ("javascript", "f64"),
            ("javascript", "string"), ("javascript", "boolean"),
            ("number", "i32"), ("number", "i64"), ("number", "f32"), ("number", "f64"),
            ("string", "boolean"), ("string", "i32"), ("string", "i64"), ("string", "f32"), ("string", "f64"),
            ("boolean", "i32"), ("boolean", "string"),
            ("i32", "javascript"), ("i64", "javascript"), ("f32", "javascript"), ("f64", "javascript"),
            ("i32", "number"), ("i64", "number"), ("f32", "number"), ("f64", "number"),
        ];

        if builtin_conversions.contains(&(source_type, target_type)) {
            return true;
        }

        // Check custom mappings
        for mapping in self.type_mappings.values() {
            if mapping.source_type == source_type && mapping.target_type == target_type {
                return true;
            }
        }

        false
    }
}

impl Default for ValueConverter {
    fn default() -> Self {
        Self::new()
    }
}