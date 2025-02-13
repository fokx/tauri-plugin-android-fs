# Overview

The Android file system is strict and complex because its behavior and the available APIs vary depending on the version.
This plugin was created to provide explicit and consistent file operations.
No special permission or configuration is required.  

# Usage
There are three main ways to manipulate files:

### 1. Dialog
Opens the file picker to read and write user-selected files.

```
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, VisualMediaTarget};

fn read_files(app: tauri::AppHandle) {
    let api = app.android_fs();
    let selected_paths = api.show_open_file_dialog(&["*/*"], true).unwrap();

    if selected_paths.is_empty() {
        // Handle cancellation
    } else {
        for path in selected_paths {
            let file_name = api.get_file_name(&path).unwrap();
            let file: std::fs::File = api.open_file(&path).unwrap();
        }
    }
}

fn write_file(app: tauri::AppHandle) {
    let api = app.android_fs();
    let selected_path = api.show_save_file_dialog("fileName", Some("image/png")).unwrap();

    if let Some(path) = selected_path {
        let mut file: std::fs::File = api.open_file_writable(&path).unwrap();
    } else {
        // Handle cancellation
    }
}
```

### 2. Public Storage
File storage intended to be shared with other apps and user.

```
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PublicImageDir, PublicStorage};

fn example(app: tauri::AppHandle) {
    let api = app.android_fs().public_storage();
    let contents: Vec<u8> = todo!();

    api.write_image(
        PublicImageDir::Pictures,
        "myApp/2025-02-13.png",
        Some("image/png"),
        &contents
    ).unwrap();
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};

fn example(app: tauri::AppHandle) {
    let api = app.android_fs().private_storage();

    // Write data
    api.write(PrivateDir::Data, "config/data1.txt", "data").unwrap();

    // Read data
    let data = api.read_to_string(PrivateDir::Data, "config/data1.txt").unwrap();
    assert_eq!(data, "data");
}
```

# License
MIT OR Apache-2.0