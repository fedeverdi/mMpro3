use anyhow::Result;
use std::io::{self as stdio, BufRead};

// Import organized modules
mod effects;
mod meters;
mod io;
mod processing;
mod engine;
mod ipc;
mod command;

use ipc::*;  // Import all IPC message types
use engine::{
    AudioEngine,
    send_response,
};

fn main() -> Result<()> {
    eprintln!("[Engine] mMpro3 Audio Engine starting...");

    let mut engine = AudioEngine::new();
    
    // Auto-select default output device on startup
    match engine.list_devices() {
        Ok(devices) => {
            // Find default output device
            if let Some(default_device) = devices.iter().find(|d| d.is_default && d.output_channels > 0) {
                let default_selection = format!("{}:0:1", default_device.id);
                *engine.selected_master_output.lock().unwrap() = Some(default_selection.clone());
                eprintln!("[Engine] Auto-selected default output: {} ({})", default_device.name, default_selection);
            }
            // Store available output devices for serialization
            engine.available_output_devices = devices.into_iter()
                .filter(|d| d.output_channels > 0)
                .collect();
        }
        Err(e) => {
            eprintln!("[Engine] Warning: Could not enumerate devices for auto-selection: {}", e);
        }
    }
    
    let stdin = stdio::stdin();
    let mut lines = stdin.lock().lines();

    // Loop: read commands from stdin
    while let Some(Ok(line)) = lines.next() {
        match serde_json::from_str::<Command>(&line) {
            Ok(command) => {
                // Only send response if there is one (critical operations only)
                if let Some(response) = engine.handle_command(command) {
                    send_response(&response);
                }
            }
            Err(e) => {
                eprintln!("[Engine] Failed to parse command: {}", e);
                eprintln!("[Engine] Raw input: {}", line);
                send_response(&Response::Error {
                    message: format!("Invalid command: {}", e),
                });
            }
        }
    }

    eprintln!("[Engine] Shutting down...");
    Ok(())
}

