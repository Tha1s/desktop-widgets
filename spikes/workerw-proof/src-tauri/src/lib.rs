#![cfg(windows)]

use tauri::Manager;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, SendMessageW, SetParent, HTCLIENT,
    HTTRANSPARENT, WM_NCHITTEST,
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

fn inside_circle(local_x: f64, local_y: f64) -> bool {
    let dx = local_x - CIRCLE_CX;
    let dy = local_y - CIRCLE_CY;
    let (sin, cos) = ROT_RAD.sin_cos();
    let ix = dx * cos + dy * sin; // inverse rotation (-45°)
    let iy = -dx * sin + dy * cos;
    ix * ix + iy * iy <= CIRCLE_R * CIRCLE_R
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid_subclass: usize,
    _ref_data: usize,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
        let mut pt = POINT { x, y };
        let _ = ScreenToClient(hwnd, &mut pt);
        // le contenu HTML est en pixels logiques ; convertir le client physique
        let scale = GetDpiForWindow(hwnd) as f64 / 96.0;
        let (lx, ly) = (pt.x as f64 / scale, pt.y as f64 / scale);
        let hit: isize = if inside_circle(lx, ly) {
            HTCLIENT as isize
        } else {
            HTTRANSPARENT as isize
        };
        return LRESULT(hit);
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
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
        if let Ok(_defview) = defview {
            let found = lparam.0 as *mut Option<HWND>;
            (*found) = Some(hwnd);
            return BOOL(0);
        }
    }
    BOOL(1)
}

fn find_workerw() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), PCWSTR::null()).ok()?;
        // demander à explorer.exe de (re)créer la fenêtre WorkerW si absente
        let _ = SendMessageW(progman, WM_SPAWN_WORKERW, WPARAM(0), LPARAM(0));

        // recherche directe : la SHELLDLL_DefView est enfant de Progman ; le WorkerW
        // qui la suit est une fenêtre top-level
        if let Ok(defview) = FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), PCWSTR::null())
        {
            if let Ok(workerw) =
                FindWindowExW(None, Some(defview), w!("WorkerW"), PCWSTR::null())
            {
                return Some(workerw);
            }
        }
        // repli : énumération des fenêtres top-level pour un WorkerW contenant SHELLDLL_DefView
        let mut found: Option<HWND> = None;
        let _ = EnumWindows(
            Some(enum_workerw_cb),
            LPARAM(&mut found as *mut Option<HWND> as isize),
        );
        found
    }
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

            unsafe {
                let ok = SetWindowSubclass(hwnd, Some(subclass_proc), 1, 0);
                log(&format!("SetWindowSubclass ok={}", ok.0 != 0));
            }

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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running workerw-proof");
}
