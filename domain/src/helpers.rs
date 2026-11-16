pub fn format_size(bytes: i64) -> String {
    match bytes {
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: i64 = 1024;
    const MB: i64 = 1024 * 1024;

    #[test]
    fn bytes_below_one_kb() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(KB - 1), "1023 B");
    }

    #[test]
    fn kilobytes() {
        assert_eq!(format_size(KB), "1.0 KB");
        assert_eq!(format_size(KB + KB / 2), "1.5 KB");
        assert_eq!(format_size(500 * KB), "500.0 KB");
    }

    #[test]
    fn just_below_one_mb_stays_in_kb() {
        assert_eq!(format_size(MB - 1), "1024.0 KB");
    }

    #[test]
    fn megabytes() {
        assert_eq!(format_size(MB), "1.0 MB");
        assert_eq!(format_size(15 * MB), "15.0 MB");
        assert_eq!(format_size(2 * MB + MB / 4), "2.2 MB");
    }

    #[test]
    fn very_large_sizes_stay_in_mb() {
        assert_eq!(format_size(1024 * MB), "1024.0 MB");
    }

    #[test]
    fn negative_sizes_fall_into_bytes() {
        assert_eq!(format_size(-1), "-1 B");
    }
}
