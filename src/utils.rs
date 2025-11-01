pub fn from_wide_string(wide: *const u16) -> String {
    if wide.is_null() {
        return String::new();
    }

    unsafe {
        let len = (0..).take_while(|&i| *wide.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(wide, len);
        String::from_utf16_lossy(slice)
    }
}
