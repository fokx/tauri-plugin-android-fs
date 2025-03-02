package com.plugin.android_fs

import android.net.Uri
import android.webkit.MimeTypeMap
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import java.io.File

class RawFileController: FileController {

    override fun getMimeType(uri: FileUri): String? {
        return _getMimeType(File(Uri.parse(uri.uri).path!!))
    }

    override fun getName(uri: FileUri): String {
        return File(Uri.parse(uri.uri).path!!).name
    }

    override fun readDir(dirUri: FileUri): JSArray {
        val dir = File(Uri.parse(dirUri.uri).path!!)
        val buffer = JSArray()

        for (file in dir.listFiles()!!) {
            val uriObj = JSObject()
            uriObj.put("uri", file.toURI())
            uriObj.put("documentTopTreeUri", null)

            val obj = JSObject()
            obj.put("uri", uriObj)
            obj.put("mimeType", _getMimeType(file))
            obj.put("name", file.name)
            obj.put("lastModified", file.lastModified())
            obj.put("byteSize", file.length())
            buffer.put(obj)
        }

        return buffer
    }

    // この関数が返すUriは他のアプリに共有できない
    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        val dir = File(Uri.parse(dirUri.uri).path!!)
        val file = File(dir.path + "/" + relativePath.trimStart('/'))
        file.parentFile?.mkdirs()
        file.createNewFile()

        val res = JSObject()
        res.put("uri", Uri.fromFile(file))
        res.put("documentTopTreeUri", null)
        return res
    }

    override fun delete(uri: FileUri) {
        if (!File(Uri.parse(uri.uri).path!!).delete()) {
            throw Error("Failed to delete file: ${uri.uri}")
        }
    }

    override fun takePersistableUriPermission(uri: FileUri, flag: Int) {
        // Do nothing
    }


    // フォルダの場合のみnullを返す
    private fun _getMimeType(file: File): String? {
        if (file.isDirectory) {
            return null
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(file.extension)
            ?: "application/octet-stream"
    }
}