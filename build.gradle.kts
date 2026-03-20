plugins {
    kotlin("jvm") version "2.3.0"
    kotlin("plugin.serialization") version "2.3.0"
}

group = "com.penguin"
version = "1.0.0"

repositories {
    mavenCentral()
}

val kotlinxSerializationVersion = "1.7.3"

subprojects {
    apply(plugin = "org.jetbrains.kotlin.jvm")

    repositories {
        google()
        mavenCentral()
    }
}