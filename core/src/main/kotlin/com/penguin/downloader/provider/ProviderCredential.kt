package com.penguin.downloader.provider

interface ProviderCredential {
    fun isValid(): Boolean
    fun isExpired(): Boolean = false
    fun needRefresh(): Boolean = false
    fun canRefresh(): Boolean = false
}
