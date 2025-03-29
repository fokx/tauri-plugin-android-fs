package com.plugin.android_fs

import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

interface FileController {

    fun getMimeType(uri: FileUri): String?

    fun getName(uri: FileUri): String

    fun readDir(dirUri: FileUri): JSArray

    fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject

    fun delete(uri: FileUri)
}