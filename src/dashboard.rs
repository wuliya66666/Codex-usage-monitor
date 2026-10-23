//! Compact side-by-side ring dashboard. GDI+ provides antialiased vectors;
//! text is rasterized by Windows using the current monitor's physical DPI.
use std::{ffi::c_void, sync::OnceLock};
use windows::core::PCWSTR;
use windows::Win32::{Foundation::*, Graphics::Gdi::*};

type Native = *mut c_void;
pub const FONT_FACE: &str = "Source Han Sans HW SC";
pub fn load_font() {
    static FONT: OnceLock<Option<usize>> = OnceLock::new();
    FONT.get_or_init(|| unsafe {
        let bytes = include_bytes!("../assets/fonts/SourceHanSansHWSC-Bold.otf");
        let mut count = 0;
        let handle = AddFontMemResourceEx(
            bytes.as_ptr() as *const c_void,
            bytes.len() as u32,
            None,
            &mut count,
        );
        if handle.is_invalid() {
            None
        } else {
            Some(handle.0 as usize)
        }
    });
}
#[repr(C)]
struct Startup {
    version: u32,
    callback: Native,
    suppress_thread: i32,
    suppress_codecs: i32,
}
#[link(name = "gdiplus")]
extern "system" {
    fn GdiplusStartup(token: *mut usize, input: *const Startup, output: Native) -> i32;
    fn GdipCreateFromHDC(hdc: Native, graphics: *mut Native) -> i32;
    fn GdipDeleteGraphics(graphics: Native) -> i32;
    fn GdipSetSmoothingMode(graphics: Native, mode: i32) -> i32;
    fn GdipCreatePen1(color: u32, width: f32, unit: i32, pen: *mut Native) -> i32;
    fn GdipDeletePen(pen: Native) -> i32;
    fn GdipSetPenLineCap197819(pen: Native, start: i32, end: i32, dash: i32) -> i32;
    fn GdipDrawArc(
        g: Native,
        pen: Native,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        start: f32,
        sweep: f32,
    ) -> i32;
    fn GdipDrawLine(g: Native, pen: Native, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
}

pub const PANEL_WIDTH: i32 = 86;
pub const GAP: i32 = 12;
pub const PAD: i32 = 9;
pub fn logical_width(session: bool, weekly: bool) -> i32 {
    let count = session as i32 + weekly as i32;
    2 * PAD + PANEL_WIDTH * count.max(1) + GAP * (count - 1).max(0)
}

unsafe fn pen(g: Native, color: u32, width: f32, draw: impl FnOnce(Native, Native)) {
    let mut p = std::ptr::null_mut();
    if GdipCreatePen1(color, width, 2, &mut p) == 0 {
        GdipSetPenLineCap197819(p, 2, 2, 0);
        draw(g, p);
        GdipDeletePen(p);
    }
}

unsafe fn line(g: Native, p: Native, a: (f32, f32), b: (f32, f32)) {
    GdipDrawLine(g, p, a.0, a.1, b.0, b.1);
}

unsafe fn label(hdc: HDC, text: &str, rect: RECT, height: i32, _weight: i32, color: u32) {
    // An empty Vec exposes a dangling (non-null) pointer. DrawTextW on
    // Windows 10 can read that pointer even with a zero character count.
    // Startup placeholders and provider errors have no reset-time label.
    if text.is_empty() {
        return;
    }
    load_font();
    let face: Vec<u16> = FONT_FACE.encode_utf16().chain(Some(0)).collect();
    let font = CreateFontW(
        -height,
        0,
        0,
        0,
        FW_BOLD.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET.0 as u32,
        OUT_TT_PRECIS.0 as u32,
        CLIP_DEFAULT_PRECIS.0 as u32,
        ANTIALIASED_QUALITY.0 as u32,
        0,
        PCWSTR(face.as_ptr()),
    );
    let old = SelectObject(hdc, font);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, COLORREF(color));
    let mut text: Vec<u16> = text.encode_utf16().collect();
    let mut rect = rect;
    DrawTextW(
        hdc,
        &mut text,
        &mut rect,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
    SelectObject(hdc, old);
    let _ = DeleteObject(font);
}

fn display_parts(value: &str) -> (&str, String) {
    match value.split_once('%') {
        Some((percent, rest)) => (
            &value[..percent.len() + 1],
            rest.trim_start_matches([' ', '·'])
                .replace("重置", "")
                .trim()
                .to_string(),
        ),
        None => (value, String::new()),
    }
}

/// `scale` maps logical coordinates to the current monitor's physical pixels.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    hdc: HDC,
    width: i32,
    height: i32,
    dark: bool,
    large: bool,
    session: (f64, &str),
    weekly: (f64, &str),
    show_session: bool,
    show_weekly: bool,
    scale: impl Fn(i32) -> i32 + Copy,
) {
    if width <= 0 || height <= 0 {
        return;
    }
    // Render at three times the physical resolution, then average each 3x3
    // pixel block. This smooths both vectors and text on a transparent layer.
    unsafe {
        let factor = 3;
        let hi_w = width * factor;
        let hi_h = height * factor;
        let dc = CreateCompatibleDC(hdc);
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: hi_w,
                biHeight: -hi_h,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let Ok(bitmap) = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = DeleteDC(dc);
            return;
        };
        let old = SelectObject(dc, bitmap);
        let brush = CreateSolidBrush(GetPixel(hdc, 0, 0));
        FillRect(
            dc,
            &RECT {
                left: 0,
                top: 0,
                right: hi_w,
                bottom: hi_h,
            },
            brush,
        );
        draw_native(
            dc,
            hi_w,
            hi_h,
            dark,
            large,
            session,
            weekly,
            show_session,
            show_weekly,
            |v| scale(v) * factor,
        );
        let _ = GdiFlush();
        let pixels = std::slice::from_raw_parts(bits as *const u32, (hi_w * hi_h) as usize);
        let mut output = vec![0u32; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let mut channels = [0u32; 3];
                for dy in 0..factor {
                    for dx in 0..factor {
                        let pixel = pixels[((y * factor + dy) * hi_w + x * factor + dx) as usize];
                        for (index, sum) in channels.iter_mut().enumerate() {
                            *sum += (pixel >> (index * 8)) & 255;
                        }
                    }
                }
                output[(y * width + x) as usize] =
                    channels.iter().enumerate().fold(0, |pixel, (index, sum)| {
                        pixel | ((sum + 4) / 9) << (index * 8)
                    });
            }
        }
        info.bmiHeader.biWidth = width;
        info.bmiHeader.biHeight = -height;
        SetDIBitsToDevice(
            hdc,
            0,
            0,
            width as u32,
            height as u32,
            0,
            0,
            0,
            height as u32,
            output.as_ptr() as *const c_void,
            &info,
            DIB_RGB_COLORS,
        );
        SelectObject(dc, old);
        let _ = DeleteObject(bitmap);
        let _ = DeleteObject(brush);
        let _ = DeleteDC(dc);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_native(
    hdc: HDC,
    width: i32,
    height: i32,
    dark: bool,
    large: bool,
    session: (f64, &str),
    weekly: (f64, &str),
    show_session: bool,
    show_weekly: bool,
    scale: impl Fn(i32) -> i32 + Copy,
) {
    static TOKEN: OnceLock<usize> = OnceLock::new();
    let token = *TOKEN.get_or_init(|| unsafe {
        let mut token = 0;
        let input = Startup {
            version: 1,
            callback: std::ptr::null_mut(),
            suppress_thread: 0,
            suppress_codecs: 0,
        };
        if GdiplusStartup(&mut token, &input, std::ptr::null_mut()) == 0 {
            token
        } else {
            0
        }
    });
    if token == 0 {
        return;
    }
    unsafe {
        let mut g = std::ptr::null_mut();
        if GdipCreateFromHDC(hdc.0, &mut g) != 0 {
            return;
        }
        GdipSetSmoothingMode(g, 4);
        let factor = scale(100) as f32 / 100.0;
        let center_y = height as f32 / 2.0;
        let diameter = (scale(27) as f32).min(height as f32 - scale(11) as f32);
        let text_height = scale(if large { 15 } else { 14 });
        let detail_height = scale(12);
        let mut left = scale(PAD);
        if show_session && show_weekly {
            let mid = width as f32 / 2.0 - scale(8) as f32;
            pen(
                g,
                if dark { 0x60566A7B } else { 0x50566A7B },
                factor,
                |g, p| {
                    line(
                        g,
                        p,
                        (mid, center_y - scale(11) as f32),
                        (mid, center_y + scale(11) as f32),
                    )
                },
            );
        }
        // The surrounding taskbar supplies the glass background. Avoid an
        // opaque card or outer border so the component blends into the shell.
        for (weekly_icon, (percent, value), visible) in
            [(false, session, show_session), (true, weekly, show_weekly)]
        {
            if !visible {
                continue;
            }
            let cx = left as f32 + scale(31) as f32 / 2.0;
            let cy = center_y;
            let color = if weekly_icon { 0xFF00E2AB } else { 0xFF00DCCC };
            let text_color = if weekly_icon { 0x00ABE200 } else { 0x00CCDC00 };
            let track = if dark { 0xA03D4A58 } else { 0x70596B78 };
            let thickness = 2.7 * factor;
            pen(g, track, thickness, |g, p| {
                GdipDrawArc(
                    g,
                    p,
                    cx - diameter / 2.0,
                    cy - diameter / 2.0,
                    diameter,
                    diameter,
                    0.0,
                    360.0,
                );
            });
            let fill = if value == "--" || !value.contains('%') {
                0.0
            } else {
                percent.clamp(0.0, 100.0) as f32
            };
            if fill > 0.0 {
                pen(g, color, thickness, |g, p| {
                    GdipDrawArc(
                        g,
                        p,
                        cx - diameter / 2.0,
                        cy - diameter / 2.0,
                        diameter,
                        diameter,
                        -90.0,
                        fill * 3.6,
                    );
                });
            }
            pen(g, color, 1.3 * factor, |g, p| {
                let at =
                    |x: f32, y: f32| (cx + x * factor * 27.0 / 31.0, cy + y * factor * 27.0 / 31.0);
                if weekly_icon {
                    let points = [
                        (-4.5, -3.5),
                        (4.5, -3.5),
                        (4.5, 5.0),
                        (-4.5, 5.0),
                        (-4.5, -3.5),
                    ];
                    for pair in points.windows(2) {
                        line(g, p, at(pair[0].0, pair[0].1), at(pair[1].0, pair[1].1));
                    }
                    line(g, p, at(-4.5, -0.8), at(4.5, -0.8));
                    line(g, p, at(-2.5, -5.0), at(-2.5, -2.5));
                    line(g, p, at(2.5, -5.0), at(2.5, -2.5));
                    line(g, p, at(-1.8, 2.0), at(1.8, 2.0));
                } else {
                    let points = [
                        (1.5, -6.0),
                        (-4.0, 1.0),
                        (0.5, 1.0),
                        (-1.5, 6.0),
                        (4.0, -1.0),
                        (-0.5, -1.0),
                        (1.5, -6.0),
                    ];
                    for pair in points.windows(2) {
                        line(g, p, at(pair[0].0, pair[0].1), at(pair[1].0, pair[1].1));
                    }
                }
            });
            // Complete GDI+ work before drawing GDI text onto the same DC.
            let _ = GdipDeleteGraphics(g);
            g = std::ptr::null_mut();
            let (percent_text, reset) = display_parts(value);
            let x = left + scale(40);
            let right = left + scale(PANEL_WIDTH);
            let top = (height - scale(31)) / 2 - scale(1);
            label(
                hdc,
                percent_text,
                RECT {
                    left: x,
                    top,
                    right,
                    bottom: top + scale(17),
                },
                text_height,
                FW_SEMIBOLD.0 as i32,
                text_color,
            );
            label(
                hdc,
                &reset,
                RECT {
                    left: x,
                    top: top + scale(17),
                    right,
                    bottom: top + scale(32),
                },
                detail_height,
                FW_NORMAL.0 as i32,
                if dark { 0x00B5BCC6 } else { 0x00525B66 },
            );
            left += scale(PANEL_WIDTH + GAP);
            if GdipCreateFromHDC(hdc.0, &mut g) != 0 {
                return;
            }
            GdipSetSmoothingMode(g, 4);
        }
        GdipDeleteGraphics(g);
    }
}

/// Render representative data without touching account credentials or taskbar.
pub fn preview(path: &str) -> Result<(), String> {
    unsafe {
        let preview_scale = if std::env::args().any(|arg| arg == "--native-size") {
            1
        } else {
            3
        };
        let width = logical_width(true, true) * preview_scale;
        let height = 46 * preview_scale;
        let dc = CreateCompatibleDC(HDC::default());
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: 40,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
            .map_err(|e| e.to_string())?;
        let old = SelectObject(dc, bitmap);
        let brush = CreateSolidBrush(COLORREF(0x001A1410));
        FillRect(
            dc,
            &RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
            brush,
        );
        let startup = std::env::args().any(|arg| arg == "--startup-state");
        draw(
            dc,
            width,
            height,
            true,
            true,
            (75.0, if startup { "--" } else { "75% 01:21重置" }),
            (83.0, if startup { "--" } else { "83% 10/27重置" }),
            true,
            true,
            |v| v * preview_scale,
        );
        let _ = GdiFlush();
        let bytes = std::slice::from_raw_parts(bits as *const u8, (width * height * 4) as usize);
        let mut bmp = Vec::new();
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(54 + bytes.len() as u32).to_le_bytes());
        bmp.extend_from_slice(&[0; 4]);
        bmp.extend_from_slice(&54u32.to_le_bytes());
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&width.to_le_bytes());
        bmp.extend_from_slice(&(-height).to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes());
        bmp.extend_from_slice(&32u16.to_le_bytes());
        bmp.extend_from_slice(&[0; 24]);
        bmp.extend_from_slice(bytes);
        let result = std::fs::write(path, bmp).map_err(|e| e.to_string());
        SelectObject(dc, old);
        let _ = DeleteObject(bitmap);
        let _ = DeleteObject(brush);
        let _ = DeleteDC(dc);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requested_source_han_bold_is_selected_without_font_fallback() {
        load_font();
        unsafe {
            let dc = CreateCompatibleDC(HDC::default());
            let face: Vec<u16> = FONT_FACE.encode_utf16().chain(Some(0)).collect();
            let font = CreateFontW(
                -36,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET.0 as u32,
                0,
                0,
                ANTIALIASED_QUALITY.0 as u32,
                0,
                PCWSTR(face.as_ptr()),
            );
            let old = SelectObject(dc, font);
            let mut name = [0u16; 128];
            let count = GetTextFaceW(dc, Some(&mut name));
            let selected = String::from_utf16_lossy(&name[..count.saturating_sub(1) as usize]);
            let mut metrics = TEXTMETRICW::default();
            GetTextMetricsW(dc, &mut metrics).unwrap();
            SelectObject(dc, old);
            let _ = DeleteObject(font);
            let _ = DeleteDC(dc);
            assert!(
                selected == FONT_FACE || selected == "思源黑体 HW",
                "unexpected font fallback: {selected}"
            );
            assert_eq!(metrics.tmWeight, FW_BOLD.0 as i32);
        }
    }
    #[test]
    fn startup_and_error_labels_render_without_reset_time() {
        unsafe {
            let dc = CreateCompatibleDC(HDC::default());
            let bitmap = CreateBitmap(420, 92, 1, 32, None);
            let old = SelectObject(dc, bitmap);
            for value in ["--", "...", "!", "NET", "75%"] {
                draw(
                    dc,
                    420,
                    92,
                    true,
                    true,
                    (0.0, value),
                    (0.0, value),
                    true,
                    true,
                    |v| v * 2,
                );
            }
            let _ = GdiFlush();
            SelectObject(dc, old);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(dc);
        }
    }
    #[test]
    fn compact_display_preserves_status_and_reset_dates() {
        assert_eq!(display_parts("75% 01:21重置"), ("75%", "01:21".into()));
        assert_eq!(display_parts("83% 10/27重置 *"), ("83%", "10/27 *".into()));
        assert_eq!(display_parts("--"), ("--", String::new()));
    }
    #[test]
    fn hiding_a_window_reclaims_its_panel_and_separator() {
        assert_eq!(logical_width(true, false), 104);
        assert_eq!(logical_width(true, true), 202);
    }
}
