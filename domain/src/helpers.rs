pub fn format_size(bytes: i64) -> String {
    match bytes {
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.1} KB", b / 1024),
        b => format!("{:.1} MB", b / (1024 * 1024)),
    }
}
