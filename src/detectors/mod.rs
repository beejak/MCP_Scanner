pub mod tool_poisoning;
pub mod prompt_injection;
pub mod secrets;
pub mod pii;
pub mod code_vulns;
pub mod toxic_flows;
pub mod anomalies;

pub use tool_poisoning::ToolPoisoningDetector;
pub use prompt_injection::PromptInjectionDetector;
pub use secrets::SecretsDetector;

use crate::models::vulnerability::Vulnerability;
use anyhow::Result;

/// Trait for all vulnerability detectors
pub trait Detector: Send + Sync {
    /// Name of the detector
    fn name(&self) -> &'static str;

    /// Scan content for vulnerabilities
    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>>;
}
