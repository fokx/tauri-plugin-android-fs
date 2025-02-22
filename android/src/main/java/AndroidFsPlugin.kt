package com.plugin.android_fs

import android.annotation.SuppressLint
import android.app.Activity
import android.content.ContentValues
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.activity.result.ActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.result.contract.ActivityResultContracts.PickMultipleVisualMedia
import androidx.activity.result.contract.ActivityResultContracts.PickVisualMedia
import androidx.documentfile.provider.DocumentFile
import app.tauri.Logger
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class GetFileDescriptorArgs {
    lateinit var mode: String
    lateinit var path: String
}

@InvokeArg
class GetFileNameArgs {
    lateinit var path: String
}

@InvokeArg
class ShowOpenFileDialogArgs {
    lateinit var mimeTypes: Array<String>
    var multiple: Boolean = false
}

@InvokeArg
class ShowOpenVisualMediaDialogArgs {
    lateinit var target: VisualMediaPickerType
    var multiple: Boolean = false
}

@InvokeArg
class ShowSaveFileDialogArgs {
    lateinit var defaultFileName: String
    lateinit var mimeType: String
}

@InvokeArg
enum class VisualMediaPickerType {
    ImageOnly,
    VideoOnly,
    ImageAndVideo
}

@InvokeArg
class SavePublicFileBeforeWriteArgs {
    lateinit var fileType: FileType
    lateinit var baseDir: BaseDir
    var mimeType: String? = null
    lateinit var subDir: String
    lateinit var fileName: String
}

@InvokeArg
class GetMimeTypeArgs {
    lateinit var path: String
}

@InvokeArg
class SavePublicFileAfterWriteArgs {
    lateinit var path: String
}

@InvokeArg
class TakePersistableUriPermissionArgs {
    lateinit var path: String
    lateinit var mode: PersistableUriPermissionMode
}

@InvokeArg
enum class PersistableUriPermissionMode {
    ReadOnly,
    WriteOnly,
    ReadAndWrite
}

@InvokeArg
enum class FileType {
    Image,
    Video,
    Audio,
    GeneralPurpose
}

@InvokeArg
enum class BaseDir {
    Pictures,
    Movies,
    Music,
    Alarms,
    Audiobooks,
    Notifications,
    Podcasts,
    Ringtones,
    Recordings,
    DCIM,
    Documents,
    Download,
}

@InvokeArg
class RemoveFileArgs {
    lateinit var path: String
}

@InvokeArg
class ReadDirArgs {
    lateinit var path: DirPath
}

@InvokeArg
class CreateFileInDirArgs {
    lateinit var path: DirPath
    lateinit var relativePath: String
    lateinit var mimeType: String
}

@InvokeArg
class GetDirNameArgs {
    lateinit var path: DirPath
}

@InvokeArg
class DirPath {
    lateinit var topTreeUri: String
    lateinit var relativeTerms: Array<String>
}

@TauriPlugin
class AndroidFsPlugin(private val activity: Activity): Plugin(activity) {
    private val isVisualMediaPickerAvailable = ActivityResultContracts.PickVisualMedia.isPhotoPickerAvailable()
    private val dirUtils = DirUtils()
    
    @Command
    fun createFileInDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateFileInDirArgs::class.java)

            val res = JSObject()
            res.put("path", dirUtils.createFile(activity, args.path, args.relativePath, args.mimeType).toString())
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke createFileInDir."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun readDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReadDirArgs::class.java)
            val res = JSObject()
            res.put("entries", dirUtils.getChildren(activity, args.path))
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke readDir."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun getDirName(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetDirNameArgs::class.java)
            val res = JSObject()
            res.put("name", dirUtils.getName(activity, args.path))
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getDirName."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun removeFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(RemoveFileArgs::class.java)
            val uri = Uri.parse(args.path)
            if (DocumentsContract.isDocumentUri(activity, uri)) {
                if (DocumentFile.fromSingleUri(activity, uri)?.delete() == true) {
                    invoke.resolve()
                }
                else {
                    invoke.reject("Failed to delete file: $uri")
                }
            }
            else {
                activity.contentResolver.delete(uri, null, null)
                invoke.resolve()
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke removeFile."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenDirDialog(invoke: Invoke) {
        try {
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
            startActivityForResult(invoke, intent, "dirDialogResult")
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenDirDialog."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @ActivityCallback
    private fun dirDialogResult(invoke: Invoke, result: ActivityResult) {
        try {
            val res = JSObject()
            
            val uri = result.data?.data?.toString()
            if (uri != null) {
                val obj = JSObject()
                obj.put("topTreeUri", uri)
                obj.put("relativeTerms", JSArray())
                res.put("path", obj)
            }
            else {
                res.put("path", null)
            }
            
            invoke.resolve(res)
        }
        catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to invoke dirDialogResult."
            Logger.error(message)
            invoke.reject(message)
        }
    }


    @Command
    fun getPrivateBaseDirAbsolutePaths(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("data", activity.filesDir.absolutePath)
            res.put("cache", activity.cacheDir.absolutePath)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getPrivateBaseDirAbsolutePaths."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun getMimeType(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetMimeTypeArgs::class.java)
            val uri = Uri.parse(args.path)

            activity.contentResolver.query(uri, arrayOf(), null, null, null).use {
                if (it?.moveToFirst() != true) {
                    throw Error("Failed to find file: $uri")
                }
            }

            val type = activity.contentResolver.getType(uri)
            val res = JSObject()
            res.put("value", type)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getMimeType."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun getFileName(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetFileNameArgs::class.java)

            val ret = JSObject()
            val uri = Uri.parse(args.path)
            var name: String? = null
            when (uri.scheme) {
                "content" -> {
                    val projection = arrayOf(MediaStore.MediaColumns.DISPLAY_NAME)
                    activity.contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
                        if (cursor.moveToFirst()) {
                            name = cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME))
                        }
                    }
                }
                "file" -> {
                    val path = uri.path
                    if (path != null) {
                        name = File(path).name
                    }
                }
            }

            if (name != null) {
                ret.put("name", name)
                invoke.resolve(ret)
            }
            else {
                invoke.reject("Failed to invoke getFileName.")
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileName."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun takePersistableUriPermission(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(TakePersistableUriPermissionArgs::class.java)

            // this is folder or file uri
            val uri = Uri.parse(args.path)

            val flag = when (args.mode) {
                PersistableUriPermissionMode.ReadOnly -> Intent.FLAG_GRANT_READ_URI_PERMISSION
                PersistableUriPermissionMode.WriteOnly -> Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                PersistableUriPermissionMode.ReadAndWrite -> Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            }

            activity.contentResolver.takePersistableUriPermission(uri, flag)
            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke takePersistableUriPermission."
            Logger.error(message)
            invoke.reject(message)
        }
    }
    
    @Command
    fun showOpenFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenFileDialogArgs::class.java)
            val intent = createFilePickerIntent(args.mimeTypes, args.multiple)

            startActivityForResult(invoke, intent, "filePickerResult")
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenFileDialog."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenVisualMediaDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenVisualMediaDialogArgs::class.java)
            val intent = createVisualMediaPickerIntent(args.multiple, args.target)

            startActivityForResult(invoke, intent, "filePickerResult")
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenVisualMediaDialog."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun showSaveFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowSaveFileDialogArgs::class.java)

            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT)

            intent.setType(args.mimeType)
            intent.addCategory(Intent.CATEGORY_OPENABLE)
            intent.putExtra(Intent.EXTRA_TITLE, args.defaultFileName)

            startActivityForResult(invoke, intent, "saveFileDialogResult")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to pick save file"
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @ActivityCallback
    fun saveFileDialogResult(invoke: Invoke, result: ActivityResult) {
        try {
            when (result.resultCode) {
                Activity.RESULT_OK -> {
                    val callResult = JSObject()
                    val intent: Intent? = result.data
                    if (intent != null) {
                        val uri = intent.data
                        if (uri != null) {
                            callResult.put("path", uri.toString())
                        }
                    }
                    invoke.resolve(callResult)
                }
                Activity.RESULT_CANCELED -> {
                    val callResult = JSObject()
                    callResult.put("path", null)
                    invoke.resolve(callResult)
                }
                else -> invoke.reject("Failed to pick files")
            }
        } catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to read file pick result"
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun isVisualMediaDialogAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", isVisualMediaPickerAvailable)
            invoke.resolve(res)
        }
        catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to invoke isVisualMediaDialogAvailable."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptor(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetFileDescriptorArgs::class.java)
            val fd = activity.contentResolver
                .openAssetFileDescriptor(Uri.parse(args.path), args.mode)!!
                .parcelFileDescriptor
                .detachFd()

            val res = JSObject()
            res.put("fd", fd)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileDescriptor."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun savePublicFileBeforeWrite(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SavePublicFileBeforeWriteArgs::class.java)
            val props = when (args.fileType) {
                FileType.Image -> getPropsForSaveImageFile()
                FileType.Video -> getPropsForSaveVideoFile()
                FileType.Audio -> getPropsForSaveAudioFile()
                FileType.GeneralPurpose -> getPropsForSaveGeneralPurposeFile()
            }
            val baseDir = when (args.baseDir) {
                BaseDir.Pictures -> Environment.DIRECTORY_PICTURES
                BaseDir.DCIM -> Environment.DIRECTORY_DCIM
                BaseDir.Movies -> Environment.DIRECTORY_MOVIES
                BaseDir.Music -> Environment.DIRECTORY_MUSIC
                BaseDir.Alarms -> Environment.DIRECTORY_ALARMS
                BaseDir.Audiobooks -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    Environment.DIRECTORY_AUDIOBOOKS
                } else {
                    throw Error("Environment.DIRECTORY_AUDIOBOOKS isn't available on Android 9 (API level 28) and lower.")
                }
                BaseDir.Notifications -> Environment.DIRECTORY_NOTIFICATIONS
                BaseDir.Podcasts -> Environment.DIRECTORY_PODCASTS
                BaseDir.Ringtones -> Environment.DIRECTORY_RINGTONES
                BaseDir.Documents -> Environment.DIRECTORY_DOCUMENTS
                BaseDir.Download -> Environment.DIRECTORY_DOWNLOADS
                BaseDir.Recordings -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    Environment.DIRECTORY_RECORDINGS
                } else {
                    throw Error("Environment.DIRECTORY_RECORDINGS isn't available on Android 11 (API level 30) and lower.")
                }
            }

            val contentValues = ContentValues().apply {
                put(props.displayName, args.fileName)
                if (args.mimeType != null) {
                    put(props.mimeType, args.mimeType)
                }
                if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                    put(MediaStore.MediaColumns.IS_PENDING, true)
                }
                put(props.relativePath, baseDir + "/" + args.subDir + "/")
            }

            val resolver = activity.contentResolver
            val destUri = resolver.insert(props.parentDir, contentValues)!!
            val fd = resolver
                .openAssetFileDescriptor(destUri, "w")!!
                .parcelFileDescriptor
                .detachFd()

            val res = JSObject()
            res.put("fd", fd)
            res.put("path", destUri.toString())
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke savePublicFileBeforeWrite."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun savePublicFileAfterFailedWrite(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SavePublicFileAfterWriteArgs::class.java)
            activity.contentResolver.delete(Uri.parse(args.path), null, null)
            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke savePublicFileAfterFailedWrite."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun savePublicFileAfterSucceedWrite(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SavePublicFileAfterWriteArgs::class.java)

            if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                val values = ContentValues().apply {
                    put(MediaStore.MediaColumns.IS_PENDING, false)
                }
                activity.contentResolver.update(Uri.parse(args.path), values, null, null)
            }

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke savePublicFileAfterSucceedWrite."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun isAudiobooksDirAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke isAudiobooksDirAvailable."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun isRecordingsDirAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke isRecordingsDirAvailable."
            Logger.error(message)
            invoke.reject(message)
        }
    }



    @ActivityCallback
    fun filePickerResult(invoke: Invoke, result: ActivityResult) {
        try {
            when (result?.resultCode) {
                Activity.RESULT_OK -> {
                    val callResult = createPickFilesResult(result.data)
                    invoke.resolve(callResult)
                }
                Activity.RESULT_CANCELED -> {
                    val callResult = createPickFilesResult(null)
                    invoke.resolve(callResult)
                }
                else -> invoke.reject("Failed to pick files")
            }
        }
        catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to read file pick result"
            Logger.error(message)
            invoke.reject(message)
        }
    }

    private fun createPickFilesResult(data: Intent?): JSObject {
        val callResult = JSObject()
        if (data == null) {
            callResult.put("paths", JSArray())
            return callResult
        }
        val uris: MutableList<String?> = ArrayList()
        if (data.clipData == null) {
            val uri: Uri? = data.data
            uris.add(uri?.toString())
        } else {
            for (i in 0 until data.clipData!!.itemCount) {
                val uri: Uri = data.clipData!!.getItemAt(i).uri
                uris.add(uri.toString())
            }
        }
        callResult.put("paths", JSArray.from(uris.toTypedArray()))
        return callResult
    }

    private fun createFilePickerIntent(mimeTypes: Array<String>, multiple: Boolean): Intent {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT)
            .addCategory(Intent.CATEGORY_OPENABLE)
            .putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);

        if (mimeTypes.isEmpty()) {
            return intent.setType("*/*")
        }
        else if (mimeTypes.size == 1) {
            return intent.setType(mimeTypes[0])
        }

        return intent
            .setType("*/*")
            .putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes)
    }

    private fun createVisualMediaPickerIntent(multiple: Boolean, target: VisualMediaPickerType): Intent {
        val req = PickVisualMediaRequest(when (target) {
            VisualMediaPickerType.ImageOnly -> PickVisualMedia.ImageOnly
            VisualMediaPickerType.VideoOnly -> PickVisualMedia.VideoOnly
            VisualMediaPickerType.ImageAndVideo -> PickVisualMedia.ImageAndVideo
        })

        return when (multiple) {
            true -> PickMultipleVisualMedia().createIntent(activity, req)
            false -> PickVisualMedia().createIntent(activity, req)
        }
    }


    data class SaveFileProps(
        val parentDir: Uri,
        val displayName: String,
        val mimeType: String,
        val relativePath: String,
    )

    private fun getPropsForSaveImageFile(): SaveFileProps {
        val parentDir = if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            MediaStore.Images.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        } else {
            MediaStore.Images.Media.EXTERNAL_CONTENT_URI
        }

        return SaveFileProps(
            parentDir = parentDir,
            displayName = MediaStore.Images.Media.DISPLAY_NAME,
            mimeType = MediaStore.Images.Media.MIME_TYPE,
            relativePath = MediaStore.Images.ImageColumns.RELATIVE_PATH,
        )
    }

    private fun getPropsForSaveVideoFile(): SaveFileProps {
        val parentDir = if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        } else {
            MediaStore.Video.Media.EXTERNAL_CONTENT_URI
        }

        return SaveFileProps(
            parentDir = parentDir,
            displayName = MediaStore.Video.Media.DISPLAY_NAME,
            mimeType = MediaStore.Video.Media.MIME_TYPE,
            relativePath = MediaStore.Video.VideoColumns.RELATIVE_PATH,
        )
    }

    private fun getPropsForSaveAudioFile(): SaveFileProps {
        val parentDir = if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        } else {
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
        }

        return SaveFileProps(
            parentDir = parentDir,
            displayName = MediaStore.Audio.Media.DISPLAY_NAME,
            mimeType = MediaStore.Audio.Media.MIME_TYPE,
            relativePath = MediaStore.Audio.AudioColumns.RELATIVE_PATH,
        )
    }

    private fun getPropsForSaveGeneralPurposeFile(): SaveFileProps {
        val parentDir = if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        } else {
            MediaStore.Files.getContentUri("external")
        }

        return SaveFileProps(
            parentDir = parentDir,
            displayName = MediaStore.Files.FileColumns.DISPLAY_NAME,
            mimeType = MediaStore.Files.FileColumns.MIME_TYPE,
            relativePath = MediaStore.Files.FileColumns.RELATIVE_PATH
        )
    }
}
