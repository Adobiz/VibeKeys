pub fn vk_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let margin = (size / 16).max(1);
    let radius = (size / 5).max(2);

    for y in margin..size - margin {
        for x in margin..size - margin {
            let dx = if x < margin + radius {
                margin + radius - x
            } else if x >= size - margin - radius {
                x - (size - margin - radius - 1)
            } else {
                0
            };
            let dy = if y < margin + radius {
                margin + radius - y
            } else if y >= size - margin - radius {
                y - (size - margin - radius - 1)
            } else {
                0
            };
            if dx * dx + dy * dy <= radius * radius {
                set_pixel(&mut rgba, size, x, y, [23, 25, 24, 255]);
            }
        }
    }

    const V: [&str; 7] = [
        "10001", "10001", "10001", "10001", "01010", "01010", "00100",
    ];
    const K: [&str; 7] = [
        "10001", "10010", "10100", "11000", "10100", "10010", "10001",
    ];
    let scale = (size / 16).max(1);
    let glyph_width = 5 * scale;
    let gap = scale;
    let total_width = glyph_width * 2 + gap;
    let start_x = (size - total_width) / 2;
    let start_y = (size - 7 * scale) / 2;
    paint_glyph(
        &mut rgba,
        size,
        start_x,
        start_y,
        scale,
        &V,
        [255, 255, 255, 255],
    );
    paint_glyph(
        &mut rgba,
        size,
        start_x + glyph_width + gap,
        start_y,
        scale,
        &K,
        [52, 190, 142, 255],
    );
    rgba
}

fn paint_glyph(
    rgba: &mut [u8],
    size: u32,
    start_x: u32,
    start_y: u32,
    scale: u32,
    glyph: &[&str; 7],
    color: [u8; 4],
) {
    for (row, pattern) in glyph.iter().enumerate() {
        for (column, pixel) in pattern.bytes().enumerate() {
            if pixel != b'1' {
                continue;
            }
            for offset_y in 0..scale {
                for offset_x in 0..scale {
                    set_pixel(
                        rgba,
                        size,
                        start_x + column as u32 * scale + offset_x,
                        start_y + row as u32 * scale + offset_y,
                        color,
                    );
                }
            }
        }
    }
}

fn set_pixel(rgba: &mut [u8], size: u32, x: u32, y: u32, color: [u8; 4]) {
    if x >= size || y >= size {
        return;
    }
    let index = ((y * size + x) * 4) as usize;
    rgba[index..index + 4].copy_from_slice(&color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_contains_background_and_both_letter_colors() {
        let rgba = vk_icon_rgba(32);
        assert_eq!(rgba.len(), 32 * 32 * 4);
        assert!(rgba.chunks_exact(4).any(|pixel| pixel == [23, 25, 24, 255]));
        assert!(
            rgba.chunks_exact(4)
                .any(|pixel| pixel == [255, 255, 255, 255])
        );
        assert!(
            rgba.chunks_exact(4)
                .any(|pixel| pixel == [52, 190, 142, 255])
        );
    }
}
