use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::{plugin::{PluginApi, PluginHandle}, AppHandle, Runtime};
use crate::{models::*, PathError, AndroidFs, PrivateStorage};


pub struct AndroidFsImpl<R: Runtime>(PluginHandle<R>, AppHandle<R>);

impl<R: Runtime> AndroidFsImpl<R> {

    pub fn new<C: DeserializeOwned>(
        app: &AppHandle<R>,
        api: PluginApi<R, C>,
    ) -> crate::Result<impl AndroidFs> {

        Ok(Self(api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?, app.clone()))
    }
}


macro_rules! impl_serde {
    (struct $struct_ident:ident $(< $lifetime:lifetime >)? { $( $name:ident: $ty:ty ),* $(,)? }) => {
        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident $(< $lifetime >)? {
            $($name: $ty,)*
        }
    };
    (struct $struct_ident:ident $(;)?) => {
        #[derive(Serialize, Deserialize)]
        struct $struct_ident;
    };
}

impl<R: Runtime> AndroidFs for AndroidFsImpl<R> {

    fn get_name(&self, uri: &FileUri) -> crate::Result<String> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Res { name: String });

        let uri = uri.clone();

        self.0  
            .run_mobile_plugin::<Res>("getName", Req { uri })
            .map(|v| v.name)
            .map_err(Into::into)
    }

    fn get_mime_type(&self, uri: &FileUri) -> crate::Result<Option<String>> {
        impl_serde!(struct Req { uri: FileUri});
        impl_serde!(struct Res { value: Option<String> });

        let uri = uri.clone();

        self.0  
            .run_mobile_plugin::<Res>("getMimeType", Req { uri })
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> crate::Result<std::fs::File> {
        impl_serde!(struct Req<'a> { uri: FileUri, mode: &'a str });
        impl_serde!(struct Res { fd: std::os::fd::RawFd });
    
        let uri = uri.clone();
        let mode = match mode {
            FileAccessMode::Read => "r",
            FileAccessMode::Write => "w",
            FileAccessMode::WriteAppend => "wa",
            FileAccessMode::ReadWriteTruncate => "rwt",
            FileAccessMode::ReadWrite => "rw",
        };

        self.0  
            .run_mobile_plugin::<Res>("getFileDescriptor", Req { uri, mode })
            .map(|v| {
                use std::os::fd::FromRawFd;
                unsafe { std::fs::File::from_raw_fd(v.fd) }
            })
            .map_err(Into::into)
    }

    fn show_open_file_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        impl_serde!(struct Req { mime_types: Vec<String>, multiple: bool });
        impl_serde!(struct Res { uris: Vec<FileUri> });
    
        let mime_types = mime_types.iter().map(|s| s.to_string()).collect();

        self.0  
            .run_mobile_plugin::<Res>("showOpenFileDialog", Req { mime_types, multiple })
            .map(|v| v.uris)
            .map_err(Into::into)
    }

    fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        impl_serde!(struct Req { multiple: bool, target: VisualMediaTarget });
        impl_serde!(struct Res { uris: Vec<FileUri> });
    
        self.0  
            .run_mobile_plugin::<Res>("showOpenVisualMediaDialog", Req { multiple, target })
            .map(|v| v.uris)
            .map_err(Into::into)
    }

    fn show_save_file_dialog(
        &self,
        default_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>> {

        impl_serde!(struct Req<'a> { default_file_name: &'a str, mime_type: &'a str });
        impl_serde!(struct Res { uri: Option<FileUri> });

        let default_file_name = default_file_name.as_ref();
        let mime_type = mime_type.as_ref().map(|s| s.as_ref()).unwrap_or("application/octet-stream");
    
        self.0  
            .run_mobile_plugin::<Res>("showSaveFileDialog", Req { default_file_name, mime_type })
            .map(|v| v.uri)
            .map_err(Into::into)
    }

    fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>> {
        impl_serde!(struct Res { uri: Option<FileUri> });
    
        self.0  
            .run_mobile_plugin::<Res>("showOpenDirDialog", "")
            .map(|v| v.uri)
            .map_err(Into::into)
    }
    
    fn remove_file(&self, uri: &FileUri) -> crate::Result<()> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Res;);

        let uri = uri.clone();
    
        self.0  
            .run_mobile_plugin::<Res>("delete", Req { uri })
            .map(|_| ())
            .map_err(Into::into)
    }

    fn remove_dir(&self, uri: &FileUri) -> crate::Result<()> {
        AndroidFs::remove_file(self, uri)
    }
    
    fn create_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        impl_serde!(struct Req<'a> { dir: FileUri, mime_type: &'a str, relative_path: &'a str });
        
        let relative_path = relative_path.as_ref();
        let mime_type = mime_type.unwrap_or("application/octet-stream");
        let dir = dir.clone();

        self.0  
            .run_mobile_plugin::<FileUri>("createFile", Req { dir, mime_type, relative_path })
            .map_err(Into::into)
    }
    
    fn create_file_in_public_location(
        &self,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        impl_serde!(struct Req<'a> { dir: PublicDir, dir_type: &'a str });
        impl_serde!(struct Res { name: String, uri: String });

        let dir = dir.into();
        let (_, dir_type) = match dir {
            PublicDir::Image(_) => (mime_type.unwrap_or("image/*"), "Image"),
            PublicDir::Video(_) => (mime_type.unwrap_or("video/*"), "Video"),
            PublicDir::Audio(_) => (mime_type.unwrap_or("audio/*"), "Audio"),
            PublicDir::GeneralPurpose(_) => (mime_type.unwrap_or("application/octet-stream"), "GeneralPurpose"),
        };

        let (dir_name, dir_parent_uri) = self.0  
            .run_mobile_plugin::<Res>("getPublicDirInfo", Req { dir, dir_type })
            .map(|v| (v.name, v.uri))?;
        
        let app = self.1.config();
        let app_name = app.product_name.as_ref().unwrap_or(&app.identifier);
        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let relative_path = format!("{dir_name}/{app_name}/{relative_path}");

        let dir_parent_uri = FileUri {
            uri: dir_parent_uri,
            document_top_tree_uri: None
        };

        AndroidFs::create_file(self, &dir_parent_uri, relative_path, mime_type)
    }
    
    fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Obj { name: String, uri: FileUri, last_modified: i64, byte_size: i64, mime_type: Option<String> });
        impl_serde!(struct Res { entries: Vec<Obj> });

        let uri = uri.clone();
    
        self.0  
            .run_mobile_plugin::<Res>("readDir", Req { uri })
            .map(|v| 
                v.entries
                    .into_iter()
                    .map(|v| match v.mime_type {
                        Some(mime_type) => Entry::File {
                            name: v.name,
                            last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                            byte_size: v.byte_size as u64,
                            mime_type,
                            uri: v.uri,
                        },
                        None => Entry::Dir {
                            name: v.name,
                            last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                            uri: v.uri,
                        }
                    })
            )
            .map_err(Into::into)
    }
    
    fn take_persistable_uri_permission(&self, uri: FileUri, mode: PersistableAccessMode) -> crate::Result<()> {
        impl_serde!(struct Req { uri: FileUri, mode: PersistableAccessMode });
        impl_serde!(struct Res;);

        let uri = uri.clone();

        self.0  
            .run_mobile_plugin::<Res>("takePersistableUriPermission", Req { uri, mode })
            .map(|_| ())
            .map_err(Into::into)
    }
    
    fn is_public_audiobooks_dir_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isAudiobooksDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn is_public_recordings_dir_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isRecordingsDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
	}

    fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isVisualMediaDialogAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn private_storage(&self) -> &impl PrivateStorage {
        self
    }
}

impl<R: Runtime> PrivateStorage for AndroidFsImpl<R> {

    fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        impl_serde!(struct Paths { data: String, cache: String });
        
        static PATHS: std::sync::OnceLock<Paths> = std::sync::OnceLock::new();

        if PATHS.get().is_none() {
            let paths = self.0  
                .run_mobile_plugin::<Paths>("getPrivateBaseDirAbsolutePaths", "")?;

            let _ = PATHS.set(paths);
        }

        let paths = PATHS.get().unwrap();

        Ok(match dir {
            PrivateDir::Data => std::path::PathBuf::from(paths.data.to_owned()),
            PrivateDir::Cache => std::path::PathBuf::from(paths.cache.to_owned()),
        })
    }
}

/// convert "dir1/dir2/dir3/file.txt" to ("dir1/dir2/dir3", "file.txt")
/// convert "/dir/file.txt" to ("dir", "file.txt")
/// 
/// error when "": empty
/// error when "dir1/": no file name
/// error when "file.txt": no sub dir
/// error when "dir1//file.txt": illegal
fn split_relative_path(relative_path: &str) -> crate::Result<(&str, &str)> {
    // "dir/file.txt"も"/dir/file.txt"も同じように扱う
    let relative_path = relative_path.trim_start_matches('/');

    if relative_path.is_empty() {
        return Err(PathError::Empty.into());
    }
    if relative_path.ends_with('/') {
        return Err(PathError::DoesNotContainFileName.into());
    }
    if !relative_path.contains('/') {
        return Err(PathError::DoesNotContainSubDir.into());
    }
    if relative_path.chars()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w[0] == '/' && w[1] == '/') {

        return Err(PathError::ConsecutiveSeparator.into());
    }

    let mut iter = relative_path.rsplitn(2, '/');
    let file_name = iter.next().ok_or(PathError::DoesNotContainFileName)?;
    let sub_dir = iter.next().ok_or(PathError::DoesNotContainSubDir)?;
    Ok((sub_dir, file_name))
}




/*


impl<R: Runtime> AndroidFs for AndroidFsImpl<R> {


    fn new_file(&self, base_dir: &DirPath, relative_path: impl AsRef<str>, mime_type: Option<&str>) -> crate::Result<FilePath> {
        impl_serde!(struct Req<'a> { path: DirPath, relative_path: &'a str, mime_type: &'a str });
        impl_serde!(struct Res { path: FilePath });

        let relative_path = relative_path.as_ref().trim_start_matches('/');
        if relative_path.is_empty() {
            return Err(PathError::Empty.into());
        }
        if relative_path.ends_with('/') {
            return Err(PathError::DoesNotContainFileName.into());
        }
        if relative_path.chars()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| w[0] == '/' && w[1] == '/') {
    
            return Err(PathError::ConsecutiveSeparator.into());
        }

        let mime_type = mime_type.as_ref().map(|s| s.as_ref()).unwrap_or("application/octet-stream");
        let path = base_dir.clone();
    
        self.0  
            .run_mobile_plugin::<Res>("createFileInDir", Req { path, relative_path, mime_type })
            .map(|v| v.path)
            .map_err(Into::into)
    }

    fn read_dir(&self, path: &DirPath) -> crate::Result<Vec<Entry>> {
        impl_serde!(struct Req { path: DirPath });
        impl_serde!(struct Obj { name: String, path: EntryPath, last_modified: i64, byte_size: i64, mime_type: String });
        impl_serde!(struct Res { entries: Vec<Obj> });

        let path = path.clone();
    
        self.0  
            .run_mobile_plugin::<Res>("readDir", Req { path })
            .map(|v| 
                v.entries
                    .into_iter()
                    .map(|v| match v.path {
                        EntryPath::File(path) => Entry::File {
                            name: v.name,
                            last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                            byte_size: v.byte_size as u64,
                            mime_type: v.mime_type,
                            path,
                        },
                        EntryPath::Dir(path) => Entry::Dir {
                            name: v.name,
                            last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                            path,
                        }
                    })
                    .collect()
            )
            .map_err(Into::into)
    }
}


impl<R: Runtime> PublicStorage for AndroidFsImpl<R> {

    fn write_with_contents_writer(
        &self,
        base_dir: PublicGeneralPurposeDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
        
        let (sub_dir, file_name) = split_relative_path(relative_path_with_sub_dir.as_ref())?;

        self.save_public_file(
            "GeneralPurpose",
            contents_writer,
            mime_type, 
            PublicDir::GeneralPurpose(base_dir), 
            sub_dir,
            file_name
        )
    }

    fn write_image_with_contents_writer(
        &self,
        base_dir: PublicImageDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
        
        let (sub_dir, file_name) = split_relative_path(relative_path_with_sub_dir.as_ref())?;

        self.save_public_file(
            "Image",
            contents_writer,
            mime_type, 
            PublicDir::Image(base_dir), 
            sub_dir,
            file_name
        )
    }

    fn write_video_with_contents_writer(
        &self,
        base_dir: PublicVideoDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
        
        let (sub_dir, file_name) = split_relative_path(relative_path_with_sub_dir.as_ref())?;

        self.save_public_file(
            "Video",
            contents_writer,
            mime_type, 
            PublicDir::Video(base_dir), 
            sub_dir,
            file_name
        )
    }

    fn write_audio_with_contents_writer(
        &self,
        base_dir: PublicAudioDir,
        relative_path_with_sub_dir: impl AsRef<str>,
        mime_type: Option<&str>,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
        
        let (sub_dir, file_name) = split_relative_path(relative_path_with_sub_dir.as_ref())?;

        self.save_public_file(
            "Audio",
            contents_writer,
            mime_type, 
            PublicDir::Audio(base_dir), 
            sub_dir,
            file_name
        )
    }
}

impl<R: Runtime> AndroidFsImpl<R> {

    fn open_file_with_mode(&self, path: &FilePath, mode: &str) -> crate::Result<std::fs::File> {
        impl_serde!(struct Req<'a> { path: String, mode: &'a str });
        impl_serde!(struct Res { fd: std::os::fd::RawFd });
    
        let path = crate::convert_file_path_to_string(path);

        self.0  
            .run_mobile_plugin::<Res>("getFileDescriptor", Req { path, mode })
            .map(|v| {
                use std::os::fd::FromRawFd;
                unsafe { std::fs::File::from_raw_fd(v.fd) }
            })
            .map_err(Into::into)
    }

    fn save_public_file(
        &self,
        file_type: &str,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>,
        mime_type: Option<&str>,
        base_dir: PublicDir,
        sub_dir: &str,
        file_name: &str,
    ) -> crate::Result<FilePath> {

        let (path, mut file) = self.save_public_file_before_write(
            file_type,
            mime_type, 
            base_dir, 
            sub_dir, 
            file_name
        )?;

        match contents_writer(&mut file) {
            Ok(_) => {
                std::mem::drop(file);
                self.save_public_file_after_succeed_write(&path)?;
                Ok(path)
            },
            Err(e) => {
                std::mem::drop(file);
                self.save_public_file_after_failed_write(&path)?;
                Err(crate::Error::Io(e))
            },
        }
    }

    fn save_public_file_before_write(
        &self,
        file_type: &str,
        mime_type: Option<&str>,
        base_dir: PublicDir,
        sub_dir: &str,
        file_name: &str,
    ) -> crate::Result<(FilePath, std::fs::File)> {
        
        impl_serde!(
            struct Req<'a> {
                file_type: &'a str,
                mime_type: Option<&'a str>,
                base_dir: PublicDir,
                sub_dir: &'a str,
                file_name: &'a str
            }
        );
        impl_serde!(struct Res { fd: std::os::fd::RawFd, path: FilePath });

        self.0  
            .run_mobile_plugin::<Res>("savePublicFileBeforeWrite", Req { file_type, mime_type, file_name, base_dir, sub_dir })
            .map(|v| {
                use std::os::fd::FromRawFd;
                (v.path, unsafe { std::fs::File::from_raw_fd(v.fd) })
            })
            .map_err(Into::into)
    }

    fn save_public_file_after_failed_write(&self, path: &FilePath) -> crate::Result<()> {
        impl_serde!(struct Req { path: String });
        impl_serde!(struct Res;);
    
        let path = crate::convert_file_path_to_string(path);

        self.0  
            .run_mobile_plugin::<Res>("savePublicFileAfterFailedWrite", Req { path })
            .map(|_| ())
            .map_err(Into::into)
    }

    fn save_public_file_after_succeed_write(&self, path: &FilePath) -> crate::Result<()> {
        impl_serde!(struct Req { path: String });
        impl_serde!(struct Res;);

        let path = crate::convert_file_path_to_string(path);
    
        self.0  
            .run_mobile_plugin::<Res>("savePublicFileAfterSucceedWrite", Req { path })
            .map(|_| ())
            .map_err(Into::into)
    }

    fn take_persistable_permission(&self, path: String, mode: PersistableAccessMode) -> crate::Result<()> {
        impl_serde!(struct Req { path: String, mode: PersistableAccessMode });
        impl_serde!(struct Res;);

        self.0  
            .run_mobile_plugin::<Res>("takePersistableUriPermission", Req { path, mode })
            .map(|_| ())
            .map_err(Into::into)
	}
}
*/