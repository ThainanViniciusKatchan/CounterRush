use std::ffi::c_void;
use crate::WindowsApiError;

type HANDLE = *mut c_void;
type HWND = *mut c_void;
type BOOL = i32;

const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;

#[link(name = "user32")]
extern "system" {
    fn OpenClipboard(hwnd: HWND) -> BOOL;
    fn CloseClipboard() -> BOOL;
    fn EmptyClipboard() -> BOOL;
    fn SetClipboardData(u_format: u32, h_mem: HANDLE) -> HANDLE;
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(u_flags: u32, dw_bytes: usize) -> HANDLE;
    fn GlobalLock(h_mem: HANDLE) -> *mut c_void;
    fn GlobalUnlock(h_mem: HANDLE) -> BOOL;
    fn GlobalFree(h_mem: HANDLE) -> HANDLE;
    fn GetLastError() -> u32;
}

/// Copia uma string formatada para a área de transferência do Windows utilizando a API nativa Win32.
pub fn set_clipboard_text(text: &str) -> Result<(), WindowsApiError> {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes_len = wide.len() * std::mem::size_of::<u16>();

    unsafe {
        let h_mem = GlobalAlloc(GMEM_MOVEABLE, bytes_len);
        if h_mem.is_null() {
            return Err(WindowsApiError::ExecutionFailed(format!(
                "Falha ao alocar memória global para o clipboard: código {}",
                GetLastError()
            )));
        }

        let ptr = GlobalLock(h_mem) as *mut u16;
        if ptr.is_null() {
            GlobalFree(h_mem);
            return Err(WindowsApiError::ExecutionFailed(format!(
                "Falha ao bloquear memória global do clipboard: código {}",
                GetLastError()
            )));
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(h_mem);

        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        if !opened {
            GlobalFree(h_mem);
            return Err(WindowsApiError::ExecutionFailed(format!(
                "Falha ao abrir o clipboard do Windows: código {}",
                GetLastError()
            )));
        }

        EmptyClipboard();

        let handle = SetClipboardData(CF_UNICODETEXT, h_mem);
        CloseClipboard();

        if handle.is_null() {
            GlobalFree(h_mem);
            return Err(WindowsApiError::ExecutionFailed(format!(
                "Falha ao definir dados no clipboard do Windows: código {}",
                GetLastError()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_clipboard_text_roundtrip() {
        let test_text = "CounterRush Unit Test Clipboard String";
        let res = set_clipboard_text(test_text);
        assert!(res.is_ok(), "Falha ao definir texto no clipboard: {:?}", res.err());
    }
}
