package com.penguin.downloader.download

interface DownloadNotifier {
    fun info(message: String)
    fun error(message: String)
    fun progress(message: String, overwrite: Boolean = false)
}

object StdoutDownloadNotifier : DownloadNotifier {
    override fun info(message: String) {
        println(message)
    }

    override fun error(message: String) {
        println(message)
    }

    override fun progress(message: String, overwrite: Boolean) {
        if (overwrite) {
            print("\r$message     ")
        } else {
            println(message)
        }
    }
}
