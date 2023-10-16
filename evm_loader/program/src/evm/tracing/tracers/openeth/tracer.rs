use crate::evm::tracing::tracers::openeth::types::{CallAnalytics, TraceResults};
use crate::evm::tracing::{EmulationResult, Event, EventListener};
use crate::types::hexbytes::HexBytes;
use serde_json::Value;
use std::fmt::Debug;

#[derive(Debug)]
pub struct OpenEthereumTracer {
    output: Option<HexBytes>,
    _call_analytics: CallAnalytics,
}

impl OpenEthereumTracer {
    pub fn new(call_analytics: CallAnalytics) -> OpenEthereumTracer {
        OpenEthereumTracer {
            output: None,
            _call_analytics: call_analytics,
        }
    }
}

impl EventListener for OpenEthereumTracer {
    fn event(&mut self, event: Event) {
        match event {
            Event::EndStep {
                gas_used: _gas_used,
                return_data,
            } => {
                self.output = return_data.map(Into::into);
            }
            _ => {}
        }
    }

    fn into_traces(self: Box<Self>, emulation_result: EmulationResult) -> Value {
        serde_json::to_value(TraceResults {
            output: self.output.unwrap_or_default(),
            trace: vec![],
            vm_trace: None,
            state_diff: Some(emulation_result.state_diff),
        })
        .unwrap()
    }
}
