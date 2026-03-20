plugins {
    kotlin("jvm")
    kotlin("plugin.serialization")
    id("org.jetbrains.compose") version "1.10.3"
    id("org.jetbrains.kotlin.plugin.compose") version "2.3.0"
    application
}

group = "com.penguin"
version = "1.0.0"

val kotlinxSerializationVersion = "1.7.3"

dependencies {
    implementation(project(":core"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:$kotlinxSerializationVersion")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core-jvm:1.10.2")
    implementation("org.jetbrains.kotlinx:kotlinx-cli:0.3.6")
    implementation("com.jakewharton.mosaic:mosaic-runtime:0.18.0")
    testImplementation(kotlin("test"))
}

application {
    mainClass.set("com.penguin.downloader.tui.MainKt")
}

tasks.jar {
    archiveBaseName.set("penguin-downloader-tui")
    manifest {
        attributes["Main-Class"] = "com.penguin.downloader.tui.MainKt"
    }
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
}

tasks.test {
    useJUnitPlatform()
}