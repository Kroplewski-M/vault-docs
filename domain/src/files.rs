use uuid::Uuid;

/// Default max file size: 50 MiB
pub const DEFAULT_MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Pdf,
    Md,
}
impl FileFormat {
    pub const ALL: &'static [Self] = &[Self::Pdf, Self::Md];

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Md => "md",
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
        }
    }
    /// Detext format from magic bytes, Resturns None if unrecognised
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(b"%PDF-") {
            return Some(Self::Pdf);
        }
        if is_text(bytes) {
            return Some(Self::Md);
        }
        None
    }
}
fn is_text(bytes: &[u8]) -> bool {
    !bytes.is_empty() && !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadError {
    TooLarge {
        size: usize,
        max: usize,
    },
    UnsupportedExtension,
    UnrecognisedContent,
    ExtensionMismatch {
        expected: FileFormat,
        detected: FileFormat,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Valid(FileFormat),
    Invalid(Vec<UploadError>),
}

/// A file the user dropped. It may or may not be valid.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub id: Uuid,
    bytes: Vec<u8>,
    name: String,
    status: FileStatus,
}

impl UploadedFile {
    pub fn new(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        let name = name.into();
        let status = Self::validate(&name, &bytes);
        Self {
            id: Uuid::new_v4(),
            bytes,
            name,
            status,
        }
    }

    fn validate(name: &str, bytes: &[u8]) -> FileStatus {
        let mut errors = Vec::new();

        let too_large = bytes.len() > DEFAULT_MAX_FILE_SIZE;
        if too_large {
            errors.push(UploadError::TooLarge {
                size: bytes.len(),
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
        let detected = if too_large {
            None
        } else {
            FileFormat::from_bytes(bytes)
        };
        if !too_large && detected.is_none() {
            errors.push(UploadError::UnrecognisedContent);
        }

        if let (Some(expected), Some(detected)) = (expected, detected)
            && expected != detected
        {
            errors.push(UploadError::ExtensionMismatch { expected, detected });
        }

        match (errors.is_empty(), detected) {
            (true, Some(format)) => FileStatus::Valid(format),
            _ => FileStatus::Invalid(errors),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn size(&self) -> i64 {
        self.bytes.len() as i64
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
