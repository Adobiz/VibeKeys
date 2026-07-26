#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod context;
mod desktop;
mod devices;
mod diagnostics;
mod icon_pixels;
mod injector;
mod mapper;

use capture::CaptureMode;
use config::AppConfig;
use mapper::KeyAction;
use std::sync::{Arc, RwLock};

fn main() {
    if let Err(error) = run_cli(std::env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run_cli(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("detect") => capture::listen(CaptureMode::Detect),
        Some("context") => {
            let foreground = context::foreground_window()?;
            diagnostics::print_context(&foreground);
            Ok(())
        }
        Some("inject") => {
            let Some(action_arg) = args.get(1) else {
                return Err("usage: vibekeys inject <up|down|enter>".to_string());
            };
            let action = KeyAction::parse(action_arg)
                .ok_or_else(|| "usage: vibekeys inject <up|down|enter>".to_string())?;
            injector::inject_key(action)?;
            println!("injected {action}");
            Ok(())
        }
        Some("devices") => {
            for device in devices::list_mouse_devices()? {
                println!("brand: {}", device.brand);
                println!("  vid: {}", device.vendor_id.as_deref().unwrap_or("-"));
                println!("  pid: {}", device.product_id.as_deref().unwrap_or("-"));
                println!("  name: {}", device.name);
            }
            Ok(())
        }
        Some("init-config") => {
            let path = config::default_config_path();
            AppConfig::save_default(&path)?;
            println!("wrote {}", path.display());
            Ok(())
        }
        Some("run") => {
            let debug = args.iter().any(|arg| arg == "--debug");
            let config = Arc::new(RwLock::new(AppConfig::load(None)?));
            capture::listen(CaptureMode::Run { debug, config })
        }
        Some("app") | None => {
            let debug = args.iter().any(|arg| arg == "--debug");
            desktop::launch(debug)
        }
        Some("-h") | Some("--help") => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown command: {command}")),
    }
}

fn print_help() {
    println!("VibeKeys Windows-first MVP");
    println!();
    println!("Usage:");
    println!("  vibekeys                 Open the app and start mappings");
    println!("  vibekeys app [--debug]   Open the app and start mappings");
    println!("  vibekeys detect");
    println!("  vibekeys context");
    println!("  vibekeys devices");
    println!("  vibekeys inject <action>");
    println!("  vibekeys init-config");
    println!("  vibekeys run [--debug]");
}
