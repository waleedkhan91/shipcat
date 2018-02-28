use std::fmt;

use super::traits::Verify;
use super::Result;

/// Image to run in a pod
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Image {
    /// Name of service relied upon
    pub name: Option<String>,
    /// Repository to fetch the image from (can be empty string)
    pub repository: Option<String>,
    /// Tag to fetch the image from (defaults to latest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Filled in display value
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = self.repository.clone().map(|s| {
            if s != "" { format!("{}/", s) } else { s }
        }).unwrap_or_else(|| "".into());
        let suffix = self.tag.clone().unwrap_or_else(|| "latest".to_string());
        // NB: assume image.name is always set at this point
        write!(f, "{}{}:{}", prefix, self.name.clone().unwrap(), suffix)
    }
}

impl Verify for Image {
    fn verify(&self) -> Result<()> {
        // TODO: warn if it doesn't start with default docker repository
        Ok(())
    }
}