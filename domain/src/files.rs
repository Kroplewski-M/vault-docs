use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::helpers::format_size;

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
    bytes: Arc<[u8]>,
    size: usize,
    name: String,
    status: FileStatus,
}

impl UploadedFile {
    pub fn new(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        let name = name.into();
        let size = bytes.len();
        let status = Self::validate(&name, size, Some(&bytes));
        Self {
            id: Uuid::new_v4(),
            bytes: bytes.into(),
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
            bytes: Arc::from([]),
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
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
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
