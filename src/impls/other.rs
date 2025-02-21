use std::fs::File;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};
use crate::{models::*, AndroidFs, FilePath, PrivateStorage, PublicStorage};


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

    fn get_file_name(&self, _path: &FilePath) -> crate::Result<String> {
        Err(crate::Error::NotAndroid)
    }

    fn get_mime_type(&self, _path: &FilePath) -> crate::Result<Option<String>> {
        Err(crate::Error::NotAndroid)
    }

    fn open_file(&self, _path: &FilePath) -> crate::Result<File> {
        Err(crate::Error::NotAndroid)
    }

    fn create_file(&self, _path: &FilePath) -> crate::Result<std::fs::File> {
        Err(crate::Error::NotAndroid)
    }

    fn show_open_file_dialog(
        &self,
        _mime_types: &[&str],
        _multiple: bool
    ) -> crate::Result<Vec<FilePath>> {
		
        Err(crate::Error::NotAndroid)
    }

    fn show_open_visual_media_dialog(
        &self,
        _target: VisualMediaTarget,
        _multiple: bool
    ) -> crate::Result<Vec<FilePath>> {
		
        Err(crate::Error::NotAndroid)
    }

    fn show_open_dir_dialog(&self) -> crate::Result<Option<DirPath>> {
        Err(crate::Error::NotAndroid)
    }

    fn read_dir(&self, _path: &DirPath) -> crate::Result<Vec<EntryPath>> {
        Err(crate::Error::NotAndroid)
    }

    fn get_dir_name(&self, _path: &DirPath) -> crate::Result<String> {
        Err(crate::Error::NotAndroid)
    }

    fn show_save_file_dialog(
        &self,
        _default_file_name: impl AsRef<str>,
        _mime_type: Option<&str>,
    ) -> crate::Result<Option<FilePath>> {

        Err(crate::Error::NotAndroid)
    }

    fn grant_persistable_file_access(&self, _path: &FilePath, _mode: PersistableAccessMode) -> crate::Result<()> {
        Err(crate::Error::NotAndroid)
    }

    fn grant_persistable_dir_access(&self, _path: &DirPath, _mode: PersistableAccessMode) -> crate::Result<()> {
        Err(crate::Error::NotAndroid)
    }

    fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }

    fn public_storage(&self) -> &impl crate::PublicStorage {
        self
    }

    fn private_storage(&self) -> &impl crate::PrivateStorage {
        self
    }
}

impl<R: Runtime> PublicStorage for AndroidFsImpl<R> {

    fn write_with_contents_writer(
        &self,
        _base_dir: PublicGeneralPurposeDir,
        _relative_path_with_sub_dir: impl AsRef<str>,
        _mime_type: Option<&str>,
        _contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
		
        Err(crate::Error::NotAndroid)
    }

    fn write_image_with_contents_writer(
        &self,
        _base_dir: PublicImageDir,
        _relative_path: impl AsRef<str>,
        _mime_type: Option<&str>,
        _contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
		
        Err(crate::Error::NotAndroid)
    }

    fn write_video_with_contents_writer(
        &self,
        _base_dir: PublicVideoDir,
        _relative_path: impl AsRef<str>,
        _mime_type: Option<&str>,
        _contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
		
        Err(crate::Error::NotAndroid)
    }

    fn write_audio_with_contents_writer(
        &self,
        _base_dir: PublicAudioDir,
        _relative_path: impl AsRef<str>,
        _mime_type: Option<&str>,
        _contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>
    ) -> crate::Result<FilePath> {
		
        Err(crate::Error::NotAndroid)
    }

    fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }

    fn is_recordings_dir_available(&self) -> crate::Result<bool> {
        Err(crate::Error::NotAndroid)
    }
}

impl<R: Runtime> PrivateStorage for AndroidFsImpl<R> {

    fn resolve_path(&self, _dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        Err(crate::Error::NotAndroid)
    }
}
