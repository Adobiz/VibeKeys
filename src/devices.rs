use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MouseDevice {
    pub name: String,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub brand: String,
}

#[cfg(windows)]
pub fn list_mouse_devices() -> Result<Vec<MouseDevice>, String> {
    use windows::Win32::UI::Input::{GetRawInputDeviceList, RAWINPUTDEVICELIST, RIM_TYPEMOUSE};

    unsafe {
        let mut count = 0u32;
        let size = std::mem::size_of::<RAWINPUTDEVICELIST>() as u32;
        let result = GetRawInputDeviceList(None, &mut count, size);
        if result == u32::MAX {
            return Err("GetRawInputDeviceList failed while counting devices".to_string());
        }

        let mut devices = vec![RAWINPUTDEVICELIST::default(); count as usize];
        let result = GetRawInputDeviceList(Some(devices.as_mut_ptr()), &mut count, size);
        if result == u32::MAX {
            return Err("GetRawInputDeviceList failed while reading devices".to_string());
        }

        let mut mice = Vec::new();
        for device in devices.into_iter().take(count as usize) {
            if device.dwType != RIM_TYPEMOUSE {
                continue;
            }
            let name = raw_device_name(device.hDevice)?;
            let (vendor_id, product_id) = parse_vid_pid(&name);
            let brand = brand_from_vendor(vendor_id.as_deref()).to_string();
            mice.push(MouseDevice {
                name,
                vendor_id,
                product_id,
                brand,
            });
        }
        return Ok(mice);
    }
}

#[cfg(windows)]
unsafe fn raw_device_name(handle: windows::Win32::Foundation::HANDLE) -> Result<String, String> {
    use windows::Win32::UI::Input::{GetRawInputDeviceInfoW, RIDI_DEVICENAME};

    let mut chars = 0u32;
    let first = unsafe { GetRawInputDeviceInfoW(Some(handle), RIDI_DEVICENAME, None, &mut chars) };
    if first == u32::MAX {
        return Err("GetRawInputDeviceInfoW failed while counting name length".to_string());
    }

    let mut buffer = vec![0u16; chars as usize];
    let second = unsafe {
        GetRawInputDeviceInfoW(
            Some(handle),
            RIDI_DEVICENAME,
            Some(buffer.as_mut_ptr().cast()),
            &mut chars,
        )
    };
    if second == u32::MAX {
        return Err("GetRawInputDeviceInfoW failed while reading name".to_string());
    }

    let len = buffer
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

#[cfg(not(windows))]
pub fn list_mouse_devices() -> Result<Vec<MouseDevice>, String> {
    Err("VibeKeys MVP only supports Windows".to_string())
}

pub fn parse_vid_pid(name: &str) -> (Option<String>, Option<String>) {
    let upper = name.to_ascii_uppercase();
    (
        find_hex_after(&upper, "VID_"),
        find_hex_after(&upper, "PID_"),
    )
}

pub fn brand_from_vendor(vendor_id: Option<&str>) -> &'static str {
    match vendor_id.unwrap_or_default().to_ascii_uppercase().as_str() {
        "046D" => "Logitech",
        "1532" => "Razer",
        "1B1C" => "Corsair",
        "1038" => "SteelSeries",
        "045E" => "Microsoft",
        "17EF" => "Lenovo",
        "04D9" => "Generic HID",
        "" => "Unknown",
        _ => "Unknown",
    }
}

fn find_hex_after(value: &str, prefix: &str) -> Option<String> {
    let start = value.find(prefix)? + prefix.len();
    let hex: String = value[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_hexdigit())
        .take(4)
        .collect();
    if hex.len() == 4 { Some(hex) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vid_pid_from_raw_input_name() {
        let (vid, pid) = parse_vid_pid(r#"\\?\HID#VID_046D&PID_C539&MI_00#abc"#);
        assert_eq!(vid.as_deref(), Some("046D"));
        assert_eq!(pid.as_deref(), Some("C539"));
        assert_eq!(brand_from_vendor(vid.as_deref()), "Logitech");
    }
}
