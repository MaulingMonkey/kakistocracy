pub(crate) mod assoc;
mod hooks;

unsafe fn on_hwnd_creating(hwnd: winapi::shared::windef::HWND) {
    assoc::on_hwnd_creating(hwnd);
}

unsafe fn on_hwnd_destroyed(hwnd: winapi::shared::windef::HWND) {
    assoc::on_hwnd_destroyed(hwnd);
}
