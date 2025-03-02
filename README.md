# Overview

The Android file system is strict and complex because its behavior and the available APIs vary depending on the version.
This plugin was created to provide explicit and consistent file operations.
No special permission or configuration is required.  

# Setup
All you need to do is register this plugin with your Tauri project: 

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

- Currently, Dialog has an issue. Details and resolution are following.
<https://github.com/aiueo13/tauri-plugin-android-fs/issues/1>

Opens the file/folder picker to read and write user-selected entries.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, FileAccessMode};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    
    // pick files to read
    let selected_files = api.show_open_file_dialog(
        &["*/*"], // Target MIME types
        true // Allow multiple files
    )?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
            let file_type = api.get_mime_type(&uri)?;
            let file_name = api.get_name(&uri)?;
            let file: std::fs::File = api.open_file(&uri, FileAccessMode::Read)?;
            // Handle read-only file.

            // Alternatively, the uri can be returned to the front end, 
            // and file processing can be handled within another tauri::command function that takes it as an argument.
            //
            // If you need to use file data on frontend, 
            // consider using Tauri’s custom protocols for efficient transmission.
            let file_path: tauri_plugin_fs::FilePath = uri.into();
        }
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, Entry};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();

    // pick folder to read and write
    let selected_folder = api.show_open_dir_dialog()?;

    if let Some(uri) = selected_folder {
        for entry in api.read_dir(&uri)? {
            match entry {
                Entry::File { name, uri, last_modified, byte_size, mime_type, .. } => {
                    let file: std::fs::File = api.open_file(&uri, FileAccessMode::ReadWrite)?;
                    
                    // handle file
                },
                Entry::Dir { name, uri, last_modified, .. } => {
                    // handle folder
                },
            }
        }
    } 
    else {
        // Handle cancel
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, FileAccessMode};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();

    // pick file to write
    let selected_file = api.show_save_file_dialog(
        "", // Initial file name
        Some("image/png") // Target MIME type
    )?;

    if let Some(uri) = selected_file {
        let mut file: std::fs::File = api.open_file(&uri, FileAccessMode::WriteTruncate)?;
        // Handle write-only file
    } 
    else {
        // Handle cancel
    }
    Ok(())
}
```

### 2. Public Storage
File storage that is available to other applications and users.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PublicImageDir, PublicGeneralPurposeDir};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    let contents: Vec<u8> = vec![];

    // create a PNG image
    let uri = api.create_file_in_public_location(
         PublicImageDir::Pictures, // Base directory
         "my-image.png", // Relative file path
         Some("image/png") // Mime type
    )?;
    // write the contents to PNG image
    if let Err(e) = api.write(&uri, &contents) {
        // handle err
        api.remove_file(&uri)?;
        return Err(e)
    }

    // create a text file
    let uri = api.create_file_in_public_location(
         PublicGeneralPurposeDir::Documents, // Base directory
         "2025-3-2/data.txt", // Relative file path
         Some("text/plain") // Mime type
    )?;
    // write the contents to text file
    if let Err(e) = api.write(&uri, &contents) {
        // handle err
        api.remove_file(&uri)?;
        return Err(e)
    }

    Ok(())
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let storage = app.android_fs().private_storage();
    let contents: Vec<u8> = todo!();

    // Get the absolute path.
    // Apps require no permissions to read or write to this path.
    let path: std::path::PathBuf = storage.resolve_path(PrivateDir::Cache)?;


    // Write the contents.
    // This is wrapper of above resolve_path
    storage.write(
        PrivateDir::Data, // Base directory
        "config/data1", // Relative file path
        &contents
    )?;

    // Read the contents.
    // This is wrapper of above resolve_path
    let contents = storage.read(
        PrivateDir::Data, // Base directory
        "config/data1" // Relative file path
    )?;

    Ok(())
}
```

# Link
- [Changelog](https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/CHANGES.md)

# License
MIT OR Apache-2.0
