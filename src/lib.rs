//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

mod models;
mod impls;
mod error;

use std::io::{Read as _, Write as _};

pub use models::*;
pub use error::{Error, Result};
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
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    fn get_name(&self, uri: &FileUri) -> crate::Result<String>;

    /// Query the provider to get mime type.  
    /// If the directory, this returns `None`.  
    /// If the file, this returns no `None`.  
    /// If the file type is unknown or unset, this returns `Some("application/octet-stream")`.  
    ///
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    fn get_mime_type(&self, uri: &FileUri) -> crate::Result<Option<String>>;

    /// Queries the file system to get information about a file, directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Note
    /// This uses [`AndroidFs::open_file`] internally.
    /// 
    /// # Support
    /// All.
    fn get_metadata(&self, uri: &FileUri) -> crate::Result<std::fs::Metadata> {
        let file = self.open_file(uri, FileAccessMode::Read)?;
        Ok(file.metadata()?)
    }

    /// Open a file in the specified mode.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This must have corresponding permissions (read, write, or both) for the specified **mode**.
    /// 
    /// - ***mode*** :  
    /// Indicates how the file is opened and the permissions granted.  
    /// Note that files provided by third-party apps may not support [`FileAccessMode::WriteAppend`]. (ex: Files on GoogleDrive)  
    ///
    /// # Note
    /// This method uses a FileDescriptor internally. 
    /// However, if the target file does not physically exist on the device, such as cloud-based files, 
    /// the write operation using a FileDescriptor may not be reflected properly.
    /// In such cases, consider using [AndroidFs::write_via_kotlin], 
    /// which writes using a standard method, 
    /// or [AndroidFs::write], which automatically falls back to that approach when necessary.
    /// If you specifically need to write using std::fs::File not entire contents, see [AndroidFs::write_via_kotlin_in] or [AndroidFs::copy_via_kotlin].  
    /// 
    /// It seems that the issue does not occur on all cloud storage platforms. At least, files on Google Drive have issues, 
    /// but files on Dropbox can be written to correctly using a FileDescriptor.
    /// If you encounter issues with cloud storage other than Google Drive, please let me know on [Github](https://github.com/aiueo13/tauri-plugin-android-fs/issues/new). 
    /// This information will be used in [AndroidFs::need_write_via_kotlin] used by `AndroidFs::write`.  
    /// 
    /// There are no problems with file reading.
    /// 
    /// # Support
    /// All.
    fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> crate::Result<std::fs::File>;

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.    
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
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
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
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
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    fn write(&self, uri: &FileUri, contents: impl AsRef<[u8]>) -> crate::Result<()> {
        if self.need_write_via_kotlin(uri)? {
            self.write_via_kotlin(uri, contents)?;
            return Ok(())
        }

        let mut file = self.open_file(uri, FileAccessMode::WriteTruncate)?;
        file.write_all(contents.as_ref())?;
        Ok(())
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// Differences from `std::fs::File::write_all` is the process is done on Kotlin side.  
    /// See [`AndroidFs::open_file`] for why this function exists.
    /// 
    /// If [`AndroidFs::write`] is used, it automatically fall back to this by [`AndroidFs::need_write_via_kotlin`], 
    /// so there should be few opportunities to use this.
    /// 
    /// If you want to write using `std::fs::File`, not entire contents, use [`AndroidFs::write_via_kotlin_in`].
    /// 
    /// # Inner process
    /// The contents is written to a temporary file by Rust side 
    /// and then copied to the specified file on Kotlin side by [`AndroidFs::copy_via_kotlin`].  
    /// 
    /// # Support
    /// All.
    fn write_via_kotlin(
        &self, 
        uri: &FileUri,
        contents: impl AsRef<[u8]>
    ) -> crate::Result<()> {

        self.write_via_kotlin_in(uri, |file| file.write_all(contents.as_ref()))
    }

    /// See [`AndroidFs::write_via_kotlin`] for information.  
    /// Use this if you want to write using `std::fs::File`, not entire contents.
    /// 
    /// If you want to retain the file outside the closure, 
    /// you can perform the same operation using [`AndroidFs::copy_via_kotlin`] and [`PrivateStorage`]. 
    /// For details, please refer to the internal implementation of this function.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to write.
    /// 
    /// - **contetns_writer** :  
    /// A closure that accepts a mutable reference to a `std::fs::File`
    /// and performs the actual write operations. Note that this represents a temporary file.
    fn write_via_kotlin_in<T>(
        &self, 
        uri: &FileUri,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<T>
    ) -> crate::Result<T> {

        static TMP_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

        // 一時ファイルの排他アクセスを保証
        let _guard = TMP_FILE_LOCK.lock();

        // 一時ファイルのパスを取得
        let tmp_file_path = self.app_handle()
            .android_fs()
            .private_storage()
            .resolve_path_with(PrivateDir::Cache, "tauri-plugin-android-fs-tempfile-write-via-kotlin")?;

        // 一時ファイルに内容を書き込む
        // エラー処理は一時ファイルを削除するまで保留
        let result = {
            let ref mut file = std::fs::File::create(&tmp_file_path)?;
            contents_writer(file)
        };

        // 上記処理が成功していれば、一時ファイルの内容をkotlin側で指定されたファイルにコピーする
        // エラー処理は一時ファイルを削除するまで保留
        let result = result
            .map_err(crate::Error::from)
            .and_then(|t| self.copy_via_kotlin(&(&tmp_file_path).into(), uri).map(|_| t));

        // 一時ファイルを削除
        let _ = std::fs::remove_file(&tmp_file_path);

        result
    }

    /// Determines if the file needs to be written via Kotlin side instead of Rust side.  
    /// Currently, this returns true only if the file is on GoogleDrive.  
    /// 
    /// # Support
    /// All.
    fn need_write_via_kotlin(&self, uri: &FileUri) -> crate::Result<bool> {
        Ok(uri.uri.starts_with("content://com.google.android.apps.docs.storage"))
    }

    /// Copies the contents of src file to dest. 
    /// 
    /// This copy process is done on Kotlin side, not on Rust.  
    /// Large files in GB units are also supported.  
    ///  
    /// See [`AndroidFs::write_via_kotlin`] for why this function exists.
    /// 
    /// # Args
    /// - ***src*** :  
    /// The URI of source file.   
    /// This needs to be **readable**.
    /// 
    /// - ***dest*** :  
    /// The URI of destination file.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    fn copy_via_kotlin(&self, src: &FileUri, dest: &FileUri) -> crate::Result<()>;

    /// Remove the file.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **writable**, at least. But even if it is, 
    /// removing may not be possible in some cases. 
    /// For details, refer to the documentation of the function that provided the URI.
    /// 
    /// # Support
    /// All.
    fn remove_file(&self, uri: &FileUri) -> crate::Result<()>;

    /// Remove the **empty** directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    fn remove_dir(&self, uri: &FileUri) -> crate::Result<()>;

    /// Creates a new empty file in the specified location and returns a URI.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`AndroidFs::show_open_dir_dialog`]) 
    ///  
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// This needs to be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// If a file with the same name already exists, a sequential number will be appended to ensure uniqueness.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.  
    ///  
    /// # Support
    /// All.
    fn create_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri>;

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`AndroidFs::show_open_dir_dialog`])  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// This needs to be **readable**.
    ///  
    /// # Note  
    /// The returned type is an iterator because of the data formatting and the file system call is not executed lazily. 
    /// Thus, for directories with thousands or tens of thousands of elements, it may take several seconds.  
    /// 
    /// # Support
    /// All.
    fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>>;

    /// Opens a system file picker and returns a **read-write** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface, 
    /// and also allows file selection from part of third-party apps or cloud storage.
    ///
    /// Removing the returned files is also supported in most cases, 
    /// but note that files provided by third-party apps may not be removable.  
    ///  
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    fn show_open_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>>;

    /// Opens a file picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// Returned URI is valid until the app is terminated. Can not persist it.
    /// 
    /// This works differently depending on the model and version.  
    /// But recent devices often have the similar behaviour as [`AndroidFs::show_open_visual_media_dialog`] or [`AndroidFs::show_open_file_dialog`].  
    /// Use this, if you want your app to simply read/import data.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    fn show_open_content_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>>;

    /// Opens a media picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    ///  
    /// This media picker provides a browsable interface that presents the user with their media library, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    ///  
    /// # Note
    /// The file obtained from this function cannot retrieve the correct file name using [`AndroidFs::get_name`].  
    /// Instead, it will be assigned a sequential number, such as `1000091523.png`. 
    /// And this is marked intended behavior, not a bug.
    /// - <https://issuetracker.google.com/issues/268079113>  
    ///  
    /// # Support
    /// This feature is available on devices that meet the following criteria:  
    /// - Running Android 11 (API level 30) or higher  
    /// - Receive changes to Modular System Components through Google System Updates  
    ///  
    /// Availability on a given device can be verified by calling [`AndroidFs::is_visual_media_dialog_available`].  
    /// If not supported, this function behaves the same as [`AndroidFs::show_open_file_dialog`].  
    /// 
    /// # References
    /// <https://developer.android.com/training/data-storage/shared/photopicker>
    fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>>;

    /// Opens a system directory picker, allowing the creation of a new directory or the selection of an existing one, 
    /// and returns a **read-write** directory URI. 
    /// App can fully manage entries within the returned directory.  
    /// If no directory is selected or the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface.
    /// 
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT_TREE>
    fn show_manage_dir_dialog(
        &self,
        initial_location: Option<&FileUri>,
    ) -> crate::Result<Option<FileUri>>;

    /// Please use [`AndroidFs::show_manage_dir_dialog`] instead.
    #[deprecated = "Confusing name. Please use show_manage_dir_dialog instead."]
    #[warn(deprecated)]
    fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>> {
        self.show_manage_dir_dialog(None)
    }

    /// Opens a dialog to save a file and returns a **writeonly** URI.  
    /// The returned file may be a newly created file with no content,
    /// or it may be an existing file with the requested MIME type.  
    /// If the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface, 
    /// and also allows file selection from part of third-party apps or cloud storage.
    /// 
    /// Removing and reading the returned files is also supported in most cases, 
    /// but note that files provided by third-party apps may not.  
    ///  
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// 
    /// - ***initial_file_name*** :  
    /// An initial file name, but the user may change this value before creating the file.  
    /// 
    /// - ***mime_type*** :  
    /// The MIME type of the file to be saved.  
    /// If this is None, MIME type is inferred from the extension of ***initial_file_name*** (not file name by user input)
    /// and if that fails, `application/octet-stream` is used.  
    ///  
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_CREATE_DOCUMENT>
    fn show_save_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>>;

    /// Opens a dialog for sharing file to other apps.  
    /// 
    /// An error will occur if there is no app that can handle the request. 
    /// Please use [`AndroidFs::can_share_file`] to confirm.
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri to share.  
    /// This needs to be **readable**.  
    /// This given from [`PrivateStorage`] or [`AndroidFs::show_open_visual_media_dialog`] ***cannot*** be used.
    /// 
    /// # Support
    /// All.
    fn show_share_file_dialog(&self, uri: &FileUri) -> crate::Result<()>;

    /// Opens a dialog for viewing file on other apps.  
    /// This performs the general "open file" action.
    /// 
    /// An error will occur if there is no app that can handle the request. 
    /// Please use [`AndroidFs::can_view_file`] to confirm.
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri to view.  
    /// This needs to be **readable**.  
    /// This given from [`PrivateStorage`] or [`AndroidFs::show_open_visual_media_dialog`] ***cannot*** be used.
    /// 
    /// # Support
    /// All.
    fn show_view_file_dialog(&self, uri: &FileUri) -> crate::Result<()>;

    /// Determines whether the specified file can be used with [`AndroidFs::show_share_file_dialog`].
    /// # Args
    /// - **uri** :  
    /// Target file uri.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    fn can_share_file(&self, uri: &FileUri) -> crate::Result<bool>;

    /// Determines whether the specified file can be used with [`AndroidFs::show_view_file_dialog`].
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    fn can_view_file(&self, uri: &FileUri) -> crate::Result<bool>;

    /// Take persistent permission to access the file, directory and its descendants.  
    /// This is a prolongation of an already acquired permission, not the acquisition of a new one.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.
    /// 
    /// Note that [there is a limit to the total number of URI that can be made persistent by this function.](https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621)  
    /// Therefore, it is recommended to relinquish the unnecessary persisted URI by [`AndroidFs::release_persisted_uri_permission`] or [`AndroidFs::release_all_persisted_uri_permissions`].  
    /// Persisted permissions may be relinquished by other apps, user, or by moving/removing entries.
    /// So check by [`AndroidFs::check_persisted_uri_permission`].  
    /// And you can retrieve the list of persisted uris using [`AndroidFs::get_all_persisted_uri_permissions`].
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory. This must be a URI taken from following :  
    ///     - [`AndroidFs::show_open_file_dialog`]
    ///     - [`AndroidFs::show_open_visual_media_dialog`]
    ///     - [`AndroidFs::show_save_file_dialog`]
    ///     - [`AndroidFs::show_manage_dir_dialog`]  
    ///     - [`AndroidFs::read_dir`] :  
    ///         If this, the permissions of the origin directory URI is persisted, not a entry iteself. 
    ///         Because the permissions and validity period of the entry URIs depend on the origin directory.
    /// 
    /// # Support
    /// All. 
    fn take_persistable_uri_permission(&self, uri: &FileUri) -> crate::Result<()>;

    /// Check a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].  
    /// Returns false if there are only non-persistent permissions or no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    ///
    /// - **mode** :  
    /// The mode of permission you want to check.  
    /// 
    /// # Support
    /// All.
    fn check_persisted_uri_permission(&self, uri: &FileUri, mode: PersistableAccessMode) -> crate::Result<bool>;

    /// Return list of all persisted URIs that have been persisted by [`AndroidFs::take_persistable_uri_permission`] and currently valid.   
    /// 
    /// # Support
    /// All.
    fn get_all_persisted_uri_permissions(&self) -> crate::Result<impl Iterator<Item = PersistedUriPermission>>;

    /// Relinquish a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].   
    /// 
    /// # Args
    /// - ***uri*** :  
    /// URI of the target file or directory.  
    ///
    /// # Support
    /// All.
    fn release_persisted_uri_permission(&self, uri: &FileUri) -> crate::Result<()>;

    /// Relinquish a all persisted uri permission grants by [`AndroidFs::take_persistable_uri_permission`].  
    /// 
    /// # Support
    /// All.
    fn release_all_persisted_uri_permissions(&self) -> crate::Result<()>;

    /// Verify whether [`AndroidFs::show_open_visual_media_dialog`] is available on a given device.
    /// 
    /// # Support
    /// All.
    fn is_visual_media_dialog_available(&self) -> crate::Result<bool>;

    /// File storage intended for the app’s use only.
    fn private_storage(&self) -> &impl PrivateStorage<R>;

    /// File storage that is available to other applications and users.
    fn public_storage(&self) -> &impl PublicStorage<R>;

    fn app_handle(&self) -> &tauri::AppHandle<R>;
}

/// File storage that is available to other applications and users.
pub trait PublicStorage<R: tauri::Runtime> {

    /// See [`PublicStorage::create_file_in_public_dir`] for description.  
    /// 
    /// This is the same as following: 
    /// ``````ignore
    /// create_file_in_public_dir(
    ///     dir,
    ///     format!("{app_name}/{relative_path}"),
    ///     mime_type
    /// );
    /// ``````
    fn create_file_in_public_app_dir(
        &self,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        let config = self.app_handle().config();
        let app_name = config.product_name.as_deref().unwrap_or("");
        let app_name = match app_name.is_empty() {
            true => &config.identifier,
            false => app_name
        };
        let app_name = app_name.replace('/', " ");
        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let relative_path_with_subdir = format!("{app_name}/{relative_path}");

        self.create_file_in_public_dir(dir, relative_path_with_subdir, mime_type)
    }

    /// Creates a new empty file in the specified public directory
    /// and returns a **persistent read-write** URI.  
    ///  
    /// The created file has following features :   
    /// - Will be registered with the corresponding MediaStore as needed.  
    /// - Always supports remove.
    /// - Not removed when the app is uninstalled.
    /// 
    /// # Args
    /// - ***dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
    ///  
    /// - ***relative_path_with_subdir*** :  
    /// The file path relative to the base directory.  
    /// If a file with the same name already exists, a sequential number will be appended to ensure uniqueness.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// Please specify a subdirectory in this, 
    /// such as `MyApp/file.txt` or `MyApp/2025-2-11/file.txt`. Do not use `file.txt`.  
    /// As shown above, it is customary to specify the app name at the beginning of the subdirectory, 
    /// and in this case, using [`PublicStorage::create_file_in_public_app_dir`] is recommended.
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path_with_subdir***
    /// and if that fails, `application/octet-stream` is used.  
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
    /// All.
    fn is_audiobooks_dir_available(&self) -> crate::Result<bool>;

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.
    /// 
    /// # Support
    /// All.
    fn is_recordings_dir_available(&self) -> crate::Result<bool>;

    fn app_handle(&self) -> &tauri::AppHandle<R>;
}

/// File storage intended for the app’s use only.  
pub trait PrivateStorage<R: tauri::Runtime> {

    /// Get the absolute path of the specified directory.  
    /// App can fully manage entries within this directory without any permission via std::fs.   
    ///
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s initialising request.  
    /// When using [`PrivateDir::Cache`], the system will automatically delete files in this directory as disk space is needed elsewhere on the device.   
    /// 
    /// The returned path may change over time if the calling app is moved to an adopted storage device, 
    /// so only relative paths should be persisted.   
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
    /// All.
    fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf>;

    /// Get the absolute path of the specified relative path and base directory.  
    /// App can fully manage entries of this path without any permission via std::fs.   
    ///
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All.
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
        self.resolve_path(dir).map(Into::into)
    }

    fn resolve_uri_with(&self, dir: PrivateDir, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        self.resolve_path_with(dir, relative_path).map(Into::into)
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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
    /// All.
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