use crate::models::scan_result::ScanResult;
use serde_json;

pub fn display_scan_result(result: &ScanResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}
