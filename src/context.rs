#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundWindow {
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub is_terminal: bool,
}

pub fn is_terminal_process(process_name: &str) -> bool {
    matches!(
        process_name.to_ascii_lowercase().as_str(),
        "windowsterminal.exe" | "pwsh.exe" | "powershell.exe" | "cmd.exe"
    )
}

#[cfg(windows)]
pub fn foreground_window() -> Result<ForegroundWindow, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND(std::ptr::null_mut()) {
            return Err("no foreground window".to_string());
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return Err("foreground window has no process id".to_string());
        }

        let title = window_title(hwnd);
        let process_name = process_name_from_snapshot(process_id)?;
        let is_terminal = is_terminal_process(&process_name);

        return Ok(ForegroundWindow {
            title,
            process_name,
            process_id,
            is_terminal,
        });
    }
}

#[cfg(windows)]
unsafe fn window_title(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return String::new();
    }

    let mut buffer = vec![0u16; len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..copied as usize])
}

#[cfg(windows)]
unsafe fn process_name_from_snapshot(process_id: u32) -> Result<String, String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|err| format!("CreateToolhelp32Snapshot failed: {err}"))?;

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut found = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while found {
        if entry.th32ProcessID == process_id {
            let name = wide_c_array_to_string(&entry.szExeFile);
            let _ = unsafe { CloseHandle(snapshot) };
            return Ok(name);
        }
        found = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }

    let _ = unsafe { CloseHandle(snapshot) };
    Err(format!(
        "process {process_id} not found in ToolHelp snapshot"
    ))
}

#[cfg(windows)]
fn wide_c_array_to_string(buffer: &[u16; windows::Win32::Foundation::MAX_PATH as usize]) -> String {
    let len = buffer
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

#[cfg(not(windows))]
pub fn foreground_window() -> Result<ForegroundWindow, String> {
    Err("VibeKeys MVP only supports Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_terminal_processes() {
        assert!(is_terminal_process("WindowsTerminal.exe"));
        assert!(is_terminal_process("pwsh.exe"));
        assert!(is_terminal_process("powershell.exe"));
        assert!(is_terminal_process("cmd.exe"));
        assert!(is_terminal_process("CMD.EXE"));
        assert!(!is_terminal_process("chrome.exe"));
    }
}
