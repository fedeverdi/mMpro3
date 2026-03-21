use std::path::PathBuf;
use crate::ipc::messages::Response;

/// Get available disk space in GB for the given path
pub fn get_available_disk_space_gb(path: &PathBuf) -> f32 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Get the parent directory (recordings folder)
        let dir = path.parent().unwrap_or(path.as_path());
        
        // Use df -k to get available space in KB
        match Command::new("df")
            .arg("-k")
            .arg(dir)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse output: df -k returns KB in 4th column of last line
                // Example: /dev/disk1s1  488555536 123456789 364098747    26%    1234567  9876543210   0%   /System/Volumes/Data
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available_kb) = parts[3].parse::<f64>() {
                            return (available_kb / (1024.0 * 1024.0)) as f32;
                        }
                    }
                }
                0.0
            }
            Err(_) => 0.0,
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        let dir = path.parent().unwrap_or(path.as_path());
        
        match Command::new("df")
            .arg("-k")
            .arg(dir)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available_kb) = parts[3].parse::<f64>() {
                            return (available_kb / (1024.0 * 1024.0)) as f32;
                        }
                    }
                }
                0.0
            }
            Err(_) => 0.0,
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Get drive letter
        let path_str = path.to_string_lossy();
        if path_str.len() >= 2 {
            let drive = &path_str[0..2];
            
            // Use wmic to get free space
            match Command::new("wmic")
                .args(&["logicaldisk", "where", &format!("DeviceID='{}'", drive), "get", "FreeSpace"])
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if let Ok(free_bytes) = line.trim().parse::<f64>() {
                            return (free_bytes / (1024.0 * 1024.0 * 1024.0)) as f32;
                        }
                    }
                    0.0
                }
                Err(_) => 0.0,
            }
        } else {
            0.0
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        0.0
    }
}

/// Send JSON response to stdout
pub fn send_response(response: &Response) {
    if let Ok(json) = serde_json::to_string(response) {
        println!("{}", json);
    }
}
