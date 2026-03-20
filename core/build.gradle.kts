plugins {
    kotlin("jvm")
    kotlin("plugin.serialization")
}

group = "com.penguin"
version = "1.0.0"

repositories {
    mavenCentral()
}

val kotlinxSerializationVersion = "1.7.3"

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:$kotlinxSerializationVersion")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core-jvm:1.10.2")
    implementation("org.jetbrains.kotlinx:kotlinx-cli:0.3.6")
    implementation("net.jthink:jaudiotagger:3.0.1")
    implementation("org.slf4j:slf4j-simple:2.0.16")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("com.varabyte.kotter:kotter-jvm:1.1.2")
    implementation("com.google.zxing:core:3.5.3")
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}
