#![cfg(windows)]

use tauri::Manager;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{CreatePolygonRgn, SetWindowRgn, HRGN, WINDING};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetWindowLongPtrW, IsWindowVisible,
    SetParent, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_BOTTOM, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE,
};

const WS_EX_TOPMOST: isize = 0x0000_0010;

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

unsafe extern "system" fn find_defview_workerw_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if class_of(hwnd) == "WorkerW" && has_defview_child(hwnd) {
        let slot = lparam.0 as *mut Option<HWND>;
        (*slot) = Some(hwnd);
        return BOOL(0);
    }
    BOOL(1)
}

/// Cible de parentage : un WorkerW contenant SHELLDLL_DefView (cas standard),
/// sinon Progman (Win11 courant : les icônes vivent sous Progman).
/// Renvoie (hwnd, est_un_WorkerW).
fn find_parent_target() -> Option<(HWND, bool)> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), PCWSTR::null())
            .ok()
            .filter(|h| !h.is_invalid())?;

        let mut by_enum: Option<HWND> = None;
        let _ = EnumWindows(
            Some(find_defview_workerw_cb),
            LPARAM(&mut by_enum as *mut Option<HWND> as isize),
        );
        if let Some(ww) = by_enum {
            return Some((ww, true));
        }
        Some((progman, false))
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

            // Retirer WS_EX_TOPMOST : sinon la fenêtre reste au-dessus des icônes.
            // (le spike ne renvoie jamais 0x052C : ça spawn des WorkerW fantômes sur Win11)
            unsafe {
                let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let new_ex = ex & !WS_EX_TOPMOST;
                let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);
                log(&format!("exstyle 0x{ex:08x} -> 0x{new_ex:08x} (TOPMOST retiré)"));
            }

            match find_parent_target() {
                Some((target, is_workerw)) => unsafe {
                    match SetParent(hwnd, Some(target)) {
                        Ok(prev) => {
                            let kind = if is_workerw { "WorkerW" } else { "Progman" };
                            log(&format!(
                                "SetParent -> {target:?} ({kind}) OK (prev={prev:?})"
                            ));
                            if !is_workerw {
                                // Fond du z-order de Progman -> sous les icônes
                                let r = SetWindowPos(
                                    hwnd,
                                    Some(HWND_BOTTOM),
                                    0,
                                    0,
                                    0,
                                    0,
                                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                );
                                log(&format!("SetWindowPos(HWND_BOTTOM) -> {r:?}"));
                            }
                        }
                        Err(e) => log(&format!("SetParent failed: {e:?}")),
                    }
                },
                None => {
                    log("Progman NOT FOUND — fallback always-on-bottom");
                    let _ = win.set_always_on_bottom(true);
                }
            }

            // Click-through : la région de fenêtre = empreinte opaque du cercle tourné.
            // Les pixels hors région ne font pas partie de la fenêtre -> clics au bureau.
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
