use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Middle,
    X1,
    X2,
    Other,
}

impl MouseButton {
    pub fn config_key(&self) -> &'static str {
        match self {
            Self::Middle => "middle",
            Self::X1 => "x1",
            Self::X2 => "x2",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyAction {
    Up,
    Down,
    Enter,
    Left,
    Right,
    Tab,
    Escape,
    PageUp,
    PageDown,
    Home,
    End,
    Space,
}

impl KeyAction {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "enter" | "return" => Some(Self::Enter),
            "tab" => Some(Self::Tab),
            "escape" | "esc" => Some(Self::Escape),
            "pageup" | "page_up" => Some(Self::PageUp),
            "pagedown" | "page_down" => Some(Self::PageDown),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "space" => Some(Self::Space),
            _ => None,
        }
    }
}

impl std::fmt::Display for KeyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Enter => write!(f, "Enter"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Tab => write!(f, "Tab"),
            Self::Escape => write!(f, "Escape"),
            Self::PageUp => write!(f, "PageUp"),
            Self::PageDown => write!(f, "PageDown"),
            Self::Home => write!(f, "Home"),
            Self::End => write!(f, "End"),
            Self::Space => write!(f, "Space"),
        }
    }
}

#[cfg(test)]
fn map_mouse_button(button: MouseButton) -> Option<KeyAction> {
    match button {
        MouseButton::X1 => Some(KeyAction::Up),
        MouseButton::X2 => Some(KeyAction::Down),
        MouseButton::Middle => Some(KeyAction::Enter),
        MouseButton::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_default_mouse_buttons() {
        assert_eq!(map_mouse_button(MouseButton::X1), Some(KeyAction::Up));
        assert_eq!(map_mouse_button(MouseButton::X2), Some(KeyAction::Down));
        assert_eq!(
            map_mouse_button(MouseButton::Middle),
            Some(KeyAction::Enter)
        );
        assert_eq!(map_mouse_button(MouseButton::Other), None);
    }

    #[test]
    fn parses_key_actions() {
        assert_eq!(KeyAction::parse("up"), Some(KeyAction::Up));
        assert_eq!(KeyAction::parse("DOWN"), Some(KeyAction::Down));
        assert_eq!(KeyAction::parse("return"), Some(KeyAction::Enter));
        assert_eq!(KeyAction::parse("left"), Some(KeyAction::Left));
        assert_eq!(KeyAction::parse("f13"), None);
    }
}
