//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused)]

mod models;
mod error;
mod api;

pub use models::*;
pub use error::{Error, Result};
pub use api::{AndroidFs, PrivateStorage, PublicStorage};


pub(crate) const TMP_DIR_RELATIVE_PATH: &str = "pluginAndroidFs-tmpDir-33bd1538-4434-dc4e-7e2f-515405cccbf9";

/// Initializes the plugin.
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let builder = tauri::plugin::Builder::new("android-fs")
        .setup(|app, api| {
            use tauri::Manager as _;

            let afs = AndroidFs::new(app.clone(), api)?;

            // Cleanup temporary directory
            let _ = afs
                .private_storage()
                .remove_dir_all(PrivateDir::Cache, Some(TMP_DIR_RELATIVE_PATH));

            app.manage(afs);
            Ok(())
        });

    // https://github.com/aiueo13/tauri-plugin-android-fs/issues/1
    #[cfg(all(feature = "avoid-issue1", target_os = "android"))]
    let builder = {
        const SCRIPT: &str = "
            ;(async function () {
                const noop = async () => await window.__TAURI_INTERNALS__.invoke('plugin:android-fs|noop');

                // check noop is allowed
                await noop()

                setInterval(noop, 800)
            })();
        ";

        #[tauri::command]
        fn noop() {}

        builder
            .invoke_handler(tauri::generate_handler![noop])
            .js_init_script(SCRIPT.into())  
    };

    builder.build()
}

pub trait AndroidFsExt<R: tauri::Runtime> {

    fn android_fs(&self) -> &AndroidFs<R>;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &AndroidFs<R> {
        self.state::<AndroidFs<R>>().inner()
    }
}