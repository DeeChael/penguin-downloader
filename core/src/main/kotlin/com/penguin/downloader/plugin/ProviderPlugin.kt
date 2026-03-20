package com.penguin.downloader.plugin

interface ProviderPlugin {
    fun onLoad(registry: ProviderRegistry)
}
