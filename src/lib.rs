//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

mod models;
mod impls;
mod error;

use std::io::{Read, Write};

pub use models::*;
#[allow(deprecated)]
pub use error::{Error, Result, PathError};
pub use impls::{AndroidFsExt, init};

/// API
pub trait AndroidFs<R: tauri::Runtime> {

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

    /// Get the file or directory name.  
    /// 
    /// # Support
    /// All Android version.
    fn get_name(&self, uri: &FileUri) -> crate::Result<String>;

    /// Query the provider to get mime type.  
    /// If the directory, this returns `None`.  
    /// If the file, this returns no `None`.  
    /// If the file type is unknown or unset, this returns `Some("application/octet-stream")`.  
    ///
    /// # Support
    /// All Android version.
    fn get_mime_type(&self, uri: &FileUri) -> crate::Result<Option<String>>;

    /// Queries the file system to get information about a file, directory.
    /// 
    /// # Note
    /// This uses [`AndroidFs::open_file`] internally.
    /// 
    /// # Support
    /// All Android version.
    fn get_metadata(&self, uri: &FileUri) -> crate::Result<std::fs::Metadata> {
        let file = self.open_file(uri, FileAccessMode::Read)?;
        Ok(file.metadata()?)
    }

    /// Open a file in the specified mode.
    /// 
    /// # Note
    /// Other than [`FileAccessMode::Read`] is only for **writable** uri.
    /// 
    /// # Support
    /// All Android version.
    fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> crate::Result<std::fs::File>;

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Support
    /// All Android version.
    fn read(&self, uri: &FileUri) -> crate::Result<Vec<u8>> {
        let mut file = self.open_file(uri, FileAccessMode::Read)?;
        let mut buf = file.metadata().ok()
            .map(|m| m.len() as usize)
            .map(Vec::with_capacity)
            .unwrap_or_else(Vec::new);

        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Reads the entire contents of a file into a string.  
    /// 
    /// If you need to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Support
    /// All Android version.
    fn read_to_string(&self, uri: &FileUri) -> crate::Result<String> {
        let mut file = self.open_file(uri, FileAccessMode::Read)?;
        let mut buf = file.metadata().ok()
            .map(|m| m.len() as usize)
            .map(String::with_capacity)
            .unwrap_or_else(String::new);
    
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// If you want to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Note
    /// This is only for **writable** file uri.
    /// 
    /// # Support
    /// All Android version.
    fn write(&self, uri: &FileUri, contents: impl AsRef<[u8]>) -> crate::Result<()> {
        let mut file = self.open_file(uri, FileAccessMode::WriteTruncate)?;
        file.write_all(contents.as_ref())?;
        Ok(())
    }

    /// Remove the file.
    /// 
    /// # Note
    /// This is only for **removeable** uri.
    /// 
    /// # Support
    /// All Android version.
    fn remove_file(&self, uri: &FileUri) -> crate::Result<()>;

    /// Remove the **empty** directory.
    /// 
    /// # Note
    /// This is only for **removeable** uri.
    /// 
    /// # Support
    /// All Android version.
    fn remove_dir(&self, uri: &FileUri) -> crate::Result<()>;

    /// Creates a new empty file at the specified directory, and returns **read-write-removeable** uri.    
    /// If the same file name already exists, a sequential number is added to the name. And recursively create sub directories if they are missing.  
    /// 
    /// # Note
    /// `mime_type`  specify the type of the file to be created. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// # Support
    /// All Android version.
    fn create_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri>;

    /// Please use [`PublicStorage::create_file_in_public_app_dir`] insted.
    #[deprecated = "Please use PublicStorage::create_file_in_public_app_dir insted."]
    #[warn(deprecated)]
    fn create_file_in_public_location(
        &self,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        self.public_storage().create_file_in_public_app_dir(dir, relative_path, mime_type)
    }

    /// Returns the unordered child entries of the specified directory.  
    /// Returned [`Entry`](crate::Entry) contains file or directory uri.
    ///
    /// # Note
    /// By default, children are valid until the app is terminated.  
    /// To persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`]. 
    /// However, if you have obtained persistent permissions for the origin directory (e.g. parent, grand parents), it is unnecessary.
    /// 
    /// The returned type is an iterator because of the data formatting and the file system call is not executed lazily.
    /// 
    /// # Support
    /// All Android version.
    fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>>;

    /// Take persistent permission to access the file, directory and its descendants.  
    /// 
    /// Preserve access across app and device restarts. 
    /// If you only need it until the end of the app session, you do not need to use this.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.  
    /// 
    /// # Note
    /// Even after calling this, the app doesn't retain access to the entry if it is moved or deleted.  
    /// 
    /// # Support
    /// All Android version.
    fn take_persistable_uri_permission(&self, uri: FileUri, mode: PersistableAccessMode) -> crate::Result<()>;

    /// Open a dialog for file selection.  
    /// This returns a **read-only** uris. If no file is selected or canceled by user, it is an empty.  
    /// 
    /// For images and videos, consider using [`AndroidFs::show_open_visual_media_dialog`] instead.  
    /// 
    /// # Issue
    /// **Dialog has an issue. Details and resolution are following.**  
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/issues/1>
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/README.md>
    /// 
    /// # Note
    /// `mime_types` represents the types of files that should be selected. 
    /// However, there is no guarantee that the returned file will match the specified types.    
    /// If this is empty, all file types will be available for selection. 
    /// This is equivalent to `["*/*"]`, and it will invoke the standard file picker in most cases.  
    /// 
    /// By default, returned uri is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// # Support
    /// All Android version.
    fn show_open_file_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>>;

    /// Opens a dialog for media file selection, such as images and videos.  
    /// This returns a **read-only** uris. If no file is selected or canceled by user, it is an empty.  
    /// 
    /// This is more user-friendly than [`AndroidFs::show_open_file_dialog`].  
    ///
    /// # Issue
    /// **Dialog has an issue. Details and resolution are following.**  
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/issues/1>
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/README.md>
    /// 
    /// # Note
    /// By default, returned uri is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].  
    /// 
    /// The file obtained from this function cannot retrieve the correct file name using [`AndroidFs::get_name`].
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
    ) -> crate::Result<Vec<FileUri>>;

    /// Open a dialog for directory selection,
    /// allowing the app to read and write any file in the selected directory and its subdirectories.  
    /// If canceled by the user, return None.    
    /// 
    /// # Issue
    /// **Dialog has an issue. Details and resolution are following.**  
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/issues/1>
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/README.md>
    /// 
    /// # Note
    /// By default, retruned uri is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// If you take permission for a directory, you can recursively obtain it for its descendants.
    /// 
    /// # Support
    /// All Android version.
    fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>>;

    /// Open a dialog for file saving, and return the selected path.  
    /// This returns a **read-write-removeable** uri. If canceled by the user, return None.    
    /// 
    /// When storing media files such as images, videos, and audio, consider using [`AndroidFs::create_file_in_public_location`].  
    /// When a file does not need to be accessed by other applications and users, consider using  [`PrivateStorage::write`].  
    /// These are easier because the destination does not need to be selected in a dialog.  
    /// 
    /// # Issue
    /// **Dialog has an issue. Details and resolution are following.**  
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/issues/1>
    /// - <https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/README.md>
    /// 
    /// # Note
    /// `mime_type` specify the type of the target file to be saved. 
    /// It should be provided whenever possible. If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types.  
    /// 
    /// The file created this way will not be registered in the MediaStore that is used by [`AndroidFs::show_open_visual_media_dialog`] and etc.
    /// Images and videos files can be registered in the gallery by using methods like [`PublicStorage::create_file_in_public_app_dir`] to create them.
    /// 
    /// By default, returned uri is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// Note that if the user selects a file on Google drive, this function returns None. 
    /// This is because it is not possible to write files on Google drive using FileDescriptor.
    /// 
    /// # Support
    /// All Android version.
    fn show_save_file_dialog(
        &self,
        default_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>>;

    /// Verify whether [`AndroidFs::show_open_visual_media_dialog`] is available on a given device.
    /// 
    /// # Support
    /// All Android version.
    fn is_visual_media_dialog_available(&self) -> crate::Result<bool>;

    /// Please use [`PublicStorage::is_audiobooks_dir_available`] insted.
    #[deprecated(note = "Please use PublicStorage::is_audiobooks_dir_available insted.")]
    #[warn(deprecated)]
    fn is_public_audiobooks_dir_available(&self) -> crate::Result<bool> {
        self.public_storage().is_audiobooks_dir_available()
    }

    /// Please use [`PublicStorage::is_recordings_dir_available`] insted.
    #[deprecated(note = "Please use PublicStorage::is_recordings_dir_available insted.")]
    #[warn(deprecated)]
    fn is_public_recordings_dir_available(&self) -> crate::Result<bool> {
        self.public_storage().is_recordings_dir_available()
    }

    fn app_handle(&self) -> &tauri::AppHandle<R>;

    /// File storage API intended for the app’s use only.
    fn private_storage(&self) -> &impl PrivateStorage<R>;

    /// File storage that is available to other applications and users.
    fn public_storage(&self) -> &impl PublicStorage<R>;
}

/// File storage intended for the app’s use only.  
pub trait PublicStorage<R: tauri::Runtime> {

    /// Creates a new empty file in the specified public app directory and returns a **read-write-removable** URI.  
    ///  
    /// If a file with the same name already exists, a sequential number is added to the name.  
    /// Missing subdirectories will be created recursively.  
    ///  
    /// The created file will be registered with the corresponding MediaStore as needed.  
    /// The URI will remain valid only until the app is uninstalled.
    /// 
    /// # Note
    /// `mime_type`  specify the type of the file to be created. 
    /// It should be provided whenever possible. 
    /// If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types. 
    /// When using [`PublicImageDir`], please do not use a `mime_type` other than image types.
    /// This may result in an error.
    /// Similarly, do not use non-corresponding media types for [`PublicVideoDir`] or [`PublicAudioDir`]. 
    /// Only [`PublicGeneralPurposeDir`] allows all mime types.
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
    /// Lower are need runtime request of `WRITE_EXTERNAL_STORAGE`. (This option will be made available in the future)
    ///
    /// [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// Others are available in all Android versions.
    fn create_file_in_public_app_dir(
        &self,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        let config = self.app_handle().config();
        let app_name = config.product_name.as_ref().unwrap_or(&config.identifier).replace('/', " ");
        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let relative_path_with_subdir = format!("{app_name}/{relative_path}");

        self.create_file_in_public_dir(dir, relative_path_with_subdir, mime_type)
    }

    /// Creates a new empty file in the specified public directory and returns a **read-write-removable** URI.  
    ///  
    /// If a file with the same name already exists, a sequential number is added to the name.  
    /// Missing subdirectories will be created recursively.  
    ///  
    /// The created file will be registered with the corresponding MediaStore as needed.  
    /// The uri will remain valid only until the app is uninstalled.
    /// 
    /// # Note
    /// Do not save files directly in the public directory. Please specify a subdirectory in the `relative_path_with_sub_dir`, such as `appName/file.txt` or `appName/2025-2-11/file.txt`. Do not use `file.txt`.
    /// 
    /// `mime_type`  specify the type of the file to be created. 
    /// It should be provided whenever possible. 
    /// If not specified, `application/octet-stream` is used, as generic, unknown, or undefined file types. 
    /// When using [`PublicImageDir`], please do not use a `mime_type` other than image types.
    /// This may result in an error.
    /// Similarly, do not use non-corresponding media types for [`PublicVideoDir`] or [`PublicAudioDir`]. 
    /// Only [`PublicGeneralPurposeDir`] allows all mime types.
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
    /// Lower are need runtime request of `WRITE_EXTERNAL_STORAGE`. (This option will be made available in the future)
    ///
    /// [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// Others are available in all Android versions.
    fn create_file_in_public_dir(
        &self,
        dir: impl Into<PublicDir>,
        relative_path_with_subdir: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri>;

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

    fn app_handle(&self) -> &tauri::AppHandle<R>;
}

/// File storage intended for the app’s use only.  
pub trait PrivateStorage<R: tauri::Runtime> {

    /// Get the absolute path of the specified directory.  
    /// Apps require no permissions to read or write to the returned path, since this path lives in their private storage.  
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
        dir: PrivateDir,
        relative_path: impl AsRef<str>
    ) -> crate::Result<std::path::PathBuf> {

        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let path = self.resolve_path(dir)?.join(relative_path);
        Ok(path)
    }

    fn resolve_uri(&self, dir: PrivateDir) -> crate::Result<FileUri> {
        Ok(FileUri::from(tauri_plugin_fs::FilePath::Path(self.resolve_path(dir)?)))
    }

    fn resolve_uri_with(&self, dir: PrivateDir, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        Ok(FileUri::from(tauri_plugin_fs::FilePath::Path(self.resolve_path_with(dir, relative_path)?)))
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

    /// Creates a new file in read-write mode; error if the file exists. 
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::create_new`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    fn create_new_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        let path = self.resolve_path_with(base_dir, relative_path)?;
        Ok(std::fs::File::create_new(path)?)
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

    fn app_handle(&self) -> &tauri::AppHandle<R>;
}