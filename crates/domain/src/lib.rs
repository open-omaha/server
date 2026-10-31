//! Application metadata shared by storage and HTTP handlers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub omaha_app_id: String,
    pub display_name: String,
    pub version_scheme: VersionScheme,
    pub cup_policy: CupPolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VersionScheme {
    #[default]
    Chromium,
    Semver,
    OmahaLegacy,
}

impl VersionScheme {
    pub fn name(self) -> &'static str {
        match self {
            Self::Chromium => "CHROMIUM",
            Self::Semver => "SEMVER",
            Self::OmahaLegacy => "OMAHA_LEGACY",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "CHROMIUM" => Some(Self::Chromium),
            "SEMVER" => Some(Self::Semver),
            "OMAHA_LEGACY" => Some(Self::OmahaLegacy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CupPolicy {
    #[default]
    Required,
    Optional,
}

impl CupPolicy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Required => "REQUIRED",
            Self::Optional => "OPTIONAL",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "REQUIRED" => Some(Self::Required),
            "OPTIONAL" => Some(Self::Optional),
            _ => None,
        }
    }
}
