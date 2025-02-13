#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
use android::AndroidFsImpl;

#[cfg(not(target_os = "android"))]
mod other;
#[cfg(not(target_os = "android"))]
use other::AndroidFsImpl;

use crate::AndroidFs;
use tauri::{plugin::{Builder, TauriPlugin}, Manager, Runtime};


/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("android-fs")
        .setup(|app, api| {
            app.manage(AndroidFsImpl::new(app, api)?);
            Ok(())
        })
        .build()
}

pub trait AndroidFsExt<R: Runtime> {

    fn android_fs(&self) -> &impl AndroidFs;
}

impl<R: Runtime, T: Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &impl AndroidFs {
        self.state::<AndroidFsImpl<R>>().inner()
    }
}