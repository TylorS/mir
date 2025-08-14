//! OpenTelemetry integration

use std::collections::HashMap;

/// OpenTelemetry collector for observability
pub struct OpenTelemetryCollector {
    spans: Vec<Span>,
    metrics: HashMap<String, Metric>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

impl OpenTelemetryCollector {
    pub fn new() -> Self {
        OpenTelemetryCollector {
            spans: Vec::new(),
            metrics: HashMap::new(),
        }
    }
    
    pub fn start_span(&mut self, name: String) -> SpanId {
        let span = Span {
            name,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            end_time: None,
            attributes: HashMap::new(),
        };
        
        self.spans.push(span);
        SpanId(self.spans.len() - 1)
    }
    
    pub fn end_span(&mut self, span_id: SpanId) {
        if let Some(span) = self.spans.get_mut(span_id.0) {
            span.end_time = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpanId(usize);