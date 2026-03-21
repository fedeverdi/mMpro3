use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseData {
    pub key: String,
    pub license_type: String,
    pub expires_at: Option<String>,
    pub is_valid: bool,
}

pub fn get_license_file_path() -> PathBuf {
    // Use environment variable or default to current directory
    let base_path = std::env::var("LICENSE_PATH")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base_path).join("license.json")
}

pub fn save_license_to_file(key: &str, license_type: &str, expires_at: Option<&str>) -> Result<()> {
    let license = LicenseData {
        key: key.to_string(),
        license_type: license_type.to_string(),
        expires_at: expires_at.map(|s| s.to_string()),
        is_valid: true,
    };
    
    let path = get_license_file_path();
    
    let json = serde_json::to_string_pretty(&license)?;
    
    match std::fs::write(&path, &json) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to write license file: {}", e);
            Err(e.into())
        }
    }
}

pub fn load_license_from_file() -> Result<LicenseData> {
    let path = get_license_file_path();
    
    if !path.exists() {
        eprintln!("[Engine] No license file found at {:?}, using demo mode", path);
        return Ok(LicenseData {
            key: "DEMO".to_string(),
            license_type: "demo".to_string(),
            expires_at: None,
            is_valid: true,
        });
    }
    
    let json = std::fs::read_to_string(&path)?;
    let license: LicenseData = serde_json::from_str(&json)?;
    Ok(license)
}
