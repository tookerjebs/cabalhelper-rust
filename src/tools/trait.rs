// Shared trait for all automation tools
use windows::Win32::Foundation::HWND;

/// Common interface that all tools must implement
pub trait Tool {

    
    /// Stop the tool (emergency stop)
    fn stop(&mut self);
    
    /// Check if the tool is currently running
    fn is_running(&self) -> bool;
    
    /// Get current status message
    fn get_status(&self) -> String;
}
