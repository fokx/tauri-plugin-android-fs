use std::time::SystemTime;
use serde::{Deserialize, Serialize};


/// Path to represent a file or directory.
/// 
/// # Note
/// For compatibility, an interconversion to [`tauri_plugin_fs::FilePath`] is implemented, such as follwing. 
/// This may be lossy and also not guaranteed to work properly with other plugins. 
/// But at least, it can be used with [`convertFileSrc`](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc).
/// ```no_run
/// use tauri_plugin_android_fs::FileUri;
/// use tauri_plugin_fs::FilePath;
/// 
/// let uri: FileUri = unimplemented!();
/// 
/// // this can use with convertFileSrc on frontend
/// let path: FilePath = uri.into();
/// 
/// let uri: FileUri = path.into();
/// ```
/// 
/// # Typescript type
/// You should use the following type because it might change in the future, and the inner value should not be used directly.  
/// ```typescript
/// type FileUri = any
/// type FileUri = string
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUri {
    pub(crate) uri: String,
    pub(crate) document_top_tree_uri: Option<String>,
}

impl From<tauri_plugin_fs::FilePath> for FileUri {

    fn from(value: tauri_plugin_fs::FilePath) -> Self {
        let uri = match value {
            tauri_plugin_fs::FilePath::Url(url) => url.to_string(),
            tauri_plugin_fs::FilePath::Path(path_buf) => format!("file://{}", path_buf.to_string_lossy()),
        };

        Self { uri, document_top_tree_uri: None }
    }
}

impl From<FileUri> for tauri_plugin_fs::FilePath {

    fn from(value: FileUri) -> Self {
        let result: std::result::Result<_, std::convert::Infallible> = value.uri.parse();

        // This will not cause panic. Because result err is infallible.
        result.unwrap()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Entry {

    #[non_exhaustive]
    File {
        name: String,
        uri: FileUri,
        last_modified: SystemTime,
        byte_size: u64,
        mime_type: String,
    },

    #[non_exhaustive]
    Dir {
        name: String,
        uri: FileUri,
        last_modified: SystemTime,
    }
}

/// Access mode
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum PersistableAccessMode {

    /// Read-only access.
    ReadOnly,

    /// Write-only access.
    WriteOnly,

    /// Read-write access.
    ReadAndWrite,
}

/// Access mode
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum FileAccessMode {

    /// Opens the file in read-only mode.
    /// 
    /// FileDescriptor mode: "r"
    Read,

    /// Opens the file in write-only mode.
    /// **This may or may not truncate.**
    /// So please use `WriteTruncate` or `WriteAppend` instead.
    ///
    /// FileDescriptor mode: "w"
    Write,

    /// Opens the file in write-only mode.
    /// The existing content is truncated (deleted), and new data is written from the beginning.
    /// Creates a new file if it does not exist.
    ///
    /// FileDescriptor mode: "wt"
    WriteTruncate,

    /// Opens the file in write-only mode.
    /// The existing content is preserved, and new data is appended to the end of the file.
    /// Creates a new file if it does not exist.
    /// 
    /// FileDescriptor mode: "wa"
    WriteAppend,

    /// Opens the file in read-write mode.  
    /// 
    /// FileDescriptor mode: "rw"
    ReadWrite,

    /// Opens the file in read-write mode.
    /// The existing content is truncated (deleted), and new data is written from the beginning.
    /// Creates a new file if it does not exist.
    ///
    /// FileDescriptor mode: "rwt"
    ReadWriteTruncate,
}

/// Filters for VisualMediaPicker.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum VisualMediaTarget {

    /// Allow only images to be selected.  
    ImageOnly,

    /// Allow only videos to be selected.  
    VideoOnly,

    /// Allow only images and videos to be selected.  
    ImageAndVideo
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// The system prevents other apps from accessing these locations, and on Android 10 (API level 29) and higher, these locations are encrypted.  
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// ex: `/data/user/0/{app-package-name}/files`
    Data,

    /// The application specific cache directory.  
    /// 
    /// The system prevents other apps from accessing these locations, and on Android 10 (API level 29) and higher, these locations are encrypted.  
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    /// In addition, the system will automatically delete files in this directory as disk space is needed elsewhere on the device.  
    /// 
    /// ex: `/data/user/0/{app-package-name}/cache`
    Cache,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicDir {
    
    #[serde(untagged)]
    Image(PublicImageDir),

    #[serde(untagged)]
    Video(PublicVideoDir),

    #[serde(untagged)]
    Audio(PublicAudioDir),

    #[serde(untagged)]
    GeneralPurpose(PublicGeneralPurposeDir),
}

/// Directory in which to place images that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicImageDir {

    /// Standard directory in which to place pictures that are available to the user.  
    /// 
    /// ex: `~/Pictures/{app_name}`
    Pictures,

    /// The traditional location for pictures and videos when mounting the device as a camera.  
    /// 
    /// ex: `~/DCIM/{app_name}`
    DCIM,
}

/// Directory in which to place videos that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicVideoDir {

	/// Standard directory in which to place movies that are available to the user.  
	/// 
	/// ex: `~/Movies/{app_name}`
	Movies,

	/// The traditional location for pictures and videos when mounting the device as a camera.  
	/// 
	/// ex: `~/DCIM/{app_name}`
	DCIM,
}

/// Directory in which to place audios that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicAudioDir {

    /// Standard directory in which to place movies that are available to the user.  
    /// 
    /// ex: `~/Music/{app_name}`
    Music,

    /// Standard directory in which to place any audio files that should be in the list of alarms that the user can select (not as regular music).  
    /// 
    /// ex: `~/Alarms/{app_name}`
    Alarms,

    /// Standard directory in which to place any audio files that should be in the list of audiobooks that the user can select (not as regular music).  
    /// 
    /// This is not available on Android 9 (API level 28) and lower.  
    /// 
    /// ex: `~/Audiobooks/{app_name}`  
    Audiobooks,

    /// Standard directory in which to place any audio files that should be in the list of notifications that the user can select (not as regular music).  
    /// 
    /// ex: `~/Notifications/{app_name}`
    Notifications,

    /// Standard directory in which to place any audio files that should be in the list of podcasts that the user can select (not as regular music).  
    /// 
    /// ex: `~/Podcasts/{app_name}`
    Podcasts,

    /// Standard directory in which to place any audio files that should be in the list of ringtones that the user can select (not as regular music).  
    /// 
    /// ex: `~/Ringtones/{app_name}`
    Ringtones,

    /// Standard directory in which to place any audio files that should be in the list of voice recordings recorded by voice recorder apps that the user can select (not as regular music).   
    /// 
    /// This is not available on Android 11 (API level 30) and lower.  
    /// 
    /// ex: `~/Recordings/{app_name}`
    Recordings,
}

/// Directory in which to place files that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicGeneralPurposeDir {

    /// Standard directory in which to place documents that have been created by the user.  
    /// 
    /// ex: `~/Documents/{app_name}`
    Documents,

    /// Standard directory in which to place files that have been downloaded by the user.  
    /// 
    /// ex: `~/Download/{app_name}`  
    Download,
}


macro_rules! impl_into_pubdir {
    ($target: ident, $wrapper: ident) => {
        impl From<$target> for PublicDir {
            fn from(value: $target) -> Self {
                Self::$wrapper(value)
            }
        }
    };
}
impl_into_pubdir!(PublicImageDir, Image);
impl_into_pubdir!(PublicVideoDir, Video);
impl_into_pubdir!(PublicAudioDir, Audio);
impl_into_pubdir!(PublicGeneralPurposeDir, GeneralPurpose);