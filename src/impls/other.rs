use std::fs::File;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};
use crate::{models::*, AndroidFs, PrivateStorage};


/// Access to the android-fs APIs.
pub struct AndroidFsImpl<R: Runtime>(AppHandle<R>);

impl<R: Runtime> AndroidFsImpl<R> {

    pub fn new<C: DeserializeOwned>(
        app: &AppHandle<R>,
        _api: PluginApi<R, C>,
    ) -> crate::Result<impl AndroidFs> {
		
        Ok(Self(app.clone()))
    }
}

impl<R: Runtime> AndroidFs for AndroidFsImpl<R> {

    fn get_name(&self, _path: &FileUri) -> crate::Result<String> {
        Err(crate::Error::NotAndroid)
    }

    fn get_mime_type(&self, _path: &FileUri) -> crate::Result<Option<String>> {
        Err(crate::Error::NotAndroid)
    }

    fn open_file(&self, _path: &FileUri, _mode: FileAccessMode) -> crate::Result<File> {
        Err(crate::Error::NotAndroid)
    }

    fn show_open_file_dialog(
        &self,
        _mime_types: &[&str],
        _multiple: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        Err(crate::Error::NotAndroid)
    }

    fn show_open_visual_media_dialog(
        &self,
        _target: VisualMediaTarget,
        _multiple: bool
    ) -> crate::Result<Vec<FileUri>> {
		
        Err(crate::Error::NotAndroid)
    }

    fn show_save_file_dialog(
        &self,
        _default_file_name: impl AsRef<str>,
        _mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>> {

        Err(crate::Error::NotAndroid)
    }

    fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }

    fn private_storage(&self) -> &impl crate::PrivateStorage {
        self
    }
    
    fn remove_file(&self, _uri: &FileUri) -> crate::Result<()> {
        Err(crate::Error::NotAndroid)
    }

    fn remove_dir(&self, _uri: &FileUri) -> crate::Result<()> {
        Err(crate::Error::NotAndroid)
    }
    
    fn create_file(
        &self,
        _dir: &FileUri, 
        _relative_path: impl AsRef<str>, 
        _mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        Err(crate::Error::NotAndroid)
    }
    
    fn create_file_in_public_location(
        &self,
        _dir: impl Into<PublicDir>,
        _relative_path_with_sub_dir: impl AsRef<str>, 
        _mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        Err(crate::Error::NotAndroid)
    }
    
    fn read_dir(&self, _path: &FileUri) -> crate::Result<impl Iterator<Item = Entry>> {
        Err::<std::iter::Empty<_>, _>(crate::Error::NotAndroid)
    }
    
    fn take_persistable_uri_permission(&self, _uri: FileUri, _mode: PersistableAccessMode) -> crate::Result<()> {
        Err(crate::Error::NotAndroid)
    }
    
    fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>> {
        Err(crate::Error::NotAndroid)
    }
    
    fn is_public_audiobooks_dir_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }
    
    fn is_public_recordings_dir_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }
}

impl<R: Runtime> PrivateStorage for AndroidFsImpl<R> {

    fn resolve_path(&self, _dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        Err(crate::Error::NotAndroid)
    }
}