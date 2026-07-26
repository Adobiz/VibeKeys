use crate::context::ForegroundWindow;
use crate::mapper::{KeyAction, MouseButton};

pub fn print_context(context: &ForegroundWindow) {
    println!("Foreground window");
    println!("  title: {}", empty_dash(&context.title));
    println!("  process: {}", empty_dash(&context.process_name));
    println!("  pid: {}", context.process_id);
    println!("  terminal: {}", yes_no(context.is_terminal));
}

pub fn print_detected_button(button: MouseButton) {
    println!("detected mouse button: {button:?}");
}

pub fn print_mapping(button: MouseButton, action: Option<KeyAction>) {
    match action {
        Some(action) => println!("mapping: {button:?} -> {action}"),
        None => println!("mapping: {button:?} -> no action"),
    }
}

pub fn print_injection(action: KeyAction, result: Result<(), String>) {
    match result {
        Ok(()) => println!("inject: {action} -> ok"),
        Err(error) => println!("inject: {action} -> failed: {error}"),
    }
}

fn empty_dash(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
