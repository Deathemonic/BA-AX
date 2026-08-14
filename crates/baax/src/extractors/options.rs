#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMode {
    MediaResources,
    Tables
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractOptions<'a> {
    pub mode: ExtractionMode,
    pub lowercase: bool,
    pub key: Option<&'a str>,
    pub license: Option<&'a str>
}

impl<'a> ExtractOptions<'a> {
    pub const fn new(mode: ExtractionMode) -> Self {
        Self { mode, lowercase: false, key: None, license: None }
    }

    pub const fn with_lowercase(mut self, lowercase: bool) -> Self {
        self.lowercase = lowercase;
        self
    }

    pub const fn with_key(mut self, key: Option<&'a str>) -> Self {
        self.key = key;
        self
    }

    pub const fn with_license(mut self, license: Option<&'a str>) -> Self {
        self.license = license;
        self
    }
}