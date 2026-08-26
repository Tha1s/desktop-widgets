#![cfg(windows)]

use std::time::Duration;
use tauri::Manager;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreatePolygonRgn, SetWindowRgn, HRGN, WINDING};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetWindowLongW, GetWindowRect,
    IsWindowVisible, SendMessageW, SetParent, SetWindowLongPtrW, GWL_EXSTYLE, GWL_STYLE,
    GWLP_HWNDPARENT,
};

const WM_SPAWN_WORKERW: u32 = 0x052C;

type TopInfo = (HWND, String, bool, bool, (i32, i32, i32, i32));

const CIRCLE_CX: f64 = 200.0;
const CIRCLE_CY: f64 = 200.0;
const CIRCLE_R: f64 = 120.0;
const ROT_RAD: f64 = 45.0_f64.to_radians();

fn log(msg: &str) {
    let path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("workerw-proof.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("workerw-proof.log"));
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{}", msg);
    }
}

fn class_of(hwnd: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 128];
        let len = GetClassNameW(hwnd, &mut buf);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

fn has_defview_child(hwnd: HWND) -> bool {
    unsafe {
        matches!(
            FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), PCWSTR::null()),
            Ok(h) if !h.is_invalid()
        )
    }
}

unsafe extern "system" fn collect_top_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = lparam.0 as *mut Vec<TopInfo>;
    let class = class_of(hwnd);
    if class == "Progman"
        || class == "WorkerW"
        || class == "SHELLDLL_DefView"
        || class == "ApplicationFrameWindow"
    {
        let visible = IsWindowVisible(hwnd).as_bool();
        let has_dv = has_defview_child(hwnd);
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let _ = GetWindowRect(hwnd, &mut r);
        (*list).push((
            hwnd,
            class,
            visible,
            has_dv,
            (r.left, r.top, r.right, r.bottom),
        ));
    }
    BOOL(1)
}

unsafe extern "system" fn collect_workerw_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = lparam.0 as *mut Vec<HWND>;
    if class_of(hwnd) == "WorkerW" {
        (*list).push(hwnd);
    }
    BOOL(1)
}

fn collect_workerw_windows() -> Vec<HWND> {
    unsafe {
        let mut out: Vec<HWND> = Vec::new();
        let _ = EnumWindows(
            Some(collect_workerw_cb),
            LPARAM(&mut out as *mut Vec<HWND> as isize),
        );
        out
    }
}

fn log_window_identity(hwnd: HWND) {
    unsafe {
        let class = class_of(hwnd);
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let ex_layer = ex & 0x0008_0000 != 0; // WS_EX_LAYERED
        let ex_transparent = ex & 0x0000_0020 != 0; // WS_EX_TRANSPARENT
        let ex_noredir = ex & 0x0020_0000 != 0; // WS_EX_NOREDIRECTIONBITMAP
        log(&format!(
            "window: class='{class}' hwnd={hwnd:?} style=0x{style:08x} exstyle=0x{ex:08x} (layered={ex_layer} transparent={ex_transparent} noredir={ex_noredir})"
        ));
    }
}

/// Région = empreinte opaque : polygone approximant le cercle tourné à 45°,
/// converti en pixels physiques (scaling DPI).
fn build_circle_region(scale: f64) -> HRGN {
    const N: usize = 72;
    let cx = (CIRCLE_CX * scale) as i32;
    let cy = (CIRCLE_CY * scale) as i32;
    let r = CIRCLE_R * scale;
    let mut pts = [POINT { x: 0, y: 0 }; N];
    for i in 0..N {
        let a = i as f64 * std::f64::consts::TAU / N as f64 + ROT_RAD;
        let (sin, cos) = a.sin_cos();
        pts[i] = POINT {
            x: cx + (r * cos) as i32,
            y: cy + (r * sin) as i32,
        };
    }
    unsafe { CreatePolygonRgn(&pts, WINDING) }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let win = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    log("main window not found");
                    return Ok(());
                }
            };

            let hwnd = match win.hwnd() {
                Ok(h) => h,
                Err(e) => {
                    log(&format!("hwnd() failed: {e}"));
                    return Ok(());
                }
            };

            log("==== spike diagnostic ====");
            log_window_identity(hwnd);

            unsafe {
                let mut top: Vec<TopInfo> = Vec::new();
                let _ = EnumWindows(
                    Some(collect_top_cb),
                    LPARAM(&mut top as *mut Vec<TopInfo> as isize),
                );
                for (h, class, visible, has_dv, rect) in &top {
                    log(&format!(
                        "top: class='{class}' hwnd={h:?} visible={visible} hasDefViewChild={has_dv} rect={rect:?}"
                    ));
                }
            }

            let progman = FindWindowW(w!("Progman"), PCWSTR::null())
                .ok()
                .filter(|h| !h.is_invalid());
            match progman {
                Some(p) => log(&format!("Progman found = {p:?}")),
                None => log("Progman NOT FOUND"),
            }

            // (re)créer le WorkerW via 0x052C et re-collecter quelques fois
            let mut candidates = collect_workerw_windows();
            if let Some(p) = progman {
                for _ in 0..10 {
                    let _ = SendMessageW(p, WM_SPAWN_WORKERW, Some(WPARAM(0)), Some(LPARAM(0)));
                    let fresh = collect_workerw_windows();
                    if fresh.len() > candidates.len() {
                        candidates = fresh;
                    }
                    if !candidates.is_empty() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            log(&format!("WorkerW candidates: {}", candidates.len()));
            for ww in &candidates {
                let visible = IsWindowVisible(*ww).as_bool();
                let has_dv = has_defview_child(*ww);
                log(&format!("candidate WorkerW {ww:?} visible={visible} hasDefViewChild={has_dv}"));
            }

            let mut parented = false;
            for ww in &candidates {
                unsafe {
                    match SetParent(hwnd, Some(*ww)) {
                        Ok(prev) => {
                            log(&format!("SetParent -> {ww:?} OK (prev={prev:?})"));
                            parented = true;
                            break;
                        }
                        Err(e) => log(&format!("SetParent -> {ww:?} failed: {e:?}")),
                    }
                }
            }

            if !parented {
                unsafe {
                    if let Some(ww) = candidates.first() {
                        let prev = SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, ww.0 as isize);
                        log(&format!("SetWindowLongPtr(GWLP_HWNDPARENT, {ww:?}) -> prev={prev:?}"));
                    }
                }
                log("fallback always-on-bottom");
                let _ = win.set_always_on_bottom(true);
            }

            // Click-through : région de fenêtre = empreinte opaque du cercle tourné.
            unsafe {
                let scale = GetDpiForWindow(hwnd) as f64 / 96.0;
                let rgn = build_circle_region(scale);
                if rgn.is_invalid() {
                    log("CreatePolygonRgn failed");
                } else {
                    let res = SetWindowRgn(hwnd, Some(rgn), true);
                    log(&format!("SetWindowRgn ok (ret={res})"));
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running workerw-proof");
}
