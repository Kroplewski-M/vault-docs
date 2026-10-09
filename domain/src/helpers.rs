pub fn format_size(bytes: f64) -> String {
    match bytes {
        b if b < 1024.0 => format!("{b} B"),
        b if b < 1024.0 * 1024.0 => format!("{:.1} KB", b / 1024.0),
        b => format!("{:.1} MB", b / (1024.0 * 1024.0)),
    }
}
