use crate::KasuriResult;
use serde::de::DeserializeOwned;
use std::io::{Error, ErrorKind, Write};
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Service for executing PowerShell commands.
///
/// This struct encapsulates the functionality required to run PowerShell
/// commands on Windows systems and process their output.
pub struct PowerShell {}

/// Result container for PowerShell command execution.
///
/// This struct holds the standard output and error output from
/// a PowerShell command execution.
pub struct PowerShellResult {
    /// Standard output from the PowerShell command
    pub stdout: String,
    /// Standard error output from the PowerShell command
    pub _stderr: String,
}

impl PowerShell {
    /// Creates a new PowerShell service instance.
    ///
    /// # Returns
    ///
    /// A new PowerShell service instance ready for executing PowerShell commands.
    pub fn new() -> Self {
        log::debug!("Initializing new PowerShell service");
        Self {}
    }

    /// Executes a PowerShell command and returns its results.
    ///
    /// This method:
    /// 1. Creates a temporary script file with the provided command
    /// 2. Executes the script using the PowerShell executable
    /// 3. Captures and processes the command output
    /// 4. Cleans up temporary files
    ///
    /// # Arguments
    ///
    /// * `command` - The PowerShell command to execute as a string
    ///
    /// # Returns
    ///
    /// * `KasuriResult<PowerShellResult>` - The command execution results or an error
    pub fn run(&self, command: &str) -> KasuriResult<PowerShellResult> {
        // Create temporary script file
        let temp_file_path = self.create_temp_script(command)?;
        log::debug!("Created temporary script file at: {}", temp_file_path);

        // Execute PowerShell with the script file
        log::debug!("Executing PowerShell with script file");
        let output: std::process::Output =
            Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
                .creation_flags(CREATE_NO_WINDOW)
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-WindowStyle")
                .arg("Hidden")
                .arg("-File")
                .arg(&temp_file_path)
                .output()?;

        // Clean up temporary file
        log::debug!("Cleaning up temporary script file");
        if let Err(e) = std::fs::remove_file(&temp_file_path) {
            log::warn!(
                "Failed to remove temporary script file {}: {}",
                temp_file_path,
                e
            );
        }

        // Process command output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Handle command status
        if !output.status.success() {
            log::error!(
                "PowerShell command execution failed with status: {:?}",
                output.status
            );
            log::error!("PowerShell stderr: {}", stderr);
            log::error!("PowerShell stdout: {}", stdout);

            return Err(Box::new(Error::new(
                ErrorKind::Other,
                format!(
                    "PowerShell command failed, stdout: {}, stderr: {}",
                    stdout, stderr
                ),
            )));
        }
        log::debug!("PowerShell stdout length: {} bytes", stdout.len());

        Ok(PowerShellResult {
            stdout,
            _stderr: stderr,
        })
    }
    /// Creates a temporary PowerShell script file with the provided command.
    ///
    /// This method:
    /// 1. Creates a unique temporary file in the system temp directory
    /// 2. Writes the PowerShell command with UTF-8 BOM encoding
    /// 3. Returns the path to the created script file
    ///
    /// # Arguments
    ///
    /// * `command` - The PowerShell command to write to the script file
    ///
    /// # Returns
    ///
    /// * `KasuriResult<String>` - The path to the created script file or an error
    fn create_temp_script(&self, command: &str) -> KasuriResult<String> {
        log::debug!("Creating temporary PowerShell script file");

        // Create a temporary file for the PowerShell script with unique timestamp
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let file_name = format!("kasuri_ps_{}.ps1", timestamp);
        let temp_file_path = temp_dir.join(file_name);

        log::debug!("Temporary script path: {}", temp_file_path.display()); // Create the file
        let mut file = match std::fs::File::create(&temp_file_path) {
            Ok(file) => file,
            Err(e) => {
                log::error!("Failed to create temporary script file: {}", e);
                return Err(Box::new(e));
            }
        };

        // Write UTF-8 BOM (0xEF, 0xBB, 0xBF)
        if let Err(e) = file.write_all(&[0xEF, 0xBB, 0xBF]) {
            log::error!("Failed to write BOM to script file: {}", e);
            return Err(Box::new(e));
        }

        // Write the actual command
        if let Err(e) = file.write_all(command.as_bytes()) {
            log::error!("Failed to write command to script file: {}", e);
            return Err(Box::new(e));
        }

        // Flush and finalize the file
        if let Err(e) = file.flush() {
            log::error!("Failed to flush script file: {}", e);
            return Err(Box::new(e));
        }

        // Ensure file handle is valid
        if let Err(e) = file.try_clone() {
            log::error!("Failed to validate script file handle: {}", e);
            return Err(Box::new(e));
        }

        log::debug!("Temporary script file created successfully");
        Ok(temp_file_path.to_string_lossy().to_string())
    }
}

impl PowerShellResult {
    /// Parses the PowerShell command output as JSON and converts it to a specified struct type.
    ///
    /// This method attempts to deserialize the stdout content from the PowerShell
    /// command into the specified struct type using serde_json.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The struct type to convert JSON output to. Must implement DeserializeOwned.
    ///
    /// # Returns
    ///
    /// * `KasuriResult<T>` - The deserialized struct on success
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails, typically due to:
    /// - Malformed JSON in the output
    /// - JSON structure not matching the target type
    /// - Empty or non-JSON output from the command
    pub fn to_struct<T>(self) -> KasuriResult<T>
    where
        T: DeserializeOwned,
    {
        log::debug!("Converting PowerShell output to struct");

        // Trim the output to remove any leading/trailing whitespace
        let trimmed_output = self.stdout.trim();
        if trimmed_output.is_empty() {
            log::warn!("PowerShell output is empty, deserialization will likely fail");
        } else {
            log::debug!("PowerShell output length: {} chars", trimmed_output.len());
        }

        // Parse the JSON output
        log::debug!("Attempting to deserialize JSON output");
        match serde_json::from_str::<T>(trimmed_output) {
            Ok(result) => {
                log::info!("Successfully deserialized PowerShell output to struct");
                Ok(result)
            }
            Err(e) => {
                log::error!("Failed to deserialize PowerShell output: {}", e);

                // Log a truncated version of the output for debugging
                if trimmed_output.len() > 200 {
                    log::debug!(
                        "First 200 chars of problematic output: {}",
                        &trimmed_output[0..200]
                    );
                } else {
                    log::debug!("Problematic output: {}", trimmed_output);
                }

                Err(Box::new(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Failed to parse JSON output: {}. Output was: {}",
                        e, trimmed_output
                    ),
                )))
            }
        }
    }
}
