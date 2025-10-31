use serde::{Serialize, Deserialize};
use crate::models::vulnerability::Vulnerability;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub target: String,
    pub engines: Vec<String>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub scan_duration_ms: u64,
}
