//! Type-safe wrappers for domain-specific strings and values.

use std::fmt;
use std::path::PathBuf;

/// A YouTube video ID (e.g., "dQw4w9WgXcQ")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoId(String);

impl VideoId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for VideoId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for VideoId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A path to an audio file
#[derive(Debug, Clone)]
pub struct AudioPath(PathBuf);

impl AudioPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

/// A path to a video file
#[derive(Debug, Clone)]
pub struct VideoPath(PathBuf);

impl VideoPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

/// Duration in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seconds(u32);

impl Seconds {
    pub fn new(secs: u32) -> Self {
        Self(secs)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Seconds {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
