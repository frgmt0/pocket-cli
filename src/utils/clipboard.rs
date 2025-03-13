use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};

/// Read content from the system clipboard
/// 
/// Supports macOS (pbpaste), Windows (PowerShell), and Linux (xclip/wl-paste)
pub fn read_clipboard() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|_| anyhow!("Failed to access clipboard. Make sure 'pbpaste' is available."))?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read from clipboard"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell.exe")
            .args(["-command", "Get-Clipboard"])
            .output()
            .map_err(|_| anyhow!("Failed to access clipboard. Make sure PowerShell is available."))?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read from clipboard"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Try XClip first (X11)
        let xclip_result = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output();
        
        // If XClip works, use it
        if let Ok(output) = xclip_result {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        // Try wl-paste (Wayland)
        let wl_paste_result = Command::new("wl-paste")
            .output();
        
        // If wl-paste works, use it
        if let Ok(output) = wl_paste_result {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        // If neither worked, raise an error
        Err(anyhow!("Failed to access clipboard. Please install xclip (for X11) or wl-paste (for Wayland)."))
    }
    
    // Fallback for unsupported platforms
    #[cfg(not(any(target_os = "macos", target_os = "windows", all(unix, not(target_os = "macos")))))]
    {
        Err(anyhow!("Clipboard functionality is not supported on this platform"))
    }
}

/// Write content to the system clipboard
/// 
/// Supports macOS (pbcopy), Windows (PowerShell), and Linux (xclip/wl-copy)
pub fn write_clipboard(content: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| anyhow!("Failed to access clipboard. Make sure 'pbcopy' is available."))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
        }
        
        let status = child.wait()?;
        if !status.success() {
            return Err(anyhow!("Failed to write to clipboard"));
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    {
        // PowerShell command to set clipboard
        let mut child = Command::new("powershell.exe")
            .args(["-command", "Set-Clipboard -Value $input"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| anyhow!("Failed to access clipboard. Make sure PowerShell is available."))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
        }
        
        let status = child.wait()?;
        if !status.success() {
            return Err(anyhow!("Failed to write to clipboard"));
        }
        
        Ok(())
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Try XClip first (X11)
        let mut xclip_child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn();
        
        if let Ok(mut child) = xclip_child {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())?;
            }
            
            let status = child.wait()?;
            if status.success() {
                return Ok(());
            }
        }
        
        // Try wl-copy (Wayland)
        let mut wl_copy_child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn();
        
        if let Ok(mut child) = wl_copy_child {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())?;
            }
            
            let status = child.wait()?;
            if status.success() {
                return Ok(());
            }
        }
        
        // If neither worked, raise an error
        Err(anyhow!("Failed to access clipboard. Please install xclip (for X11) or wl-copy (for Wayland)."))
    }
    
    // Fallback for unsupported platforms
    #[cfg(not(any(target_os = "macos", target_os = "windows", all(unix, not(target_os = "macos")))))]
    {
        Err(anyhow!("Clipboard functionality is not supported on this platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Ignore by default as it interacts with system clipboard
    fn test_clipboard_write_read() {
        let test_content = "Test clipboard content";
        
        // Write to clipboard
        write_clipboard(test_content).expect("Failed to write to clipboard");
        
        // Read from clipboard
        let read_content = read_clipboard().expect("Failed to read from clipboard");
        
        assert_eq!(read_content, test_content);
    }
} 