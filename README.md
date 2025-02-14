# Overview

The Android file system is strict and complex because its behavior and the available APIs vary depending on the version.
This plugin was created to provide explicit and consistent file operations.
No special permission or configuration is required.  

# Setup
All you need to do is register the core plugin with Tauri: 

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_android_fs::init()) // This
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

# Usage
There are three main ways to manipulate files:

### 1. Dialog
Opens the file picker to read and write user-selected files.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt};

fn read_files(app: tauri::AppHandle) {
    let api = app.android_fs();
    let selected_paths = api.show_open_file_dialog(
        &["*/*"], // Target MIME types
        true // Allow multiple files
    ).unwrap();

    if selected_paths.is_empty() {
        // Handle cancel
    }
    else {
        for path in selected_paths {
            let file_name = api.get_file_name(&path).unwrap();
            let file: std::fs::File = api.open_file(&path).unwrap();
            // Handle read-only file
        }
    }
}

fn write_file(app: tauri::AppHandle) {
    let api = app.android_fs();
    let selected_path = api.show_save_file_dialog(
        "", // Initial file name
        Some("image/png") // Target MIME type
    ).unwrap();

    if let Some(path) = selected_path {
        let mut file: std::fs::File = api.create_file(&path).unwrap();
        // Handle write-only file
    } 
    else {
        // Handle cancel
    }
}
```

### 2. Public Storage
File storage that is available to other applications and users.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PublicImageDir, PublicStorage};

fn example(app: tauri::AppHandle) {
    let api = app.android_fs().public_storage();
    let contents: Vec<u8> = todo!();

    // Write a PNG image
    api.write_image(
        PublicImageDir::Pictures, // Base directory
        "myApp/2025-02-13.png", // Relative file path
        Some("image/png"), // MIME type
        &contents
    ).unwrap();
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};

fn example(app: tauri::AppHandle) {
    let api = app.android_fs().private_storage();
    let contents: Vec<u8> = todo!();

    // Write contents
    api.write(
        PrivateDir::Data, // Base directory
        "config/data1", // Relative file path
        &contents
    ).unwrap();

    // Read contents
    let contents = api.read(
        PrivateDir::Data, // Base directory
        "config/data1" // Relative file path
    ).unwrap();
}
```

# License
MIT OR Apache-2.0
