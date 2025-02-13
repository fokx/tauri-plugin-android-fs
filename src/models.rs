use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum VisualMediaTarget {
	ImageOnly,
	VideoOnly,
	ImageAndVideo
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum PrivateDir {

	/// The application specific persistent-data directory.  
	/// 
	/// The system prevents other apps from accessing these locations, and on Android 10 (API level 29) and higher, these locations are encrypted.  
	///  
	/// These files will be deleted when the app is uninstalled, and may also be deleted by the user.  
	/// 
	/// ex: `/data/user/0/{app-package-name}/files`
	Data,

	/// The application specific cache directory.  
	/// 
	/// The system prevents other apps from accessing these locations, and on Android 10 (API level 29) and higher, these locations are encrypted.  
	/// 
	/// These files will be deleted when the app is uninstalled, and may also be deleted by the user.  
	/// The system will automatically delete files in this directory as disk space is needed elsewhere on the device.  
	/// 
	/// ex: `/data/user/0/{app-package-name}/cache`
	Cache,
}

/// Directory in which to place images that are available to the user.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
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

/// Directory in which to place videos that are available to the user.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
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

/// Directory in which to place audios that are available to the user.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
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
    /// Availability on a given device can be verified by calling `PublicStorage::is_audiobooks_dir_available`.  
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
    /// Availability on a given device can be verified by calling `PublicStorage::is_recordings_dir_available`.  
	/// 
	/// ex: `~/Recordings`
	Recordings,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum PublicGeneralPurposeDir {

	/// ex: `~/Documents`
	Documents,

	/// ex: `~/Download`
	Download,
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum PublicDir {
	#[serde(untagged)]
	Image(PublicImageDir),

	#[serde(untagged)]
	Video(PublicVideoDir),

	#[serde(untagged)]
	Audio(PublicAudioDir),

	#[serde(untagged)]
	GeneralPurpose(PublicGeneralPurposeDir),
}