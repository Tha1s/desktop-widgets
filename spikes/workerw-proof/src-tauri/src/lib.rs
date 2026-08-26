#![cfg(windows)]

use std::time::Duration;
use tauri::Manager;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreatePolygonRgn, SetWindowRgn, HRGN, WINDING};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, SendMessageW, SetParent,
};

const WM_SPAWN_WORKERW: u32 = 0x052C;

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

unsafe extern "system" fn enum_workerw_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut buf = [0u16; 128];
    let len = GetClassNameW(hwnd, &mut buf);
    if len == 0 {
        return BOOL(1);
    }
    let class = String::from_utf16_lossy(&buf[..len as usize]);
    if class == "WorkerW" {
        let defview = FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), PCWSTR::null());
        if let Ok(defview) = defview {
            if !defview.is_invalid() {
                let slot = lparam.0 as *mut Option<HWND>;
                (*slot) = Some(hwnd);
                return BOOL(0);
            }
        }
    }
    BOOL(1)
}

fn find_workerw() -> Option<HWND> {
    unsafe {
        let progman = match FindWindowW(w!("Progman"), PCWSTR::null()) {
            Ok(h) if !h.is_invalid() => h,
            _ => {
                log("Progman NOT FOUND");
                return None;
            }
        };
        log(&format!("Progman found = {progman:?}"));

        for attempt in 0..40 {
            // demander à explorer.exe de (re)créer le WorkerW (message async)
            let _ = SendMessageW(progman, WM_SPAWN_WORKERW, Some(WPARAM(0)), Some(LPARAM(0)));

            // 1) un WorkerW contenant SHELLDLL_DefView (structure classique)
            let mut by_enum: Option<HWND> = None;
            let _ = EnumWindows(
                Some(enum_workerw_cb),
                LPARAM(&mut by_enum as *mut Option<HWND> as isize),
            );
            if let Some(ww) = by_enum {
                log(&format!(
                    "WorkerW (avec SHELLDLL_DefView) via EnumWindows, tentative {attempt}: {ww:?}"
                ));
                return Some(ww);
            }

            // 2) repli : n'importe quel WorkerW top-level
            if let Ok(ww) = FindWindowExW(None, None, w!("WorkerW"), PCWSTR::null()) {
                if !ww.is_invalid() {
                    log(&format!(
                        "WorkerW (n'importe lequel) via FindWindowEx, tentative {attempt}: {ww:?}"
                    ));
                    return Some(ww);
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
        log("WorkerW NOT FOUND après 40 tentatives");
        None
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

            match find_workerw() {
                Some(workerw) => unsafe {
                    match SetParent(hwnd, Some(workerw)) {
                        Ok(prev) => log(&format!("SetParent -> WorkerW ok (prev={prev:?})")),
                        Err(e) => log(&format!("SetParent failed: {e}")),
                    }
                },
                None => {
                    log("WorkerW NOT FOUND — fallback always-on-bottom");
                    let _ = win.set_always_on_bottom(true);
                }
            }

            // Click-through : la région de fenêtre = empreinte opaque du cercle tourné.
            // Les pixels hors région ne font pas partie de la fenêtre -> les clics
            // passent au bureau. (WM_NCHITTEST + HTTRANSPARENT ne suffit pas pour une
            // fenêtre top-level vers une autre thread.)
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
