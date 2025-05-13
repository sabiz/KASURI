use crate::core::kasuri::KasuriResult;
use serde::de::DeserializeOwned;
use std::io::{Error, ErrorKind};
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
        let output: std::process::Output =
            Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(command)
                .output()?;
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
