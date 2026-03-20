plugins {
    kotlin("jvm")
    kotlin("plugin.serialization")
    application
}

group = "com.penguin"
version = rootProject.version

val kotlinxSerializationVersion = "1.7.3"

dependencies {
    implementation(project(":core"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:$kotlinxSerializationVersion")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core-jvm:1.10.2")
    implementation("org.jetbrains.kotlinx:kotlinx-cli:0.3.6")
    testImplementation(kotlin("test"))
}

application {
    mainClass.set("com.penguin.downloader.cli.MainKt")
}

tasks.jar {
    archiveBaseName.set("penguin-downloader-cli")
    manifest {
        attributes["Main-Class"] = "com.penguin.downloader.cli.MainKt"
    }
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
}

tasks.test {
    useJUnitPlatform()
}
