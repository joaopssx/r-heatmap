#[cfg(windows)]
pub fn use_utf8() {
    const UTF8: u32 = 65001;

    if unsafe { windows_sys::Win32::System::Console::SetConsoleOutputCP(UTF8) } == 0 {
        log::warn!("Could not switch the console to UTF-8, box drawing may be garbled");
    }
}

#[cfg(not(windows))]
pub fn use_utf8() {}
