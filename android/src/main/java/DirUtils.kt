package com.plugin.android_fs

import android.content.Context
import android.net.Uri
import android.os.Build
import androidx.documentfile.provider.DocumentFile
import android.provider.DocumentsContract
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject


class DirUtils {

    //---------------------------------------------------------------------------------//
    // 以下、ACTION_OPEN_DOCUMENT_TREEで直接選択されたフォルダをTopTreeと呼称する。
    // DocumentFile.fromTreeUriはTopTreeでしか正しく動作しないことに注意。
    // TopTree内のサブフォルダを指定してもエラーにならず、代わりにTopTreeのフォルダが返されてしまう。
    // またDocumentFileにあるlistFilesなどのメソッドはとても遅いので使うべきでない。
    //---------------------------------------------------------------------------------//


    //---------------------------------------------------------------------------------//
    // 以下、ACTION_OPEN_DOCUMENT_TREEで直接選択されたフォルダをTopTreeと呼称する。
    // DocumentFile.fromTreeUriはTopTreeでしか正しく動作しないことに注意。
    // TopTree内のサブフォルダを指定してもエラーにならず、代わりにTopTreeのフォルダが返されてしまう。
    // またDocumentFileにあるlistFilesやfindFileなどのメソッドはとても遅いので使うべきでない。
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

        val terms = dir.relativeTerms
        val topTreeUri = Uri.parse(dir.topTreeUri)
        val parentUri = when (terms.size) {
            // termがない場合はtopTree自身を指しているため、これの名前を取得して返す
            0 -> {
                return DocumentFile.fromTreeUri(activity, topTreeUri)?.name
                    ?: throw Error("Failed to get name from $topTreeUri")
            }
            // termが1つのみの場合はtopTree直下のサブフォルダであるのでtopTreeが親となる
            1 -> {
                DocumentsContract.buildChildDocumentsUriUsingTree(
                    topTreeUri,
                    DocumentsContract.getTreeDocumentId(topTreeUri)
                )
            }
            // termが２つ以上の時、後ろから1つめのtermを親とする
            else -> {
                DocumentsContract.buildChildDocumentsUriUsingTree(
                    topTreeUri,
                    terms[terms.size - 2]
                )
            }
        }
        val targetId = terms.last()

        val cursor = activity.contentResolver.query(
            parentUri,
            arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)

            while (cursor.moveToNext()) {
                val id = cursor.getString(idColumnIndex)
                if (id == targetId) {
                    return cursor.getString(nameColumnIndex)
                }
            }
        }

        throw Error("Failed to find name from { parent: $parentUri, id: $targetId }")
    }

    // 指定したフォルダ内の要素を全て取得する
    // Rustにおける { name: String, path: EntryPath } の Vec を返す
    fun getChildren(
        activity: Context,
        dir: DirPath
    ): JSArray {

        val ids = dir.relativeTerms
        val targetId = ids.lastOrNull()
        val topTreeUri = Uri.parse(dir.topTreeUri)
        val targetUri = if (targetId == null) {
            DocumentsContract.buildChildDocumentsUriUsingTree(
                topTreeUri,
                DocumentsContract.getTreeDocumentId(topTreeUri)
            )
        }
        else {
            DocumentsContract.buildChildDocumentsUriUsingTree(
                topTreeUri,
                targetId
            )
        }

        val buffer = JSArray()
        val cursor = activity.contentResolver.query(
            targetUri,
            arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val mimeTypeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val topTreeUriString = topTreeUri.toString()

            while (cursor.moveToNext()) {
                val mimeType = cursor.getString(mimeTypeColumnIndex)
                val name = cursor.getString(nameColumnIndex)
                val id = cursor.getString(idColumnIndex)
                val path = if (mimeType == DocumentsContract.Document.MIME_TYPE_DIR) {
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

                val obj = JSObject()
                obj.put("name", name)
                obj.put("path", path)
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