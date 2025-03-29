# Overview

The Android file system is strict and complex because its behavior and the available APIs vary depending on the version.
This plugin was created to provide explicit and consistent file operations.
No special permission or configuration is required.  

# Setup
Register this plugin with your Tauri project: 

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

The current dialog in this plugin has an [issue](https://github.com/aiueo13/tauri-plugin-android-fs/issues/1). To avoid this, please follow these two steps:

`src-tauri/Cargo.toml`
```toml
[dependencies]
tauri-plugin-android-fs = { features = ["avoid-issue1"], .. }
```

`src-tauri/capabilities/default.json`
```json
{
  ..
  "permissions": [
    "android-fs:default",
    ..
  ]
}
```

# Usage
This plugin can use only from rust side.  
Then, there are three main ways to manipulate files:

### 1. Dialog

Opens the file/folder picker to read and write user-selected entries.

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, FileAccessMode};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    
    // pick files to read
    let selected_files = api.show_open_file_dialog(
        None, // Initial location
        &["*/*"], // Target MIME types
        true, // Allow multiple files
        false // Enable uri until the app is closed
    )?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
            let file_type = api.get_mime_type(&uri)?.unwrap(); // If file, this returns no None.
            let file_name = api.get_name(&uri)?;

            // Handle read-only file.
            let file: std::fs::File = api.open_file(&uri, FileAccessMode::Read)?;

            // Alternatively, the uri can be returned to the front end, 
            // and file processing can be handled within another tauri::command function that takes it as an argument.
            //
            // If you need to use file data on frontend, 
            // consider using Tauri’s custom protocols for efficient transmission.
            // Or convert to FilePath and use tauri_plugin_fs functions such as 'read' on frontend.
            let file_path: tauri_plugin_fs::FilePath = uri.into();
        }
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, Entry, FileAccessMode};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();

    // pick folder to read and write
    let selected_folder = api.show_manage_dir_dialog(
        None, // Initial location
        false // Enable uri until the app is closed
    )?;

    if let Some(dir_uri) = selected_folder {

        // create a new empty file in the selected folder
        let new_file_uri = api.create_file(
            &dir_uri, // Parent folder
            "MyFolder/file.txt", // Relative path
            Some("text/plain") // Mime type
        )?;

        let new_file: std::fs::File = api.open_file(&new_file_uri, FileAccessMode::WriteTruncate)?;

        // peek children
        for entry in api.read_dir(&dir_uri)? {
            match entry {
                Entry::File { name, uri, last_modified, len, mime_type, .. } => {
                    // handle file
                    let file: std::fs::File = api.open_file(&uri, FileAccessMode::ReadWrite)?;
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

    // pick file to write (create a new empty file by user)
    let selected_file = api.show_save_file_dialog(
        None, // Initial location
        "", // Initial file name
        Some("image/png"), // MIME type
        false, // Enable uri until the app is closed
    )?;

    if let Some(uri) = selected_file {
        // Handle write-only file
        let mut file: std::fs::File = api.open_file(&uri, FileAccessMode::WriteTruncate)?;
        
        // If you need to write file from frontend, 
        // convert to FilePath and use tauri_plugin_fs functions such as 'write' on frontend.
        let file_path: tauri_plugin_fs::FilePath = uri.into();
    } 
    else {
        // Handle cancel
    }
    Ok(())
}
```

### 2. Public Storage
File storage that is available to other applications and users.
Currently, this is for Android 10 (API level 29) or higher.  

```rust
use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PublicGeneralPurposeDir, PublicImageDir, PublicStorage};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    let storage = api.public_storage();
    let contents: Vec<u8> = vec![];

    // create a new empty PNG image file
    //
    // This path is represented as follows
    // ~/Pictures/{app_name}/my-image.png
    // $HOME/Pictures/{app_name}/my-image.png
    // /storage/emulated/0/Pictures/{app_name}/my-image.png
    let uri = storage.create_file_in_public_app_dir(
         PublicImageDir::Pictures, // Base directory
         "my-image.png", // Relative file path
         Some("image/png") // Mime type
    )?;

    // write the contents to the PNG image
    if let Err(e) = api.write(&uri, &contents) {
        // handle err
        let _ = api.remove_file(&uri);
        return Err(e)
    }


    // create a new empty text file
    //
    // This path is represented as follows
    // ~/Documents/{app_name}/2025-3-2/data.txt
    // $HOME/Documents/{app_name}/2025-3-2/data.txt
    // /storage/emulated/0/Documents/{app_name}/2025-3-2/data.txt
    let uri = storage.create_file_in_public_app_dir(
         PublicGeneralPurposeDir::Documents, // Base directory
         "2025-3-2/data.txt", // Relative file path
         Some("text/plain") // Mime type
    )?;

    // write the contents to the text file
    if let Err(e) = api.write(&uri, &contents) {
        // handle err
        let _ = api.remove_file(&uri);
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
    // This is wrapper of above resolve_path and std::fs::write
    storage.write(
        PrivateDir::Data, // Base directory
        "config/data1", // Relative file path
        &contents
    )?;

    // Read the contents.
    // This is wrapper of above resolve_path and std::fs::read
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
