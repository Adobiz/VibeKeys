use std::sync::{Arc, RwLock};

use serde::Deserialize;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use wry::{WebContext, WebViewBuilder};

use crate::capture::{self, CaptureMode};
use crate::config::{self, AppConfig};

const INDEX_HTML: &str = include_str!("../ui/index.html");

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcMessage {
    SaveConfig { config: AppConfig },
}

#[derive(Debug)]
enum DesktopEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

pub fn launch(debug: bool) -> Result<(), String> {
    if restore_existing_instance() {
        return Ok(());
    }

    let path = config::app_config_path();
    let initial = if path.exists() {
        AppConfig::load(Some(&path))?
    } else {
        let config = AppConfig::load(None)?;
        config.save(&path)?;
        config
    };
    let shared_config = Arc::new(RwLock::new(initial.clone()));
    let capture_config = Arc::clone(&shared_config);
    std::thread::spawn(move || {
        if let Err(error) = capture::listen(CaptureMode::Run {
            debug,
            config: capture_config,
        }) {
            eprintln!("capture stopped: {error}");
        }
    });

    let devices = crate::devices::list_mouse_devices().unwrap_or_default();
    let bootstrap = serde_json::json!({
        "config": initial,
        "devices": devices,
    });
    let bootstrap = bootstrap.to_string().replace('<', "\\u003c");
    let html = INDEX_HTML.replace("__VIBEKEYS_BOOTSTRAP__", &bootstrap);

    let event_loop = EventLoopBuilder::<DesktopEvent>::with_user_event().build();
    let window_icon = tao::window::Icon::from_rgba(crate::icon_pixels::vk_icon_rgba(64), 64, 64)
        .map_err(|err| format!("failed to build window icon: {err}"))?;
    let window = WindowBuilder::new()
        .with_title("VibeKeys")
        .with_window_icon(Some(window_icon))
        .with_inner_size(LogicalSize::new(1060.0, 720.0))
        .with_min_inner_size(LogicalSize::new(820.0, 600.0))
        .build(&event_loop)
        .map_err(|err| format!("failed to create window: {err}"))?;

    let ipc_config = Arc::clone(&shared_config);
    let ipc_path = path.clone();
    let mut web_context = WebContext::new(Some(config::webview_data_path()));
    let _webview = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_html(&html)
        .with_ipc_handler(move |request| {
            let result = serde_json::from_str::<IpcMessage>(request.body());
            match result {
                Ok(IpcMessage::SaveConfig { config }) => {
                    if let Err(error) = config.save(&ipc_path) {
                        eprintln!("failed to save config: {error}");
                        return;
                    }
                    if let Ok(mut current) = ipc_config.write() {
                        *current = config;
                    }
                }
                Err(error) => eprintln!("invalid UI message: {error}"),
            }
        })
        .build(&window)
        .map_err(|err| format!("failed to create desktop view: {err}"))?;

    let tray_menu = Menu::new();
    let open_item = MenuItem::new("打开 VibeKeys", true, None);
    let separator = PredefinedMenuItem::separator();
    let exit_item = MenuItem::new("退出", true, None);
    tray_menu
        .append_items(&[&open_item, &separator, &exit_item])
        .map_err(|err| format!("failed to create tray menu: {err}"))?;
    let open_id = open_item.id().clone();
    let exit_id = exit_item.id().clone();
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .with_tooltip("VibeKeys - 映射运行中")
        .with_icon(build_tray_icon()?)
        .build()
        .map_err(|err| format!("failed to create tray icon: {err}"))?;

    let tray_proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(DesktopEvent::Tray(event));
    }));
    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(DesktopEvent::Menu(event));
    }));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } if size.width == 0 || size.height == 0 => {
                window.set_visible(false);
            }
            Event::UserEvent(DesktopEvent::Tray(
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                },
            )) => restore_window(&window),
            Event::UserEvent(DesktopEvent::Menu(event)) if event.id == open_id => {
                restore_window(&window);
            }
            Event::UserEvent(DesktopEvent::Menu(event)) if event.id == exit_id => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn restore_window(window: &tao::window::Window) {
    window.set_visible(true);
    window.set_minimized(false);
    window.set_focus();
}

#[cfg(windows)]
fn restore_existing_instance() -> bool {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SW_RESTORE, SetForegroundWindow, ShowWindow,
    };
    use windows::core::w;

    unsafe {
        let Ok(window) = FindWindowW(None, w!("VibeKeys")) else {
            return false;
        };
        if window == HWND(std::ptr::null_mut()) {
            return false;
        }
        let _ = ShowWindow(window, SW_RESTORE);
        let _ = SetForegroundWindow(window);
        true
    }
}

#[cfg(not(windows))]
fn restore_existing_instance() -> bool {
    false
}

fn build_tray_icon() -> Result<tray_icon::Icon, String> {
    tray_icon::Icon::from_rgba(crate::icon_pixels::vk_icon_rgba(32), 32, 32)
        .map_err(|err| format!("failed to build tray icon pixels: {err}"))
}
