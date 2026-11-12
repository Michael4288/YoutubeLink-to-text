#![windows_subsystem = "windows"]

mod transcript;

use std::{
    ffi::c_void,
    mem,
    ptr,
    thread,
};

type Hwnd = *mut c_void;
type Hinstance = *mut c_void;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_BORDER: u32 = 0x00800000;
const WS_VSCROLL: u32 = 0x00200000;
const ES_LEFT: u32 = 0;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;
const BS_PUSHBUTTON: u32 = 0;
const WM_CREATE: u32 = 0x0001;
const WM_SIZE: u32 = 0x0005;
const WM_COMMAND: u32 = 0x0111;
const EN_CHANGE: usize = 0x0300;
const WM_DESTROY: u32 = 0x0002;
const WM_APP_RESULT: u32 = 0x8001;
const WM_APP_TRACKS: u32 = 0x8002;
const SW_SHOW: i32 = 5;
const EM_SETSEL: u32 = 0x00B1;
const EM_REPLACESEL: u32 = 0x00C2;
const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;
const IDC_URL: i32 = 101;
const IDC_FETCH: i32 = 102;
const IDC_TEXT: i32 = 103;
const IDC_COPY: i32 = 104;
const IDC_STATUS: i32 = 105;
const IDC_DISCOVER: i32 = 107;
const IDC_TRACKS: i32 = 108;
const LBS_NOTIFY: u32 = 0x0001;
const LBS_EXTENDEDSEL: u32 = 0x0800;
const LB_ADDSTRING: u32 = 0x0180;
const LB_RESETCONTENT: u32 = 0x0184;
const LB_SETSEL: u32 = 0x0185;
const LB_GETSELCOUNT: u32 = 0x0190;
const LB_GETSELITEMS: u32 = 0x0191;
const LB_GETTEXTLEN: u32 = 0x018A;
const LB_GETTEXT: u32 = 0x0189;

#[repr(C)]
struct Point { x: i32, y: i32 }

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    time: u32,
    pt: Point,
}

#[repr(C)]
struct WndClassW {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Hinstance,
    icon: Hwnd,
    cursor: Hwnd,
    background: Hwnd,
    menu_name: *const u16,
    class_name: *const u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(ex_style: u32, class: *const u16, title: *const u16, style: u32,
        x: i32, y: i32, width: i32, height: i32, parent: Hwnd, menu: Hwnd,
        instance: Hinstance, param: *mut c_void) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> Lresult;
    fn PostQuitMessage(code: i32);
    fn PostMessageW(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> i32;
    fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max: i32) -> i32;
    fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    fn GetDlgItem(hwnd: Hwnd, id: i32) -> Hwnd;
    fn EnableWindow(hwnd: Hwnd, enable: i32) -> i32;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
    fn UpdateWindow(hwnd: Hwnd) -> i32;
    fn MoveWindow(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: i32) -> i32;
    fn OpenClipboard(hwnd: Hwnd) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, memory: Hwnd) -> Hwnd;
    fn CloseClipboard() -> i32;
    fn SendMessageW(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Hinstance;
    fn GlobalAlloc(flags: u32, bytes: usize) -> Hwnd;
    fn GlobalLock(memory: Hwnd) -> *mut c_void;
    fn GlobalUnlock(memory: Hwnd) -> i32;
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn set_text(hwnd: Hwnd, value: &str) {
    let text = wide(value);
    unsafe { SetWindowTextW(hwnd, text.as_ptr()); }
}

fn create_control(parent: Hwnd, class: &str, label: &str, id: i32, style: u32,
    x: i32, y: i32, width: i32, height: i32) -> Hwnd {
    let class = wide(class);
    let label = wide(label);
    unsafe {
        CreateWindowExW(0, class.as_ptr(), label.as_ptr(), WS_CHILD | WS_VISIBLE | style,
            x, y, width, height, parent, id as Hwnd, GetModuleHandleW(ptr::null()), ptr::null_mut())
    }
}

unsafe extern "system" fn window_proc(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult {
    match message {
        WM_CREATE => {
            create_control(hwnd, "STATIC", "Pega un enlace de YouTube", 0, 0, 24, 22, 400, 24);
            create_control(hwnd, "EDIT", "", IDC_URL, WS_BORDER | ES_LEFT, 24, 52, 650, 32);
            create_control(hwnd, "BUTTON", "Buscar idiomas", IDC_DISCOVER, BS_PUSHBUTTON, 690, 52, 140, 32);
            create_control(hwnd, "STATIC", "Idiomas disponibles (elige una o más pistas)", 0, 0, 24, 98, 500, 24);
            create_control(hwnd, "LISTBOX", "", IDC_TRACKS, WS_BORDER | WS_VSCROLL | LBS_NOTIFY | LBS_EXTENDEDSEL, 24, 124, 806, 100);
            create_control(hwnd, "BUTTON", "Obtener texto", IDC_FETCH, BS_PUSHBUTTON, 690, 238, 140, 32);
            create_control(hwnd, "STATIC", "Transcripción", 0, 0, 24, 284, 300, 24);
            create_control(hwnd, "EDIT", "", IDC_TEXT, WS_BORDER | WS_VSCROLL | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
                24, 312, 806, 220);
            create_control(hwnd, "BUTTON", "Copiar texto", IDC_COPY, BS_PUSHBUTTON, 690, 552, 140, 34);
            create_control(hwnd, "STATIC", "Listo. Pega un enlace para comenzar.", IDC_STATUS, 0, 24, 557, 500, 28);
            0
        }
        WM_SIZE => {
            let width = (lparam as u32 & 0xffff) as i32;
            let height = ((lparam as u32 >> 16) & 0xffff) as i32;
            let text_width = (width - 48).max(300);
            let output_height = (height - 390).max(120);
            MoveWindow(GetDlgItem(hwnd, IDC_URL), 24, 52, (width - 220).max(200), 32, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_DISCOVER), width - 164, 52, 140, 32, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_TRACKS), 24, 124, text_width, 100, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_FETCH), width - 164, 238, 140, 32, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_TEXT), 24, 312, text_width, output_height, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_COPY), width - 164, height - 62, 140, 34, 1);
            MoveWindow(GetDlgItem(hwnd, IDC_STATUS), 24, height - 60, 500, 28, 1);
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xffff) as i32;
            if id == IDC_URL && (wparam >> 16) == EN_CHANGE {
                SendMessageW(GetDlgItem(hwnd, IDC_TRACKS), LB_RESETCONTENT, 0, 0);
                set_text(GetDlgItem(hwnd, IDC_TEXT), "");
                set_text(GetDlgItem(hwnd, IDC_STATUS), "El enlace cambió. Busca de nuevo los idiomas disponibles.");
            } else if id == IDC_DISCOVER {
                let input = GetDlgItem(hwnd, IDC_URL);
                let length = GetWindowTextLengthW(input).max(0) as usize;
                let mut buffer = vec![0u16; length + 1];
                GetWindowTextW(input, buffer.as_mut_ptr(), buffer.len() as i32);
                let url = String::from_utf16_lossy(&buffer[..length]).trim().to_owned();
                if url.is_empty() {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Pega primero un enlace de YouTube.");
                    return 0;
                }
                EnableWindow(GetDlgItem(hwnd, IDC_DISCOVER), 0);
                EnableWindow(GetDlgItem(hwnd, IDC_FETCH), 0);
                set_text(GetDlgItem(hwnd, IDC_STATUS), "Buscando idiomas disponibles…");
                let window = hwnd as isize;
                let (yt_dlp, deno) = tool_paths();
                thread::spawn(move || {
                    let result = transcript::discover(&url, yt_dlp.as_deref(), deno.as_deref());
                    let payload = Box::into_raw(Box::new(result));
                    unsafe { PostMessageW(window as Hwnd, WM_APP_TRACKS, 0, payload as Lparam); }
                });
            } else if id == IDC_FETCH {
                let input = GetDlgItem(hwnd, IDC_URL);
                let length = GetWindowTextLengthW(input).max(0) as usize;
                let mut buffer = vec![0u16; length + 1];
                GetWindowTextW(input, buffer.as_mut_ptr(), buffer.len() as i32);
                let url = String::from_utf16_lossy(&buffer[..length]).trim().to_owned();
                if url.is_empty() {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Pega primero un enlace de YouTube.");
                    return 0;
                }
                let list = GetDlgItem(hwnd, IDC_TRACKS);
                let count = SendMessageW(list, LB_GETSELCOUNT, 0, 0);
                if count <= 0 {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Primero busca los idiomas y selecciona una o más pistas.");
                    return 0;
                }
                let mut indexes = vec![0i32; count as usize];
                let selected = SendMessageW(list, LB_GETSELITEMS, count as usize, indexes.as_mut_ptr() as Lparam);
                indexes.truncate(selected.max(0) as usize);
                let languages = indexes.into_iter().filter_map(|index| {
                    let length = SendMessageW(list, LB_GETTEXTLEN, index as usize, 0);
                    if length <= 0 { return None; }
                    let mut row = vec![0u16; length as usize + 1];
                    SendMessageW(list, LB_GETTEXT, index as usize, row.as_mut_ptr() as Lparam);
                    let label = String::from_utf16_lossy(&row[..length as usize]);
                    label.split(" — ").next().map(str::to_owned)
                }).collect::<Vec<_>>().join(",");
                if languages.is_empty() {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Selecciona al menos una pista.");
                    return 0;
                }
                EnableWindow(GetDlgItem(hwnd, IDC_FETCH), 0);
                EnableWindow(GetDlgItem(hwnd, IDC_DISCOVER), 0);
                set_text(GetDlgItem(hwnd, IDC_STATUS), "Obteniendo transcripción…");
                let window = hwnd as isize;
                let (yt_dlp, deno) = tool_paths();
                thread::spawn(move || {
                    let executable = yt_dlp.filter(|path| path.exists());
                    let runtime = deno.filter(|path| path.exists());
                    let result = transcript::fetch(&url, &languages, executable.as_deref(), runtime.as_deref());
                    let payload = Box::into_raw(Box::new(result));
                    unsafe { PostMessageW(window as Hwnd, WM_APP_RESULT, 0, payload as Lparam); }
                });
            } else if id == IDC_COPY {
                let edit = GetDlgItem(hwnd, IDC_TEXT);
                let length = GetWindowTextLengthW(edit).max(0) as usize;
                if length == 0 {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Todavía no hay texto para copiar.");
                    return 0;
                }
                let mut text = vec![0u16; length + 1];
                GetWindowTextW(edit, text.as_mut_ptr(), text.len() as i32);
                if OpenClipboard(hwnd) != 0 {
                    EmptyClipboard();
                    let memory = GlobalAlloc(GMEM_MOVEABLE, text.len() * mem::size_of::<u16>());
                    if !memory.is_null() {
                        let target = GlobalLock(memory) as *mut u16;
                        if !target.is_null() {
                            ptr::copy_nonoverlapping(text.as_ptr(), target, text.len());
                            GlobalUnlock(memory);
                            SetClipboardData(CF_UNICODETEXT, memory);
                            set_text(GetDlgItem(hwnd, IDC_STATUS), "Texto copiado al portapapeles.");
                        }
                    }
                    CloseClipboard();
                }
            }
            0
        }
        WM_APP_RESULT => {
            EnableWindow(GetDlgItem(hwnd, IDC_FETCH), 1);
            EnableWindow(GetDlgItem(hwnd, IDC_DISCOVER), 1);
            let result = Box::from_raw(lparam as *mut Result<String, String>);
            match *result {
                Ok(text) => {
                    SendMessageW(GetDlgItem(hwnd, IDC_TEXT), EM_SETSEL, 0, -1);
                    let text = wide(&text);
                    SendMessageW(GetDlgItem(hwnd, IDC_TEXT), EM_REPLACESEL, 1, text.as_ptr() as Lparam);
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Transcripción lista. Ya puedes copiarla.");
                }
                Err(error) => {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "No se pudo obtener la transcripción. Revisa el detalle abajo.");
                    set_text(GetDlgItem(hwnd, IDC_TEXT), &error);
                }
            }
            0
        }
        WM_APP_TRACKS => {
            EnableWindow(GetDlgItem(hwnd, IDC_DISCOVER), 1);
            EnableWindow(GetDlgItem(hwnd, IDC_FETCH), 1);
            let result = Box::from_raw(lparam as *mut Result<Vec<transcript::SubtitleTrack>, String>);
            let list = GetDlgItem(hwnd, IDC_TRACKS);
            SendMessageW(list, LB_RESETCONTENT, 0, 0);
            match *result {
                Ok(tracks) => {
                    let default_track = tracks.iter().position(|track| track.code == "es")
                        .or_else(|| tracks.iter().position(|track| track.code == "en"))
                        .unwrap_or(0);
                    for (index, track) in tracks.into_iter().enumerate() {
                        let label = wide(&track.display_name());
                        SendMessageW(list, LB_ADDSTRING, 0, label.as_ptr() as Lparam);
                        if index == default_track {
                            SendMessageW(list, LB_SETSEL, 1, index as Lparam);
                        }
                    }
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "Idiomas listos. Se seleccionó una pista; mantén Ctrl para elegir más.");
                }
                Err(error) => {
                    set_text(GetDlgItem(hwnd, IDC_STATUS), "No se pudieron consultar las pistas. Revisa el detalle abajo.");
                    set_text(GetDlgItem(hwnd, IDC_TEXT), &error);
                }
            }
            0
        }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn tool_paths() -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
    let app_dir = std::env::current_exe().ok().and_then(|path| path.parent().map(|p| p.to_path_buf()));
    let yt_dlp = app_dir.as_ref().map(|dir| dir.join("yt-dlp.exe"));
    let deno = app_dir.as_ref().map(|dir| dir.join("deno.exe"));
    (yt_dlp.filter(|path| path.exists()), deno.filter(|path| path.exists()))
}

fn main() {
    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        let class_name = wide("TextoYoutubeWindow");
        let title = wide("Texto de YouTube");
        let class = WndClassW {
            style: 0,
            wnd_proc: Some(window_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: ptr::null_mut(),
            cursor: ptr::null_mut(),
            background: 6usize as Hwnd,
            menu_name: ptr::null(),
            class_name: class_name.as_ptr(),
        };
        RegisterClassW(&class);
        let hwnd = CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW,
            100, 100, 880, 660, ptr::null_mut(), ptr::null_mut(), instance, ptr::null_mut());
        if hwnd.is_null() { return; }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        let mut msg: Msg = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
