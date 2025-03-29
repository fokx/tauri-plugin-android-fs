use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::time::{UNIX_EPOCH, Duration};
use tauri::{plugin::{PluginApi, PluginHandle}, AppHandle, Runtime};
use crate::{models::*, AndroidFs, AndroidFsExt, PrivateStorage, PublicStorage};


pub struct AndroidFsImpl<R: Runtime> {
    api: PluginHandle<R>, 
    app: AppHandle<R>, 
    intent_lock: std::sync::Mutex<()>
}

impl<R: Runtime> AndroidFsImpl<R> {

    pub fn new<C: DeserializeOwned>(
        app: &AppHandle<R>,
        api: PluginApi<R, C>,
    ) -> crate::Result<impl AndroidFs<R>> {

        Ok(Self {
            api: api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?, 
            app: app.clone(),
            intent_lock: std::sync::Mutex::new(())
        })
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

impl<R: Runtime> AndroidFs<R> for AndroidFsImpl<R> {

    fn get_name(&self, uri: &FileUri) -> crate::Result<String> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Res { name: String });

        let uri = uri.clone();

        self.api
            .run_mobile_plugin::<Res>("getName", Req { uri })
            .map(|v| v.name)
            .map_err(Into::into)
    }

    fn get_mime_type(&self, uri: &FileUri) -> crate::Result<Option<String>> {
        impl_serde!(struct Req { uri: FileUri});
        impl_serde!(struct Res { value: Option<String> });

        let uri = uri.clone();

        self.api
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
            FileAccessMode::WriteTruncate => "wt",
            FileAccessMode::WriteAppend => "wa",
            FileAccessMode::ReadWriteTruncate => "rwt",
            FileAccessMode::ReadWrite => "rw",
        };

        self.api
            .run_mobile_plugin::<Res>("getFileDescriptor", Req { uri, mode })
            .map(|v| {
                use std::os::fd::FromRawFd;
                unsafe { std::fs::File::from_raw_fd(v.fd) }
            })
            .map_err(Into::into)
    }

    fn show_open_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
        multiple: bool,
        take_persistable_uri_permission: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        impl_serde!(struct Req { mime_types: Vec<String>, multiple: bool, take_persistable_uri_permission: bool, initial_location: Option<FileUri> });
        impl_serde!(struct Res { uris: Vec<FileUri> });
    
        let initial_location = initial_location.map(Clone::clone);
        let mime_types = mime_types.iter().map(|s| s.to_string()).collect();

        let _guard = self.intent_lock.lock();
        self.api
            .run_mobile_plugin::<Res>("showOpenFileDialog", Req { mime_types, multiple, take_persistable_uri_permission, initial_location })
            .map(|v| v.uris)
            .map_err(Into::into)
    }

    fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool,
        take_persistable_uri_permission: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        impl_serde!(struct Req { multiple: bool, target: VisualMediaTarget, take_persistable_uri_permission: bool });
        impl_serde!(struct Res { uris: Vec<FileUri> });
    
        let _guard = self.intent_lock.lock();
        self.api
            .run_mobile_plugin::<Res>("showOpenVisualMediaDialog", Req { multiple, target, take_persistable_uri_permission })
            .map(|v| v.uris)
            .map_err(Into::into)
    }

    fn show_save_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
        take_persistable_uri_permission: bool
    ) -> crate::Result<Option<FileUri>> {

        impl_serde!(struct Req<'a> { initial_file_name: &'a str, mime_type: &'a str, take_persistable_uri_permission: bool, initial_location: Option<FileUri> });
        impl_serde!(struct Res { uri: Option<FileUri> });

        let initial_location = initial_location.map(Clone::clone);
        let initial_file_name = initial_file_name.as_ref();
        let mime_type = mime_type.as_ref().map(|s| s.as_ref()).unwrap_or("application/octet-stream");
    
        let _guard = self.intent_lock.lock();
        self.api
            .run_mobile_plugin::<Res>("showSaveFileDialog", Req { initial_file_name, mime_type, take_persistable_uri_permission, initial_location })
            .map(|v| v.uri)
            .map_err(Into::into)
    }

    fn show_manage_dir_dialog(
        &self,
        initial_location: Option<&FileUri>,
        take_persistable_uri_permission: bool
    ) -> crate::Result<Option<FileUri>> {
        
        impl_serde!(struct Req { take_persistable_uri_permission: bool, initial_location: Option<FileUri> });
        impl_serde!(struct Res { uri: Option<FileUri> });
    
        let initial_location = initial_location.map(Clone::clone);

        let _guard = self.intent_lock.lock();
        self.api
            .run_mobile_plugin::<Res>("showManageDirDialog", Req { take_persistable_uri_permission, initial_location })
            .map(|v| v.uri)
            .map_err(Into::into)
    }
    
    fn remove_file(&self, uri: &FileUri) -> crate::Result<()> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Res;);

        let uri = uri.clone();
    
        self.api
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

        self.api
            .run_mobile_plugin::<FileUri>("createFile", Req { dir, mime_type, relative_path })
            .map_err(Into::into)
    }
    
    fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>> {
        impl_serde!(struct Req { uri: FileUri });
        impl_serde!(struct Obj { name: String, uri: FileUri, last_modified: i64, byte_size: i64, mime_type: Option<String> });
        impl_serde!(struct Res { entries: Vec<Obj> });

        let uri = uri.clone();
    
        self.api
            .run_mobile_plugin::<Res>("readDir", Req { uri })
            .map(|v| v.entries.into_iter())
            .map(|v| v.map(|v| match v.mime_type {
                Some(mime_type) => Entry::File {
                    name: v.name,
                    last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                    len: v.byte_size as u64,
                    mime_type,
                    uri: v.uri,
                },
                None => Entry::Dir {
                    name: v.name,
                    last_modified: UNIX_EPOCH + Duration::from_millis(v.last_modified as u64),
                    uri: v.uri,
                }
            }))
            .map_err(Into::into)
    }

    fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.api
            .run_mobile_plugin::<Res>("isVisualMediaDialogAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn private_storage(&self) -> &impl PrivateStorage<R> {
        self
    }

    fn public_storage(&self) -> &impl PublicStorage<R> {
        self
    }

    fn app_handle(&self) -> &tauri::AppHandle<R> {
        &self.app
    }
}

impl<R: Runtime> PublicStorage<R> for AndroidFsImpl<R> {

    fn create_file_in_public_dir(
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

        let (dir_name, dir_parent_uri) = self.api
            .run_mobile_plugin::<Res>("getPublicDirInfo", Req { dir, dir_type })
            .map(|v| (v.name, v.uri))?;
        
        let relative_path = relative_path.as_ref().trim_start_matches('/');
        let relative_path = format!("{dir_name}/{relative_path}");

        let dir_parent_uri = FileUri {
            uri: dir_parent_uri,
            document_top_tree_uri: None
        };

        PublicStorage::app_handle(self)
            .android_fs()
            .create_file(&dir_parent_uri, relative_path, mime_type)
    }

    fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.api
            .run_mobile_plugin::<Res>("isAudiobooksDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn is_recordings_dir_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.api
            .run_mobile_plugin::<Res>("isRecordingsDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
	}

    fn app_handle(&self) -> &tauri::AppHandle<R> {
        &self.app
    }
}

impl<R: Runtime> PrivateStorage<R> for AndroidFsImpl<R> {

    fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        impl_serde!(struct Paths { data: String, cache: String });
        
        static PATHS: std::sync::OnceLock<Paths> = std::sync::OnceLock::new();

        if PATHS.get().is_none() {
            let paths = self.api
                .run_mobile_plugin::<Paths>("getPrivateBaseDirAbsolutePaths", "")?;

            let _ = PATHS.set(paths);
        }

        let paths = PATHS.get().unwrap();

        Ok(match dir {
            PrivateDir::Data => std::path::PathBuf::from(paths.data.to_owned()),
            PrivateDir::Cache => std::path::PathBuf::from(paths.cache.to_owned()),
        })
    }

    fn app_handle(&self) -> &tauri::AppHandle<R> {
        &self.app
    }
}