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
    let builder = Builder::new("android-fs")
        .setup(|app, api| {
            app.manage(AndroidFsImpl::new(app, api)?);
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

pub trait AndroidFsExt<R: Runtime> {

    fn android_fs(&self) -> &impl AndroidFs;
}

impl<R: Runtime, T: Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &impl AndroidFs {
        self.state::<AndroidFsImpl<R>>().inner()
    }
}