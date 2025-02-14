use std::io::Write;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use tauri::{plugin::{PluginApi, PluginHandle}, AppHandle, Runtime};
use crate::{models::*, PathError, AndroidFs, FilePath, PrivateStorage, PublicStorage};


pub struct AndroidFsImpl<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> AndroidFsImpl<R> {

    pub fn new<C: DeserializeOwned>(
		_app: &AppHandle<R>,
    	api: PluginApi<R, C>,
  	) -> crate::Result<impl AndroidFs> {

        Ok(Self(api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?))
    }
}


macro_rules! impl_serde {
    (struct $struct_ident: ident { $( $name: ident: $ty: ty ),* }) => {
        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident {
            $($name: $ty,)*
        }
    };
    (struct $struct_ident: ident<'a> { $( $name: ident: $ty: ty ),* }) => {
        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident<'a> {
            $($name: $ty,)*
        }
    };
    (struct $struct_ident: ident;) => {
        #[derive(Serialize, Deserialize)]
        struct $struct_ident;
    };
}

impl<R: Runtime> AndroidFs for AndroidFsImpl<R> {

    fn get_file_name(&self, path: &FilePath) -> crate::Result<String> {
        impl_serde!(struct Req { path: String });
        impl_serde!(struct Res { name: String });

        let path = path.to_string();

        self.0  
            .run_mobile_plugin::<Res>("getFileName", Req { path })
            .map(|v| v.name)
            .map_err(Into::into)
    }

    fn get_mime_type(&self, path: &FilePath) -> crate::Result<Option<String>> {
        impl_serde!(struct Req { path: String });
        impl_serde!(struct Res { value: Option<String> });

        let path = path.to_string();

        self.0  
            .run_mobile_plugin::<Res>("getMimeType", Req { path })
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn open_file(&self, path: &FilePath) -> crate::Result<std::fs::File> {
        self.open_file_with_mode(path, "r")
    }

    fn create_file(&self, path: &FilePath) -> crate::Result<std::fs::File> {
        self.open_file_with_mode(path, "w")
    }

    fn show_open_file_dialog(&self, mime_types: &[&str], multiple: bool) -> crate::Result<Vec<FilePath>> {
        impl_serde!(struct Req { mime_types: Vec<String>, multiple: bool });
        impl_serde!(struct Res { paths: Vec<FilePath> });
    
        let mime_types = mime_types.iter().map(|s| s.to_string()).collect();

        self.0  
            .run_mobile_plugin::<Res>("showOpenFileDialog", Req { mime_types, multiple })
            .map(|v| v.paths)
            .map_err(Into::into)
    }

    fn show_open_visual_media_dialog(&self, target: VisualMediaTarget, multiple: bool) -> crate::Result<Vec<FilePath>> {
        impl_serde!(struct Req { multiple: bool, target: VisualMediaTarget });
        impl_serde!(struct Res { paths: Vec<FilePath> });
    
        self.0  
            .run_mobile_plugin::<Res>("showOpenVisualMediaDialog", Req { multiple, target })
            .map(|v| v.paths)
            .map_err(Into::into)
    }

    fn show_save_file_dialog(
        &self,
        default_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FilePath>> {

        impl_serde!(struct Req<'a> { default_file_name: &'a str, mime_type: &'a str });
        impl_serde!(struct Res { path: Option<FilePath> });

        let default_file_name = default_file_name.as_ref();
        let mime_type = mime_type.as_ref().map(|s| s.as_ref()).unwrap_or("application/octet-stream");
    
        self.0  
            .run_mobile_plugin::<Res>("showSaveFileDialog", Req { default_file_name, mime_type })
            .map(|v| v.path)
            .map_err(Into::into)
    }

    fn take_persistable_read_permission(&self, path: &FilePath) -> crate::Result<()> {
        self.take_persistable_permission(path, "ReadOnly")
	}

	fn take_persistable_write_permission(&self, path: &FilePath) -> crate::Result<()> {
		self.take_persistable_permission(path, "WriteOnly")
	}

    fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isVisualMediaDialogAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn public_storage(&self) -> &impl crate::PublicStorage {
        self
    }

    fn private_storage(&self) -> &impl PrivateStorage {
        self
    }
}

impl<R: Runtime> PrivateStorage for AndroidFsImpl<R> {

    fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        impl_serde!(struct PrivateBaseDirAbsolutePaths { data: String, cache: String });
        
        static PATHS: std::sync::OnceLock<PrivateBaseDirAbsolutePaths> = std::sync::OnceLock::new();

        if PATHS.get().is_none() {
            let paths = self.0  
                .run_mobile_plugin::<PrivateBaseDirAbsolutePaths>("getPrivateBaseDirAbsolutePaths", "")?;

            let _ = PATHS.set(paths);
        }

        let paths = PATHS.get().unwrap();

        Ok(match dir {
            PrivateDir::Data => std::path::PathBuf::from(paths.data.to_owned()),
            PrivateDir::Cache => std::path::PathBuf::from(paths.cache.to_owned()),
        })
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

    fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isAudiobooksDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn is_recordings_dir_available(&self) -> crate::Result<bool> {
		impl_serde!(struct Res { value: bool });

        self.0  
            .run_mobile_plugin::<Res>("isRecordingsDirAvailable", "")
            .map(|v| v.value)
            .map_err(Into::into)
	}
}

impl<R: Runtime> AndroidFsImpl<R> {

    fn open_file_with_mode(&self, path: &FilePath, mode: &str) -> crate::Result<std::fs::File> {
        impl_serde!(struct Req<'a> { path: String, mode: &'a str });
        impl_serde!(struct Res { fd: std::os::fd::RawFd });
    
        let path = path.to_string();

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
    
        let path = path.to_string();

        self.0  
            .run_mobile_plugin::<Res>("savePublicFileAfterFailedWrite", Req { path })
            .map(|_| ())
            .map_err(Into::into)
    }

    fn save_public_file_after_succeed_write(&self, path: &FilePath) -> crate::Result<()> {
        impl_serde!(struct Req { path: String });
        impl_serde!(struct Res;);

        let path = path.to_string();
    
        self.0  
            .run_mobile_plugin::<Res>("savePublicFileAfterSucceedWrite", Req { path })
            .map(|_| ())
            .map_err(Into::into)
    }

    fn take_persistable_permission(&self, path: &FilePath, mode: &str) -> crate::Result<()> {
        impl_serde!(struct Req<'a> { path: String, mode: &'a str });
		impl_serde!(struct Res;);

        let path = path.to_string();

        self.0  
            .run_mobile_plugin::<Res>("takePersistableUriPermission", Req { path, mode })
            .map(|_| ())
            .map_err(Into::into)
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
