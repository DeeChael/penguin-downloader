package com.penguin.downloader.plugin

import com.penguin.downloader.provider.MusicProvider
import java.io.File
import java.net.URLClassLoader
import java.util.ServiceLoader
import java.util.concurrent.ConcurrentHashMap

object ProviderRegistry {
    private val providers = ConcurrentHashMap<String, MusicProvider>()
    private val providerFactories = ConcurrentHashMap<String, () -> MusicProvider>()
    
    fun register(name: String, provider: MusicProvider) {
        providers[name] = provider
    }
    
    fun registerFactory(name: String, factory: () -> MusicProvider) {
        providerFactories[name] = factory
    }
    
    fun getProvider(name: String): MusicProvider? {
        providers[name]?.let { return it }
        return providerFactories[name]?.invoke()
    }
    
    fun getProviderNames(): Set<String> {
        return providers.keys + providerFactories.keys
    }
    
    fun loadPlugins(pluginsDir: File) {
        if (!pluginsDir.exists()) {
            pluginsDir.mkdirs()
            return
        }
        
        val jarFiles = pluginsDir.listFiles { file -> 
            file.extension == "jar" 
        } ?: return
        
        val urls = jarFiles.map { it.toURI().toURL() }.toTypedArray()
        if (urls.isEmpty()) return
        
        val classLoader = URLClassLoader(urls, javaClass.classLoader)
        
        val serviceLoader = ServiceLoader.load(MusicProvider::class.java, classLoader)
        for (provider in serviceLoader) {
            register(provider.name, provider)
        }
        
        val pluginLoader = ServiceLoader.load(ProviderPlugin::class.java, classLoader)
        for (plugin in pluginLoader) {
            plugin.onLoad(this)
        }
    }
}
