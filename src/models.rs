use std::time::SystemTime;
use serde::{Deserialize, Serialize};


/// Path to represent a file or directory.
/// 
/// # Note
/// For compatibility, an interconversion to [`tauri_plugin_fs::FilePath`] is implemented, such as follwing.  
/// This is lossy and also not guaranteed to work properly with other plugins.  
/// However, reading and writing files by official [`tauri_plugin_fs`] etc. should work well.  
/// ```no_run
/// use tauri_plugin_android_fs::FileUri;
/// use tauri_plugin_fs::FilePath;
/// 
/// let uri: FileUri = unimplemented!();
/// 
/// let path: FilePath = uri.into();
/// 
/// let uri: FileUri = path.into();
/// ```
/// 
/// # Typescript type
/// You should use the following type because the inner value should not be used directly.  
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

impl FileUri {

    pub fn to_string(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    pub fn from_str(s: &str) -> crate::Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }
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
        uri: FileUri,
        name: String,
        last_modified: SystemTime,
        len: u64,
        mime_type: String,
    },

    #[non_exhaustive]
    Dir {
        uri: FileUri,
        name: String,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PersistedUriPermission {
    File {
        uri: FileUri,
        can_read: bool,
        can_write: bool,
    },
    Dir {
        uri: FileUri,
        can_read: bool,
        can_write: bool,
    }
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VisualMediaTarget {

    /// Allow only images to be selected.  
    ImageOnly,

    /// Allow only videos to be selected.  
    VideoOnly,

    /// Allow only images and videos to be selected.  
    ImageAndVideo,
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// ex: `/data/user/0/{app-package-name}/files`
    Data,

    /// The application specific cache directory.  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
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
    /// ex: `~/Pictures`
    Pictures,

    /// The traditional location for pictures and videos when mounting the device as a camera.  
    /// 
    /// ex: `~/DCIM`
    DCIM,
}

/// Directory in which to place videos that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicVideoDir {

	/// Standard directory in which to place movies that are available to the user.  
	/// 
	/// ex: `~/Movies`
	Movies,

	/// The traditional location for pictures and videos when mounting the device as a camera.  
	/// 
	/// ex: `~/DCIM`
	DCIM,
}

/// Directory in which to place audios that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicAudioDir {

    /// Standard directory in which to place movies that are available to the user.  
    /// 
    /// ex: `~/Music`
    Music,

    /// Standard directory in which to place any audio files that should be in the list of alarms that the user can select (not as regular music).  
    /// 
    /// ex: `~/Alarms`
    Alarms,

    /// Standard directory in which to place any audio files that should be in the list of audiobooks that the user can select (not as regular music).  
    /// 
    /// This is not available on Android 9 (API level 28) and lower.  
    /// 
    /// ex: `~/Audiobooks`  
    Audiobooks,

    /// Standard directory in which to place any audio files that should be in the list of notifications that the user can select (not as regular music).  
    /// 
    /// ex: `~/Notifications`
    Notifications,

    /// Standard directory in which to place any audio files that should be in the list of podcasts that the user can select (not as regular music).  
    /// 
    /// ex: `~/Podcasts`
    Podcasts,

    /// Standard directory in which to place any audio files that should be in the list of ringtones that the user can select (not as regular music).  
    /// 
    /// ex: `~/Ringtones`
    Ringtones,

    /// Standard directory in which to place any audio files that should be in the list of voice recordings recorded by voice recorder apps that the user can select (not as regular music).   
    /// 
    /// This is not available on Android 11 (API level 30) and lower.  
    /// 
    /// ex: `~/Recordings`
    Recordings,
}

/// Directory in which to place files that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicGeneralPurposeDir {

    /// Standard directory in which to place documents that have been created by the user.  
    /// 
    /// ex: `~/Documents`
    Documents,

    /// Standard directory in which to place files that have been downloaded by the user.  
    /// 
    /// ex: `~/Download`  
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