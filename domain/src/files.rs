use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    helpers::format_size,
    models::{DataKey, EncVersion},
};

/// Default max file size: 15 MiB
pub const DEFAULT_MAX_FILE_SIZE: usize = 15 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileFormat {
    Pdf,
    Md,
    Txt,
}

/// What the file's bytes actually contain. Several formats can share a kind
/// (e.g. md and txt are both plain text), so the extension picks the exact format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentKind {
    Pdf,
    Text,
}
impl fmt::Display for ContentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pdf => write!(f, "PDF"),
            Self::Text => write!(f, "plain text"),
        }
    }
}
impl ContentKind {
    /// Detect content kind from magic bytes, Returns None if unrecognised
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(b"%PDF-") {
            return Some(Self::Pdf);
        }
        if is_text(bytes) {
            return Some(Self::Text);
        }
        None
    }
}

impl FileFormat {
    pub const ALL: &'static [Self] = &[Self::Pdf, Self::Md, Self::Txt];

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Md => "md",
            Self::Txt => "txt",
        }
    }
    pub fn from_extension(ext: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|f| f.extension().eq_ignore_ascii_case(ext))
    }
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Md => "text/markdown",
            Self::Txt => "text/plain",
        }
    }
    pub fn content_kind(&self) -> ContentKind {
        match self {
            Self::Pdf => ContentKind::Pdf,
            Self::Md | Self::Txt => ContentKind::Text,
        }
    }
}
impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.extension())
    }
}
fn is_text(bytes: &[u8]) -> bool {
    !bytes.is_empty() && !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadError {
    TooLarge {
        size: usize,
        max: usize,
    },
    UnsupportedExtension,
    UnrecognisedContent,
    ExtensionMismatch {
        expected: FileFormat,
        detected: ContentKind,
    },
}
impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { size, max } => write!(
                f,
                "File is too large ({}, max {})",
                format_size(*size as i64),
                format_size(*max as i64)
            ),
            Self::UnsupportedExtension => write!(f, "Unsupported file type"),
            Self::UnrecognisedContent => write!(f, "File contents could not be recognised"),
            Self::ExtensionMismatch { expected, detected } => {
                write!(f, "File has a {expected} extension but contains {detected}")
            }
        }
    }
}

impl std::error::Error for UploadError {}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Valid(FileFormat),
    Invalid(Vec<UploadError>),
}

/// A file the user dropped. It may or may not be valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub id: Uuid,
    size: usize,
    name: String,
    status: FileStatus,
}

impl UploadedFile {
    pub fn new(name: impl Into<String>, bytes: &[u8]) -> Self {
        let name = name.into();
        let size = bytes.len();
        let status = Self::validate(&name, size, Some(bytes));
        Self {
            id: Uuid::new_v4(),
            size,
            name,
            status,
        }
    }

    /// A file rejected by size alone, so its contents are never read into memory.
    pub fn oversized(name: impl Into<String>, size: usize) -> Self {
        let name = name.into();
        let status = Self::validate(&name, size, None);
        Self {
            id: Uuid::new_v4(),
            size,
            name,
            status,
        }
    }

    pub fn exceeds_max_size(size: usize) -> bool {
        size > DEFAULT_MAX_FILE_SIZE
    }

    /// `bytes` is None when the contents were not loaded (oversized files).
    fn validate(name: &str, size: usize, bytes: Option<&[u8]>) -> FileStatus {
        let mut errors = Vec::new();

        let too_large = Self::exceeds_max_size(size);
        if too_large {
            errors.push(UploadError::TooLarge {
                size,
                max: DEFAULT_MAX_FILE_SIZE,
            });
        }

        let expected = std::path::Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(FileFormat::from_extension);

        if expected.is_none() {
            errors.push(UploadError::UnsupportedExtension);
        }

        // Skip scanning content of oversized files
        let detected = match bytes {
            Some(bytes) if !too_large => ContentKind::from_bytes(bytes),
            _ => None,
        };
        if !too_large && detected.is_none() {
            errors.push(UploadError::UnrecognisedContent);
        }

        if let (Some(expected), Some(detected)) = (expected, detected)
            && expected.content_kind() != detected
        {
            errors.push(UploadError::ExtensionMismatch { expected, detected });
        }

        match (errors.is_empty(), expected) {
            (true, Some(format)) => FileStatus::Valid(format),
            _ => FileStatus::Invalid(errors),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn size(&self) -> i64 {
        self.size as i64
    }
    pub fn status(&self) -> &FileStatus {
        &self.status
    }
    pub fn is_valid(&self) -> bool {
        matches!(self.status, FileStatus::Valid(_))
    }
    pub fn errors(&self) -> &[UploadError] {
        match &self.status {
            FileStatus::Valid(_) => &[],
            FileStatus::Invalid(e) => e,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct UserFile {
    pub id: Uuid,
    pub full_name: String,
    pub size: i64,
}
#[derive(Debug)]
pub struct FileMetaData {
    pub id: Uuid,
    pub original_name: String,
    pub ext: String,
    pub mime_type: String,
    pub size: i64,
    pub user_id: Uuid,
    pub enc_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub enc_version: EncVersion,
}
#[derive(Debug)]
pub struct EncryptedFileMetaData {
    pub bytes: Vec<u8>,
    pub nonce: Vec<u8>,
    pub enc_version: EncVersion,
    pub encryption_key: DataKey,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileUploadResult {
    pub id: String,
    pub name: String,
    pub error: Option<String>,
    pub file: Option<UserFile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const PDF_BYTES: &[u8] = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n";
    const TEXT_BYTES: &[u8] = b"# hello\nsome notes";

    // --- ContentKind::from_bytes ---

    #[test]
    fn detects_pdf_from_magic_bytes() {
        assert_eq!(ContentKind::from_bytes(PDF_BYTES), Some(ContentKind::Pdf));
    }

    #[test]
    fn detects_utf8_text() {
        assert_eq!(ContentKind::from_bytes(TEXT_BYTES), Some(ContentKind::Text));
        assert_eq!(
            ContentKind::from_bytes("zażółć gęślą jaźń".as_bytes()),
            Some(ContentKind::Text)
        );
    }

    #[test]
    fn empty_bytes_are_unrecognised() {
        assert_eq!(ContentKind::from_bytes(b""), None);
    }

    #[test]
    fn bytes_with_nul_are_unrecognised() {
        assert_eq!(ContentKind::from_bytes(b"abc\0def"), None);
    }

    #[test]
    fn invalid_utf8_is_unrecognised() {
        assert_eq!(ContentKind::from_bytes(&[0xff, 0xfe, 0xfd]), None);
    }

    #[test]
    fn partial_pdf_magic_is_not_pdf() {
        // "%PDF" without the dash is valid text, not a PDF
        assert_eq!(ContentKind::from_bytes(b"%PDF"), Some(ContentKind::Text));
    }

    // --- FileFormat ---

    #[test]
    fn from_extension_matches_known_formats() {
        assert_eq!(FileFormat::from_extension("pdf"), Some(FileFormat::Pdf));
        assert_eq!(FileFormat::from_extension("md"), Some(FileFormat::Md));
        assert_eq!(FileFormat::from_extension("txt"), Some(FileFormat::Txt));
    }

    #[test]
    fn from_extension_is_case_insensitive() {
        assert_eq!(FileFormat::from_extension("PDF"), Some(FileFormat::Pdf));
        assert_eq!(FileFormat::from_extension("Md"), Some(FileFormat::Md));
    }

    #[test]
    fn from_extension_rejects_unknown() {
        assert_eq!(FileFormat::from_extension("exe"), None);
        assert_eq!(FileFormat::from_extension(""), None);
        assert_eq!(FileFormat::from_extension(".pdf"), None);
    }

    #[test]
    fn extension_round_trips_for_all_formats() {
        for format in FileFormat::ALL {
            assert_eq!(
                FileFormat::from_extension(format.extension()),
                Some(*format)
            );
        }
    }

    #[test]
    fn mime_types_and_content_kinds() {
        assert_eq!(FileFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(FileFormat::Md.mime_type(), "text/markdown");
        assert_eq!(FileFormat::Txt.mime_type(), "text/plain");

        assert_eq!(FileFormat::Pdf.content_kind(), ContentKind::Pdf);
        assert_eq!(FileFormat::Md.content_kind(), ContentKind::Text);
        assert_eq!(FileFormat::Txt.content_kind(), ContentKind::Text);
    }

    #[test]
    fn file_format_display_has_leading_dot() {
        assert_eq!(FileFormat::Pdf.to_string(), ".pdf");
    }

    // --- UploadedFile validation ---

    #[test]
    fn valid_files_for_each_format() {
        let pdf = UploadedFile::new("doc.pdf", PDF_BYTES);
        assert_eq!(pdf.status(), &FileStatus::Valid(FileFormat::Pdf));
        assert!(pdf.is_valid());
        assert!(pdf.errors().is_empty());

        let md = UploadedFile::new("notes.md", TEXT_BYTES);
        assert_eq!(md.status(), &FileStatus::Valid(FileFormat::Md));

        let txt = UploadedFile::new("notes.TXT", TEXT_BYTES);
        assert_eq!(txt.status(), &FileStatus::Valid(FileFormat::Txt));
    }

    #[test]
    fn records_name_and_size() {
        let file = UploadedFile::new("notes.md", TEXT_BYTES);
        assert_eq!(file.name(), "notes.md");
        assert_eq!(file.size(), TEXT_BYTES.len() as i64);
    }

    #[test]
    fn each_file_gets_a_unique_id() {
        let a = UploadedFile::new("a.md", TEXT_BYTES);
        let b = UploadedFile::new("a.md", TEXT_BYTES);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn pdf_extension_with_text_content_is_mismatch() {
        let file = UploadedFile::new("doc.pdf", TEXT_BYTES);
        assert!(!file.is_valid());
        assert_eq!(
            file.errors(),
            &[UploadError::ExtensionMismatch {
                expected: FileFormat::Pdf,
                detected: ContentKind::Text,
            }]
        );
    }

    #[test]
    fn text_extension_with_pdf_content_is_mismatch() {
        let file = UploadedFile::new("notes.txt", PDF_BYTES);
        assert_eq!(
            file.errors(),
            &[UploadError::ExtensionMismatch {
                expected: FileFormat::Txt,
                detected: ContentKind::Pdf,
            }]
        );
    }

    #[test]
    fn unsupported_extension() {
        let file = UploadedFile::new("virus.exe", TEXT_BYTES);
        assert_eq!(file.errors(), &[UploadError::UnsupportedExtension]);
    }

    #[test]
    fn missing_extension_is_unsupported() {
        let file = UploadedFile::new("README", TEXT_BYTES);
        assert_eq!(file.errors(), &[UploadError::UnsupportedExtension]);
    }

    #[test]
    fn dotfile_without_extension_is_unsupported() {
        // Path::extension treats ".md" as a file stem, not an extension
        let file = UploadedFile::new(".md", TEXT_BYTES);
        assert_eq!(file.errors(), &[UploadError::UnsupportedExtension]);
    }

    #[test]
    fn binary_content_is_unrecognised() {
        let file = UploadedFile::new("notes.txt", &[0x00, 0x01, 0x02]);
        assert_eq!(file.errors(), &[UploadError::UnrecognisedContent]);
    }

    #[test]
    fn empty_file_is_unrecognised() {
        let file = UploadedFile::new("empty.txt", b"");
        assert_eq!(file.errors(), &[UploadError::UnrecognisedContent]);
    }

    #[test]
    fn unsupported_extension_and_unrecognised_content_both_reported() {
        let file = UploadedFile::new("blob.bin", &[0xff, 0x00]);
        assert_eq!(
            file.errors(),
            &[
                UploadError::UnsupportedExtension,
                UploadError::UnrecognisedContent
            ]
        );
    }

    #[test]
    fn file_at_exact_max_size_is_allowed() {
        assert!(!UploadedFile::exceeds_max_size(DEFAULT_MAX_FILE_SIZE));
        assert!(UploadedFile::exceeds_max_size(DEFAULT_MAX_FILE_SIZE + 1));

        let bytes = vec![b'a'; DEFAULT_MAX_FILE_SIZE];
        let file = UploadedFile::new("big.txt", &bytes);
        assert!(file.is_valid());
    }

    #[test]
    fn oversized_file_only_reports_too_large() {
        let size = DEFAULT_MAX_FILE_SIZE + 1;
        let file = UploadedFile::oversized("big.pdf", size);
        assert_eq!(file.size(), size as i64);
        assert_eq!(
            file.errors(),
            &[UploadError::TooLarge {
                size,
                max: DEFAULT_MAX_FILE_SIZE,
            }]
        );
    }

    #[test]
    fn oversized_content_is_not_scanned() {
        // binary bytes would normally be UnrecognisedContent, but oversized files skip the scan
        let bytes = vec![0u8; DEFAULT_MAX_FILE_SIZE + 1];
        let file = UploadedFile::new("big.txt", &bytes);
        assert_eq!(
            file.errors(),
            &[UploadError::TooLarge {
                size: bytes.len(),
                max: DEFAULT_MAX_FILE_SIZE,
            }]
        );
    }

    #[test]
    fn oversized_with_bad_extension_reports_both() {
        let size = DEFAULT_MAX_FILE_SIZE + 1;
        let file = UploadedFile::oversized("big.exe", size);
        assert_eq!(
            file.errors(),
            &[
                UploadError::TooLarge {
                    size,
                    max: DEFAULT_MAX_FILE_SIZE,
                },
                UploadError::UnsupportedExtension,
            ]
        );
    }

    // --- Display messages shown to users ---

    #[test]
    fn upload_error_messages() {
        assert_eq!(
            UploadError::UnsupportedExtension.to_string(),
            "Unsupported file type"
        );
        assert_eq!(
            UploadError::UnrecognisedContent.to_string(),
            "File contents could not be recognised"
        );
        assert_eq!(
            UploadError::ExtensionMismatch {
                expected: FileFormat::Pdf,
                detected: ContentKind::Text,
            }
            .to_string(),
            "File has a .pdf extension but contains plain text"
        );
        assert_eq!(
            UploadError::TooLarge {
                size: 20 * 1024 * 1024,
                max: DEFAULT_MAX_FILE_SIZE,
            }
            .to_string(),
            "File is too large (20.0 MB, max 15.0 MB)"
        );
    }
}
