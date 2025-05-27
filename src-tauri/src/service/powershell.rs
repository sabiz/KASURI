use crate::core::kasuri::KasuriResult;
use serde::de::DeserializeOwned;
use std::io::{Error, ErrorKind, Write};
use std::process::Command;

pub struct PowerShell {}

pub struct PowerShellResult {
    pub stdout: String,
    pub _stderr: String,
}

impl PowerShell {
    pub fn new() -> Self {
        Self {}
    }
    pub fn run(&self, command: &str) -> KasuriResult<PowerShellResult> {
        let temp_file_path = self.create_temp_script(command)?;
        let output: std::process::Output =
            Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-WindowStyle")
                .arg("Hidden")
                .arg("-File")
                .arg(&temp_file_path)
                .output()?; // Clean up the temporary file
        let _ = std::fs::remove_file(&temp_file_path);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                format!(
                    "PowerShell command failed, stdout: {}, stderr: {}",
                    stdout, stderr
                ),
            )));
        }
        Ok(PowerShellResult {
            stdout,
            _stderr: stderr,
        })
    }

    fn create_temp_script(&self, command: &str) -> KasuriResult<String> {
        // Create a temporary file for the PowerShell script
        let temp_dir = std::env::temp_dir();
        let file_name = format!(
            "kasuri_ps_{}.ps1",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let temp_file_path = temp_dir.join(file_name);

        // Write the command to the temporary file with UTF-8 BOM
        let mut file = std::fs::File::create(&temp_file_path)?;
        // Write UTF-8 BOM (0xEF, 0xBB, 0xBF)
        file.write_all(&[0xEF, 0xBB, 0xBF])?;
        // Write the actual command
        file.write_all(command.as_bytes())?;
        file.flush()?; // Execute the temporary script file
        file.try_clone()?;
        Ok(temp_file_path.to_string_lossy().to_string())
    }
}

impl PowerShellResult {
    /// Run a PowerShell command and convert JSON output to a specified struct type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The struct type to convert JSON output to. Must implement DeserializeOwned.
    ///
    /// # Arguments
    ///
    /// * `command` - The PowerShell command to execute
    ///
    /// # Returns
    ///
    /// * `KasuriResult<T>` - The deserialized struct on success
    ///
    /// # Errors
    ///
    /// Returns an error if the PowerShell command fails, or if deserialization fails
    pub fn to_struct<T>(self) -> KasuriResult<T>
    where
        T: DeserializeOwned,
    {
        // Trim the output to remove any leading/trailing whitespace
        let trimmed_output = self.stdout.trim();

        // Parse the JSON output
        match serde_json::from_str::<T>(trimmed_output) {
            Ok(result) => Ok(result),
            Err(e) => Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Failed to parse JSON output: {}. Output was: {}",
                    e, trimmed_output
                ),
            ))),
        }
    }
}
