use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, crate::Error>;

/// Path error
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, thiserror::Error)]
pub enum PathError {

	/// When the path contains consecutive separators.
    #[error("The path contains consecutive separators.")]
    ConsecutiveSeparator,

	/// When the path does not contain a filename.
    #[error("The path does not contain a filename.")]
    DoesNotContainFileName,

	/// When the path does not contain a subdirectory.
    #[error("The path does not contain a subdirectory.")]
    DoesNotContainSubDir,

	/// When the path is empty.
	#[error("The path is empty.")]
    Empty,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {

	#[error("This device is not running Android. This plugin is only supported on Android.")]
	NotAndroid,

	#[error(transparent)]
	Path(#[from] PathError),

	#[error(transparent)]
	Io(#[from] std::io::Error),
  
	#[error(transparent)]
	PluginInvoke(#[from] anyhow::Error),
}

#[cfg(target_os = "android")]
impl From<tauri::plugin::mobile::PluginInvokeError> for crate::Error {

	fn from(value: tauri::plugin::mobile::PluginInvokeError) -> Self {
		Self::PluginInvoke(anyhow::anyhow!("{value}"))
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