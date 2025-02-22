package com.plugin.android_fs

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.core.database.getStringOrNull
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class DirUtils {

    //---------------------------------------------------------------------------------//
    // 以下、ACTION_OPEN_DOCUMENT_TREEで直接選択されたフォルダをTopTreeと呼称する。
    // DocumentFile.fromTreeUriはTopTreeでしか正しく動作しないことに注意。
    // TopTree内のサブフォルダを指定してもエラーにならず、代わりにTopTreeのフォルダが返されてしまう。
    // またDocumentFileにあるlistFilesなどのメソッドはとても遅いので使うべきでない。
    //---------------------------------------------------------------------------------//


    fun createFile(
        activity: Context,
        dir: DirPath,
        relativePath: String,
        mimeType: String,
    ): Uri {

        val topTreeUri = Uri.parse(dir.topTreeUri)
        val terms = relativePath.split("/").filter { it.isNotEmpty() }

        var parentId = when (dir.relativeTerms.isEmpty()) {
            true -> DocumentsContract.getTreeDocumentId(topTreeUri)
            false -> dir.relativeTerms.last()
        }

        // サブフォルダが存在しなければ再帰的に作成する
        for (dirName in terms.dropLast(1)) {
            parentId = findId(activity, topTreeUri, parentId, dirName) ?: DocumentsContract.getDocumentId(
                DocumentsContract.createDocument(
                    activity.contentResolver,
                    DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId),
                    DocumentsContract.Document.MIME_TYPE_DIR,
                    dirName
                )
            )
        }

        val pUri = DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId)
        val fName = terms.last()
        return DocumentsContract.createDocument(
            activity.contentResolver,
            pUri,
            mimeType,
            fName
        ) ?: throw Error("Failed to create file: { parent: $pUri, fileName: $fName, mimeType: $mimeType }")
    }

    fun getName(
        activity: Context,
        dir: DirPath
    ): String {

        val topTreeUri = Uri.parse(dir.topTreeUri)
        val targetUri = DocumentsContract.buildDocumentUriUsingTree(
            topTreeUri,
            dir.relativeTerms.lastOrNull() ?: DocumentsContract.getTreeDocumentId(topTreeUri)
        )

        val cursor = activity.contentResolver.query(
            targetUri,
            arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME),
            null,
            null,
            null
        )

        cursor?.use {
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)

            if (cursor.moveToFirst()) {
                return cursor.getString(nameColumnIndex)
            }
        }

        throw Error("Failed to find file: $targetUri")
    }

    fun getChildren(
        activity: Context,
        dir: DirPath
    ): JSArray {

        val ids = dir.relativeTerms
        val topTreeUri = Uri.parse(dir.topTreeUri)
        val targetUri = DocumentsContract.buildChildDocumentsUriUsingTree(
            topTreeUri,
            ids.lastOrNull() ?: DocumentsContract.getTreeDocumentId(topTreeUri)
        )

        val buffer = JSArray()
        val cursor = activity.contentResolver.query(
            targetUri,
            arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED,
                DocumentsContract.Document.COLUMN_SIZE
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val mimeTypeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val lastModifiedColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_LAST_MODIFIED)
            val sizeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_SIZE)
            val topTreeUriString = topTreeUri.toString()

            while (cursor.moveToNext()) {
                val mimeType = cursor.getStringOrNull(mimeTypeColumnIndex) ?: "application/octet-stream"
                val id = cursor.getString(idColumnIndex)
                val isDir = mimeType == DocumentsContract.Document.MIME_TYPE_DIR

                val path = if (isDir) {
                    val path = JSObject()
                    path.put("topTreeUri", topTreeUriString)
                    path.put("relativeTerms", JSArray(ids.plus(id)))

                    val obj = JSObject()
                    obj.put("type", "Dir")
                    obj.put("path", path)
                    obj
                }
                else {
                    val childUri = DocumentsContract.buildDocumentUriUsingTree(topTreeUri, id)
                    val obj = JSObject()
                    obj.put("type", "File")
                    obj.put("path", childUri.toString())
                    obj
                }

                val lastModified = cursor.getLong(lastModifiedColumnIndex)
                val size = cursor.getLong(sizeColumnIndex)
                val name = cursor.getString(nameColumnIndex)

                val obj = JSObject()
                obj.put("name", name)
                obj.put("path", path)
                obj.put("mimeType", mimeType)
                obj.put("lastModified", lastModified)
                obj.put("byteSize", size)
                buffer.put(obj)
            }
        }

        return buffer
    }

    private fun findId(
        activity: Context,
        topTreeUri: Uri,
        parentId: String,
        name: String,
    ): String? {

        val parentUri = DocumentsContract.buildChildDocumentsUriUsingTree(
            topTreeUri,
            parentId
        )

        val cursor = activity.contentResolver.query(
            parentUri,
            arrayOf(
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_DOCUMENT_ID
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)

            while (cursor.moveToNext()) {
                if (name == cursor.getString(nameColumnIndex)) {
                    return cursor.getString(idColumnIndex)
                }
            }
        }

        return null
    }
}