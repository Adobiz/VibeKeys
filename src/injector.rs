use crate::mapper::KeyAction;

#[cfg(windows)]
pub fn inject_key(action: KeyAction) -> Result<(), String> {
    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, SendInput, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_NEXT, VK_PRIOR,
        VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
    };

    let vk = match action {
        KeyAction::Up => VK_UP,
        KeyAction::Down => VK_DOWN,
        KeyAction::Enter => VK_RETURN,
        KeyAction::Left => VK_LEFT,
        KeyAction::Right => VK_RIGHT,
        KeyAction::Tab => VK_TAB,
        KeyAction::Escape => VK_ESCAPE,
        KeyAction::PageUp => VK_PRIOR,
        KeyAction::PageDown => VK_NEXT,
        KeyAction::Home => VK_HOME,
        KeyAction::End => VK_END,
        KeyAction::Space => VK_SPACE,
    };

    let mut inputs = [keyboard_input(vk, false), keyboard_input(vk, true)];
    let sent = unsafe { SendInput(&mut inputs, std::mem::size_of::<INPUT>() as i32) };

    return if sent == inputs.len() as u32 {
        Ok(())
    } else {
        let error = unsafe { GetLastError() };
        Err(format!(
            "SendInput sent {sent}/{} events; GetLastError={}",
            inputs.len(),
            error.0
        ))
    };
}

#[cfg(windows)]
fn keyboard_input(
    vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY,
    key_up: bool,
) -> windows::Win32::UI::Input::KeyboardAndMouse::INPUT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    };

    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        Default::default()
    };

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(not(windows))]
pub fn inject_key(_action: KeyAction) -> Result<(), String> {
    Err("VibeKeys MVP only supports Windows".to_string())
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

    #[test]
    fn windows_input_size_matches_sendinput_expectation() {
        assert_eq!(std::mem::size_of::<INPUT>(), 40);
    }
}
