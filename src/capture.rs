use crate::config::AppConfig;
use crate::context;
use crate::diagnostics;
use crate::injector;
use crate::mapper::MouseButton;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum CaptureMode {
    Detect,
    Run {
        debug: bool,
        config: Arc<RwLock<AppConfig>>,
    },
}

#[cfg(windows)]
pub fn listen(mode: CaptureMode) -> Result<(), String> {
    use std::sync::OnceLock;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_XBUTTONDOWN,
        XBUTTON1, XBUTTON2,
    };

    static MODE: OnceLock<CaptureMode> = OnceLock::new();
    let _ = MODE.set(mode);

    unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code < 0 {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        let Some(mode) = MODE.get() else {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        };

        let button = unsafe { mouse_button_from_hook(w_param, l_param) };
        if button == MouseButton::Other {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        match mode {
            CaptureMode::Detect => {
                diagnostics::print_detected_button(button);
                unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
            }
            CaptureMode::Run { debug, config } => run_mapping(button, *debug, config)
                .unwrap_or_else(|| unsafe { CallNextHookEx(None, n_code, w_param, l_param) }),
        }
    }

    unsafe fn mouse_button_from_hook(w_param: WPARAM, l_param: LPARAM) -> MouseButton {
        match w_param.0 as u32 {
            WM_MBUTTONDOWN => MouseButton::Middle,
            WM_XBUTTONDOWN => {
                let hook = unsafe { &*(l_param.0 as *const MSLLHOOKSTRUCT) };
                let x_button = ((hook.mouseData >> 16) & 0xffff) as u16;
                if x_button == XBUTTON1 {
                    MouseButton::X1
                } else if x_button == XBUTTON2 {
                    MouseButton::X2
                } else {
                    MouseButton::Other
                }
            }
            _ => MouseButton::Other,
        }
    }

    fn run_mapping(
        button: MouseButton,
        debug: bool,
        shared_config: &Arc<RwLock<AppConfig>>,
    ) -> Option<LRESULT> {
        if debug {
            diagnostics::print_detected_button(button);
        }

        let config = shared_config.read().ok()?;
        let action = config.map_button(button);
        if debug {
            diagnostics::print_mapping(button, action);
        }

        let action = action?;
        let foreground = match context::foreground_window() {
            Ok(foreground) => foreground,
            Err(error) => {
                if debug {
                    println!("context: failed: {error}");
                }
                return None;
            }
        };

        if debug {
            diagnostics::print_context(&foreground);
        }

        if !config.applies_to_process(&foreground.process_name) {
            if debug {
                println!("decision: pass through original mouse event");
            }
            return None;
        }

        let result = injector::inject_key(action);
        let ok = result.is_ok();
        if debug {
            diagnostics::print_injection(action, result);
        }

        if ok { Some(LRESULT(1)) } else { None }
    }

    unsafe {
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)
            .map_err(|err| format!("SetWindowsHookExW failed: {err}"))?;
        println!("listening for mouse events; press Ctrl+C to stop");

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let _ = UnhookWindowsHookEx(hook);
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn listen(_mode: CaptureMode) -> Result<(), String> {
    Err("VibeKeys MVP only supports Windows".to_string())
}
