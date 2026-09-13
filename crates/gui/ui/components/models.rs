use std::path::PathBuf;
use anime_launcher_sdk::components::{dxvk, wine};

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Idle,
    Downloading {
        fraction: f32,
        caption: String,
    },
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ComponentVersion {
    pub name: String,
    pub title: String,
    pub uri: String,
    pub format: Option<String>,
    pub recommended: bool,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone)]
pub struct ComponentGroup {
    pub title: String,
    pub is_expanded: bool,
    pub versions: Vec<ComponentVersion>,
}

#[derive(Debug, Clone)]
pub struct ComponentsListPattern {
    pub download_folder: PathBuf,
    pub groups: Vec<ComponentGroup>,
}

impl From<wine::Group> for ComponentGroup {
    fn from(group: wine::Group) -> Self {
        Self {
            title: group.title,
            is_expanded: true,
            versions: group.versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<wine::Version> for ComponentVersion {
    fn from(v: wine::Version) -> Self {
        Self {
            recommended: v.version_features().map(|f| f.recommended).unwrap_or(true),
            name: v.name,
            title: v.title,
            uri: v.uri,
            format: v.format,
            status: DownloadStatus::Idle,
        }
    }
}

impl From<dxvk::Group> for ComponentGroup {
    fn from(group: dxvk::Group) -> Self {
        Self {
            title: group.title,
            is_expanded: true,
            versions: group.versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<dxvk::Version> for ComponentVersion {
    fn from(v: dxvk::Version) -> Self {
        Self {
            recommended: v.version_features().map(|f| f.recommended).unwrap_or(true),
            name: v.name,
            title: v.title,
            uri: v.uri,
            format: v.format,
            status: DownloadStatus::Idle,
        }
    }
}
