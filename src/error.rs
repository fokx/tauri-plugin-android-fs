use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {

    #[error("This device is not running Android. This plugin is only supported on Android.")]
    NotAndroid,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
  
    #[error("{0}")]
    PluginInvoke(String),
}

#[cfg(target_os = "android")]
impl From<tauri::plugin::mobile::PluginInvokeError> for crate::Error {

    fn from(value: tauri::plugin::mobile::PluginInvokeError) -> Self {
        Self::PluginInvoke(format!("{value}"))
    }
}

impl Serialize for crate::Error {

    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}