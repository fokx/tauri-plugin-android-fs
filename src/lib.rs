//! # Overview
//!
//! The Android file system is strict and complex because its behavior and the available APIs vary depending on the version.  
//! This plugin was created to provide explicit and consistent file operations.  
//! No special permission or configuration is required.  
//!
//! # Setup
//! All you need to do is register the core plugin with Tauri: 
//!
//! `src-tauri/src/lib.rs`
//!
//! ```no_run
//! #[cfg_attr(mobile, tauri::mobile_entry_point)]
//! pub fn run() {
//!    tauri::Builder::default()
//!        .plugin(tauri_plugin_android_fs::init()) // This
//!        .run(tauri::generate_context!())
//!        .expect("error while running tauri application");
//! }
//! ```
//!
//! # Usage
//! There are three main ways to manipulate files:
//!
//! ### 1. Dialog
//! Opens the file/folder picker to read and write user-selected entries.
//!
//! ```no_run
//! use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt};
//!
//! fn read_files(app: tauri::AppHandle) {
//!     let api = app.android_fs();
//!     let selected_paths = api.show_open_file_dialog(
//!         &["*/*"], // Target MIME types
//!         true // Allow multiple files
//!     ).unwrap();
//!
//!     if selected_paths.is_empty() {
//!         // Handle cancel
//!     }
//!     else {
//!         for path in selected_paths {
//!             let file_name = api.get_file_name(&path).unwrap();
//!             let file: std::fs::File = api.open_file(&path).unwrap();
//!             // Handle read-only file
//! 
//!             // Alternatively, the path can be returned to the front end, 
//!             // and file processing can be handled within another tauri::command function that takes it as an argument.
//!             // If you need to use file data on the front end, 
//!             // consider using Tauri’s custom protocols for efficient transmission.
//!         }
//!     }
//! }
//! ```
//! 
//! ```no_run
//! use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt};
//! 
//! fn write_file(app: tauri::AppHandle) {
//!     let api = app.android_fs();
//!     let selected_path = api.show_save_file_dialog(
//!         "", // Initial file name
//!         Some("image/png") // Target MIME type
//!     ).unwrap();
//!
//!     if let Some(path) = selected_path {
//!         let mut file: std::fs::File = api.create_file(&path).unwrap();
//!         // Handle write-only file
//!     }
//!     else {
//!         // Handle cancel
//!     }
//! }
//! ```
//!
//! ### 2. Public Storage
//! File storage that is available to other applications and users.
//!
//! ```no_run
//! use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PublicImageDir, PublicStorage};
//!
//! fn example(app: tauri::AppHandle) {
//!     let storage = app.android_fs().public_storage();
//!     let contents: Vec<u8> = todo!();
//!
//!     // Write a PNG image
//!     storage.write_image(
//!         PublicImageDir::Pictures, // Base directory
//!         "myApp/2025-02-13.png", // Relative file path
//!         Some("image/png"), // MIME type
//!         &contents
//!     ).unwrap();
//! }
//! ```
//!
//! ### 3. Private Storage
//! File storage intended for the app’s use only.
//!
//! ```no_run
//! use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};
//!
//! fn example(app: tauri::AppHandle) {
//!     let storage = app.android_fs().private_storage();
//!     let contents: Vec<u8> = todo!();
//!
//!     // Write contents
//!     storage.write(
//!         PrivateDir::Data, // Base directory
//!         "config/data1", // Relative file path
//!         &contents
//!     ).unwrap();
//!
//!     // Read contents
//!     let contents = storage.read(
//!         PrivateDir::Data, // Base directory
//!         "config/data1" // Relative file path
//!     ).unwrap();
//! }
//! ```
//!
//! # License
//! MIT OR Apache-2.0

mod models;
mod impls;
mod error;

use std::io::{Read, Write};

pub use models::*;
pub use error::{Error, Result, PathError};
pub use impls::{AndroidFsExt, init};

/// API
pub trait AndroidFs {

    /// Verify whether this plugin is available.  
    /// 
    /// On Android, this returns true.  
    /// On other platforms, this returns false.  
    fn is_available(&self) -> bool {
        #[cfg(not(target_os = "android"))] {
            false
        }
        #[cfg(target_os = "android")] {
            true
        }
    }

    /// Get the file name.  
    /// 
    /// # Note
    /// This is a little slow.  
    /// 
    /// # Support
    /// All Android version.
    fn get_file_name(&self, path: &FilePath) -> crate::Result<String>;

    /// Get the directory name.
    /// 
    /// # Note
    /// This is a little slow.  
    /// 
    /// # Support
    /// All Android version.
    fn get_dir_name(&self, path: &DirPath) -> crate::Result<String>;

    /// Query provider for mime type.  
    /// If the type is unknown, this returns `application/octet-stream`.  
    /// 
    /// # Note
    /// This is a little slow.  
    /// 
    /// # Support
    /// All Android version.
    fn get_mime_type(&self, path: &FilePath) -> crate::Result<String>;

    /// Open a file in read-only mode.
    /// 
    /// If you only need to read the entire file contents, consider using [`AndroidFs::read`] or [`AndroidFs::read_to_string`] instead.  
    /// 
    /// # Support
    /// All Android version.
    fn open_file(&self, path: &FilePath) -> crate::Result<std::fs::File>;

    /// ***Opens*** a file in write-only mode from ***writable*** `FilePath`.  
    /// This function will create a file if it does not exist, and will truncate it if it does.  
    /// 
    /// If you only need to write the contents, consider using [`AndroidFs::write`] instead.  
    /// If you want to create a new file with [`DirPath`], use [`AndroidFs::new_file`].  
    /// If you want to create a new file, use [`PublicStorage::write`], [`PrivateStorage::write`].  
    /// 
    /// # Note
    /// A **writable** `FilePath` can be obtained from [`AndroidFs::show_save_file_dialog`], 
    /// but NOT from [`AndroidFs::show_open_file_dialog`] or [`AndroidFs::show_open_visual_media_dialog`].
    /// 
    /// # Support
    /// All Android version.
    fn create_file(&self, path: &FilePath) -> crate::Result<std::fs::File>;

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need to operate on a readable file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Support
    /// All Android version.
    fn read(&self, path: &FilePath) -> crate::Result<Vec<u8>> {
        let mut file = self.open_file(path)?;
        let mut buf = file.metadata().ok()
            .map(|m| m.len() as usize)
            .map(Vec::with_capacity)
            .unwrap_or_else(Vec::new);

        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Creates a new file at the specified location, and return **writable** `FilePath`.    
    /// If the same file name already exists, a sequential number is added to the name. And recursively create sub directories if they are missing.  
    /// 
    /// If you only need to write the contents, consider using [`AndroidFs::new_file_with_contents`]  instead.  
    /// 
    /// # Note
    /// `mime_type`  specify the type of the file to be created. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// # Support
    /// All Android version.
    fn new_file(
        &self,
        base_dir: &DirPath, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FilePath>;

    /// Creates a new file at the specified location, and write the entire contents, then return **writable** `FilePath`.    
    /// If the same file name already exists, a sequential number is added to the name. And recursively create sub directories if they are missing.  
    /// 
    /// # Note
    /// `mime_type`  specify the type of the file to be created. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// # Support
    /// All Android version.
    fn new_file_with_contents(
        &self,
        base_dir: &DirPath, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<FilePath> {

        let path = self.new_file(base_dir, relative_path, mime_type)?;
        self.write(&path, contents)?;
        Ok(path)
    }

    /// Remove the file from the **writable** `FilePath`.
    /// 
    /// # Support
    /// All Android version.
    fn remove_file(&self, path: &FilePath) -> crate::Result<()>;

    /// Reads the entire contents of a file into a string.  
    /// 
    /// If you need to operate on a readable file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Support
    /// All Android version.
    fn read_to_string(&self, path: &FilePath) -> crate::Result<String> {
        let mut file = self.open_file(path)?;
        let mut buf = file.metadata().ok()
            .map(|m| m.len() as usize)
            .map(String::with_capacity)
            .unwrap_or_else(String::new);

        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Returns the unordered child entries of the specified directory.  
    /// Returned [`Entry`](crate::Entry) contains **writable** [`FilePath`] or [`DirPath`].  
    ///
    /// # Note
    /// By default, child [`DirPath`] and [`FilePath`] are valid until the app is terminated.  
    /// To persist child `FilePath` across app restarts, use [`AndroidFs::grant_persistable_dir_access`] on the **parent** `DirPath`.  
    /// To persist child `DirPath` across app restarts, use [`AndroidFs::grant_persistable_dir_access`] on the parent or childself `DirPath`.  
    /// 
    /// `DirPath` can be obtained from functions such as [`AndroidFs::show_open_dir_dialog`].  
    /// 
    /// # Support
    /// All Android version.
    fn read_dir(&self, path: &DirPath) -> crate::Result<Vec<Entry>>;

    /// Writes a slice as the entire contents of a file in a **writable** `FilePath`.  
    /// This function will create a file if it does not exist, and will entirely replace its contents if it does.  
    /// 
    /// If you want to write to a file, use [`AndroidFs::create_file`] instead.  
    /// 
    /// # Note
    /// A **writable** `FilePath` can be obtained from [`AndroidFs::show_save_file_dialog`], 
    /// but not from [`AndroidFs::show_open_file_dialog`] or [`AndroidFs::show_open_visual_media_dialog`].
    /// 
    /// # Support
    /// All Android version.
    fn write(&self, path: &FilePath, contents: impl AsRef<[u8]>) -> crate::Result<()> {
        let mut file = self.create_file(path)?;
        file.write_all(contents.as_ref())?;
        Ok(())
    }

    /// Open a dialog for file selection.  
    /// This returns a **read-only** `FilePath`  vec. If no file is selected or canceled by user, it is an empty vec.  
    /// 
    /// For images and videos, consider using [`AndroidFs::show_open_visual_media_dialog`] instead.  
    /// 
    /// # Note
    /// `mime_types` represents the types of files that should be selected. 
    /// However, there is no guarantee that the returned file will match the specified types.    
    /// If this is empty, all file types will be available for selection. 
    /// This is equivalent to `["*/*"]`, and it will invoke the standard file picker in most cases.  
    /// 
    /// By default, this [`FilePath`] is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::grant_persistable_file_access`].
    /// 
    /// # Support
    /// All Android version.
    fn show_open_file_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FilePath>>;

    /// Opens a dialog for media file selection, such as images and videos.  
    /// This returns a **read-only** `FilePath` vec. If no file is selected or canceled by user, it is an empty vec.  
    /// 
    /// This is more user-friendly than [`AndroidFs::show_open_file_dialog`].  
    ///
    /// # Note
    /// By default, this [`FilePath`] is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::grant_persistable_file_access`].  
    /// 
    /// The file obtained from this function cannot retrieve the correct file name using [`AndroidFs::get_file_name`].
    /// Instead, it will be assigned a sequential number, such as `1000091523.png`.  
    /// <https://issuetracker.google.com/issues/268079113>  
    ///
    /// # Support
    /// This is available on devices that meet the following criteria:
    ///  - Run Android 11 (API level 30) or higher
    ///  - Receive changes to Modular System Components through Google System Updates
    ///  
    /// Availability on a given device can be verified by calling [`AndroidFs::is_visual_media_dialog_available`].  
    /// If not supported, this functions the same as [`AndroidFs::show_open_file_dialog`].
    fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool
    ) -> crate::Result<Vec<FilePath>>;

    /// Open a dialog for directory selection,
    /// allowing the app to read and write any file in the selected directory and its subdirectories.  
    /// If canceled by the user, return None.    
    /// 
    /// # Note
    /// By default, this [`DirPath`] is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::grant_persistable_dir_access`].
    /// 
    /// # Support
    /// All Android version.
    ///
    /// # Related functions
    /// - [`AndroidFs::new_file`]
    /// - [`AndroidFs::new_file_with_contents`]
    /// - [`AndroidFs::read_dir`]
    /// - [`AndroidFs::get_dir_name`]  
    /// - [`AndroidFs::grant_persistable_dir_access`]  
    /// - etc...
    fn show_open_dir_dialog(&self) -> crate::Result<Option<DirPath>>;

    /// Open a dialog for file saving, and write contents to the selected file, then return that path.    
    /// This returns a **writable** `FilePath` . If canceled by the user, return None, and do not write it. 
    /// 
    /// If you want to write to a file, use [`AndroidFs::show_save_file_dialog`]  then [`AndroidFs::create_file`] insted.   
    /// 
    /// When storing media files such as images, videos, and audio, consider using [`PublicStorage::write_image`] or a similar method.  
    /// When a file does not need to be accessed by other applications and users, consider using [`PrivateStorage::write`].  
    /// These are easier because the destination does not need to be selected in a dialog.  
    /// 
    /// # Note
    /// `mime_type`  specify the type of the target file to be saved. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// By default, this [`FilePath`] is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::grant_persistable_file_access`].
    /// 
    /// # Support
    /// All Android version.
    fn show_save_file_dialog_with_contents(
        &self,
        default_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<Option<FilePath>> {

        if let Some(path) = self.show_save_file_dialog(default_file_name, mime_type)? {
            self.write(&path, contents)?;
            return Ok(Some(path))
        }
        
        Ok(None)
    }

    /// Open a dialog for file saving, and return the selected path.  
    /// This returns a **writable** `FilePath` . If canceled by the user, return None.  
    /// 
    /// If you only need to write contents, use [`AndroidFs::show_save_file_dialog_with_contents`] instead.  
    /// 
    /// When storing media files such as images, videos, and audio, consider using [`PublicStorage::write_image`] or a similar method.  
    /// When a file does not need to be accessed by other applications and users, consider using  [`PrivateStorage::write`].  
    /// These are easier because the destination does not need to be selected in a dialog.  
    /// 
    /// # Note
    /// `mime_type` specify the type of the target file to be saved. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// By default, this [`FilePath`] is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::grant_persistable_file_access`].
    /// 
    /// # Support
    /// All Android version.
    fn show_save_file_dialog(
        &self,
        default_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FilePath>>;

    /// Take persistent permission to access the file.  
    /// 
    /// Preserve access across app and device restarts. 
    /// If you only need it until the end of the app session, you do not need to use this.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.  
    /// 
    /// # Note
    /// [`PersistableAccessMode::WriteOnly`] and [`PersistableAccessMode::ReadAndWrite`] are only for **writable** [`FilePath`].  
    /// 
    /// Even after calling this, the app doesn't retain access to the file if the file is moved or deleted. 
    /// 
    /// # Helper
    /// There are helper functions, such as [`convert_file_path_to_string`] and [`convert_string_to_file_path`], for storing [`FilePath`].   
    /// 
    /// # Support
    /// All Android version.
    fn grant_persistable_file_access(&self, path: &FilePath, mode: PersistableAccessMode) -> crate::Result<()>;

    /// Take persistent permission to access the directory and its entries.  
    /// 
    /// Preserve access across app and device restarts. 
    /// If you only need it until the end of the app session, you do not need to use this.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.  
    /// 
    /// # Note
    /// Even after calling this, the app doesn't retain access to the directory if the directory is moved or deleted.  
    /// 
    /// # Helper
    /// There are helper functions, such as [`convert_dir_path_to_string`] and [`convert_string_to_dir_path`], for storing [`DirPath`].   
    /// 
    /// # Support
    /// All Android version.
    fn grant_persistable_dir_access(&self, path: &DirPath, mode: PersistableAccessMode) -> crate::Result<()>;

    /// Use [`AndroidFs::grant_persistable_file_access`] instead.
    /// 
    /// This is same as following.
    /// ```ignore
    /// self.grant_persistable_file_access(path, PersistableAccessMode::ReadOnly)
    /// ```
    #[deprecated(note = "Use AndroidFs::grant_persistable_file_access instead.")]
    #[warn(deprecated)]
    fn take_persistable_read_permission(&self, path: &FilePath) -> crate::Result<()> {
        self.grant_persistable_file_access(path, PersistableAccessMode::ReadOnly)
    }

    /// Use [`AndroidFs::grant_persistable_file_access`] instead.  
    /// 
    /// This is same as following.
    /// ```ignore
    /// self.grant_persistable_file_access(path, PersistableAccessMode::WriteOnly)
    /// ```
    #[deprecated(note = "Use AndroidFs::grant_persistable_file_access instead.")]
    #[warn(deprecated)]
    fn take_persistable_write_permission(&self, path: &FilePath) -> crate::Result<()> {
        self.grant_persistable_file_access(path, PersistableAccessMode::WriteOnly)
    }

    /// Verify whether [`AndroidFs::show_open_visual_media_dialog`] is available on a given device.
    /// 
    /// # Support
    /// All Android version.
    fn is_visual_media_dialog_available(&self) -> crate::Result<bool>;

    /// File storage API intended to be shared with other apps.
    fn public_storage(&self) -> &impl PublicStorage;

    /// File storage API intended for the app’s use only.
    fn private_storage(&self) -> &impl PrivateStorage;
}

/// File storage that is available to other applications and users.
pub trait PublicStorage {

    /// Save the contents to public storage, and return **writable** `FilePath`.  
    /// This is used when saving a file for access by other applications and users.  
    /// 
    /// If the same file name already exists, a sequential number is added to the name and saved.  
    /// 
    /// If you want to operate directly on a write-only file, use [`PublicStorage::write_with_contents_writer`] insted.  
    /// 
    /// When storing media files such as images, videos, and audio, consider using [`PublicStorage::write_image`] or a similar method.  
    /// For saving a general-purpose file, it is often better to use [`AndroidFs::show_save_file_dialog`].  
    /// 
    /// # Note
    /// Do not save files directly in the base directory. 
    /// Please specify a subdirectory in the `relative_path_with_sub_dir`, such as `appName/file.txt` or `appName/2025-2-11/file.txt`. 
    /// Do not use `file.txt`.  
    /// 
    /// # Support
    /// All Android version.
    fn write(
        &self,
        base_dir: PublicGeneralPurposeDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<FilePath> {

        self.write_with_contents_writer(
            base_dir,
            relative_path_with_sub_dir, 
            mime_type,
            |file| file.write_all(contents.as_ref())
        )
    }

    /// Save the contents as an image file to public storage, and return **writable** `FilePath`.   
    /// This is used when saving a file for access by other applications and users.  
    /// 
    /// If the same file name already exists, a sequential number is added to the name and saved.  
    /// 
    /// If you want to operate directly on a write-only file, use [`PublicStorage::write_image_with_contents_writer`] insted.  
    /// 
    /// # Note
    /// Do not set a non-image type to `mime_type`, as it may result in an error. 
    /// Even if the type is an image, an error will occur if the Android system does not recognize the type or contents as an image.   
    /// 
    /// Do not save files directly in the base directory. 
    /// Please specify a subdirectory in the `relative_path_with_sub_dir`, such as `appName/file.png` or `appName/2025-2-11/file.png`. 
    /// Do not use `file.png`.  
    /// 
    /// # Support
    /// All Android version.
    fn write_image(
        &self,
        base_dir: PublicImageDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<FilePath> {

        self.write_image_with_contents_writer(
            base_dir,
            relative_path_with_sub_dir, 
            mime_type,
            |file| file.write_all(contents.as_ref())
        )
    }

    /// Save the contents as an video file to public storage, and return **writable** `FilePath`.  
    /// This is used when saving a file for access by other applications and users.  
    /// 
    /// If the same file name already exists, a sequential number is added to the name and saved.  
    /// 
    /// If you want to operate directly on a write-only file, use [`PublicStorage::write_video_with_contents_writer`] insted.  
    /// 
    /// # Note
    /// Do not set a non-video type to `mime_type`, as it may result in an error. 
    /// Even if the type is an video, an error will occur if the Android system does not recognize the type or contents as an video.   
    /// 
    /// Do not save files directly in the base directory. 
    /// Please specify a subdirectory in the `relative_path_with_sub_dir`, such as `appName/file.mp4` or `appName/2025-2-11/file.mp4`. 
    /// Do not use `file.mp4`.  
    /// 
    /// # Support
    /// All Android version.
    fn write_video(
        &self,
        base_dir: PublicVideoDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<FilePath> {

        self.write_video_with_contents_writer(
            base_dir,
            relative_path_with_sub_dir, 
            mime_type,
            |file| file.write_all(contents.as_ref())
        )
    }

    /// Save the contents as an audio file to public storage, and return **writable** `FilePath`.    
    /// This is used when saving a file for access by other applications and users.  
    /// 
    /// If the same file name already exists, a sequential number is added to the name and saved.  
    /// 
    /// If you want to operate directly on a write-only file, use [`PublicStorage::write_audio_with_contents_writer`] insted.  
    /// 
    /// # Note
    /// Do not set a non-audio type to `mime_type`, as it may result in an error. 
    /// Even if the type is an audio, an error will occur if the Android system does not recognize the type or contents as an audio.   
    /// 
    /// Do not save files directly in the base directory. 
    /// Please specify a subdirectory in the `relative_path_with_sub_dir`, such as `appName/file.mp3` or `appName/2025-2-11/file.mp3`. 
    /// Do not use `file.mp3`.  
    /// 
    /// # Support
    /// [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.  
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// 
    /// [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.  
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// 
    /// Others are available in all Android versions.  
    fn write_audio(
        &self,
        base_dir: PublicAudioDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<FilePath> {

        self.write_audio_with_contents_writer(
            base_dir,
            relative_path_with_sub_dir, 
            mime_type,
            |file| file.write_all(contents.as_ref())
        )
    }

    /// See [`PublicStorage::write`] for description.
    ///
    /// # Note
    /// The file provided in `contents_writer` is write-only.
    /// 
    /// # Examples
    /// The following is equivalent to [`PublicStorage::write`].  
    /// ```ignore
    /// self.write_with_contents_writer(
    ///     base_dir,
    ///     relative_path_with_sub_dir, 
    ///     mime_type,
    ///     |file| file.write_all(contents)
    /// )
    /// ```
    fn write_with_contents_writer(
        &self,
        base_dir: PublicGeneralPurposeDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath>;

    /// See [`PublicStorage::write_image`] for description.
    ///
    /// # Note
    /// The file provided in `contents_writer` is write-only.
    /// 
    /// # Examples
    /// The following is equivalent to [`PublicStorage::write_image`].  
    /// ```ignore
    /// self.write_image_with_contents_writer(
    ///     base_dir,
    ///     relative_path_with_sub_dir, 
    ///     mime_type,
    ///     |file| file.write_all(contents)
    /// )
    /// ```
    fn write_image_with_contents_writer(
        &self,
        base_dir: PublicImageDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath>;

    /// See [`PublicStorage::write_video`] for description.
    ///
    /// # Note
    /// The file provided in `contents_writer` is write-only.
    /// 
    /// # Examples
    /// The following is equivalent to [`PublicStorage::write_video`].  
    /// ```ignore
    /// self.write_video_with_contents_writer(
    ///     base_dir,
    ///     relative_path_with_sub_dir, 
    ///     mime_type,
    ///     |file| file.write_all(contents)
    /// )
    /// ```
    fn write_video_with_contents_writer(
        &self,
        base_dir: PublicVideoDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath>;

    /// See [`PublicStorage::write_audio`] for description.
    ///
    /// # Note
    /// The file provided in `contents_writer` is write-only.
    /// 
    /// # Examples
    /// The following is equivalent to [`PublicStorage::write_audio`].  
    /// ```ignore
    /// self.write_audio_with_contents_writer(
    ///     base_dir,
    ///     relative_path_with_sub_dir, 
    ///     mime_type,
    ///     |file| file.write_all(contents)
    /// )
    /// ```
    fn write_audio_with_contents_writer(
        &self,
        base_dir: PublicAudioDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath>;

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.
    /// 
    /// # Support
    /// All Android version.
    fn is_audiobooks_dir_available(&self) -> crate::Result<bool>;

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.
    /// 
    /// # Support
    /// All Android version.
    fn is_recordings_dir_available(&self) -> crate::Result<bool>;
}

/// File storage intended for the app’s use only.  
pub trait PrivateStorage {

    /// Get the absolute path of the specified directory.  
    /// Apps require no extra permissions to read or write to the returned path, since this path lives in their private storage.  
    ///
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    /// When using [`PrivateDir::Cache`], the system will automatically delete files in this directory as disk space is needed elsewhere on the device.   
    /// 
    /// The returned path may change over time if the calling app is moved to an adopted storage device, so only relative paths should be persisted.   
    /// 
    /// # Examples
    /// ```no_run
    /// use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};
    /// 
    /// fn example(app: tauri::AppHandle) {
    ///     let api = app.android_fs().private_storage();
    /// 
    ///     let dir_path = api.resolve_path(PrivateDir::Data).unwrap();
    ///     let file_path = dir_path.join("2025-2-12/data.txt");
    ///     
    ///     // Write file
    ///     std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    ///     std::fs::write(&file_path, "aaaa").unwrap();
    /// 
    ///     // Read file
    ///     let _ = std::fs::read_to_string(&file_path).unwrap();
    /// 
    ///     // Remove file
    ///     std::fs::remove_file(&file_path).unwrap();
    /// 
    ///     // Remove all files in the dir.
    ///     std::fs::remove_dir_all(&dir_path).unwrap();
    /// }
    /// ```
    /// 
    /// # Support
    /// All Android version.
    fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf>;

    /// Get the absolute path of the specified relative path and base directory.  
    /// Apps require no extra permissions to read or write to the returned path, since this path lives in their private storage.  
    ///
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn resolve_path_with(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>
    ) -> crate::Result<std::path::PathBuf> {

        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let path = self.resolve_path(base_dir)?.join(relative_path);
        Ok(path)
    }

    /// Writes a slice as the entire contents of a file.  
    /// 
    /// This function will create a file if it does not exist, and will entirely replace its contents if it does.  
    /// Recursively create parent directories if they are missing.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] , [`std::fs::create_dir_all`], and [`std::fs::write`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn write(
        &self, 
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
        contents: impl AsRef<[u8]>
    ) -> crate::Result<()> {

        let path = self.resolve_path_with(base_dir, relative_path)?;

        if let Some(parent_dir) = path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        std::fs::write(path, contents)?;

        Ok(())
    }

    /// Open a file in read-only mode.  
    /// 
    /// If you only need to read the entire file contents, consider using [`PrivateStorage::read`]  or [`PrivateStorage::read_to_string`] instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::open`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn open_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::File::open(path)?)
    }

    /// Opens a file in write-only mode.  
    /// This function will create a file if it does not exist, and will truncate it if it does.
    /// 
    /// If you only need to write the contents, consider using [`PrivateStorage::write`]  instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::create`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn create_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::File::create(path)?)
    }

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need [`std::fs::File`], use [`PrivateStorage::open_file`] insted.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn read(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<Vec<u8>> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::read(path)?)
    }

    /// Reads the entire contents of a file into a string.  
    /// 
    /// If you need [`std::fs::File`], use [`PrivateStorage::open_file`] insted.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read_to_string`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn read_to_string(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<String> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::read_to_string(path)?)
    }

    /// Returns an iterator over the entries within a directory.
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read_dir`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn read_dir(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<std::fs::ReadDir> {

        let path = match relative_path {
            Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
            None => self.resolve_path(base_dir)?,
        };

        Ok(std::fs::read_dir(path)?)
    }

    /// Removes a file from the filesystem.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_file`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn remove_file(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>,
    ) -> crate::Result<()> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::remove_file(path)?)
    }

    /// Removes an empty directory.  
    /// If you want to remove a directory that is not empty, as well as all of its contents recursively, consider using [`PrivateStorage::remove_dir_all`] instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_dir`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn remove_dir(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<()> {

        let path = match relative_path {
            Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
            None => self.resolve_path(base_dir)?,
        };

        std::fs::remove_dir(path)?;
        Ok(())
    }

    /// Removes a directory at this path, after removing all its contents. Use carefully!  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_dir_all`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn remove_dir_all(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<()> {

        let path = match relative_path {
            Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
            None => self.resolve_path(base_dir)?,
        };

        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    /// Returns Ok(true) if the path points at an existing entity.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::exists`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn exists(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>
    ) -> crate::Result<bool> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::exists(path)?)
    }

    /// Queries the file system to get information about a file, directory.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::metadata`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn metadata(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<std::fs::Metadata> {

        let path = match relative_path {
            Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
            None => self.resolve_path(base_dir)?,
        };

        Ok(std::fs::metadata(path)?)
    }
}

/// Convert [`FilePath`] to string.
pub fn convert_file_path_to_string(path: &FilePath) -> String {
    path.to_string()
}

/// Convert string to [`FilePath`].  
/// 
/// # Note
/// This does not query or validate the file system.  
pub fn convert_string_to_file_path(string: impl AsRef<str>) -> FilePath {
    let result: std::result::Result<_, std::convert::Infallible> = string.as_ref().parse();

    // This will not cause panic. Because result is infallible.
    result.unwrap()
}

/// Convert [`DirPath`] to string.  
pub fn convert_dir_path_to_string(path: &DirPath) -> serde_json::Result<String> {
    serde_json::to_string(path)
}

/// Convert string to [`DirPath`].  
///
/// # Note
/// This does not query or validate the file system.
pub fn convert_string_to_dir_path(string: impl AsRef<str>) -> serde_json::Result<DirPath> {
    serde_json::from_str(string.as_ref())
}