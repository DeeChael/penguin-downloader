package com.penguin.downloader.provider

interface QualityLevel {
    val levels: Map<Int, String>
    
    fun getName(level: Int): String = levels[level] ?: "未知音质"
    
    companion object {
        const val STANDARD = 1
        const val HIGH = 2
        const val LOSSLESS = 3
        const val HI_RES = 4
        const val DOLBY = 5
        const val SPATIAL = 6
        const val MASTER = 7
    }
}
