import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("jvm") version "2.4.21"
    kotlin("plugin.serialization") version "2.4.21"
    `java-library`
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.12.0")
    testImplementation(kotlin("test"))
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
    withSourcesJar()
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_11
        freeCompilerArgs.add("-Xjdk-release=11")
    }
}

val library = System.mapLibraryName("esf_dogma_engine_jni")

fun hostPlatform(): String {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val osName = when {
        os.startsWith("linux") -> "linux"
        os.startsWith("mac") -> "macos"
        os.startsWith("windows") -> "windows"
        else -> error("unsupported OS: $os")
    }
    val archName = when (arch) {
        "amd64", "x86_64" -> "x86_64"
        "aarch64", "arm64" -> "aarch64"
        else -> error("unsupported architecture: $arch")
    }
    return "$osName-$archName"
}

val cargoBuild by tasks.registering(Exec::class) {
    workingDir = rootDir
    commandLine("cargo", "build", "--locked", "-p", "esf-kotlin")
}

val hostNatives by tasks.registering(Sync::class) {
    dependsOn(cargoBuild)
    from(rootDir.resolve("../../target/debug")) {
        include(library)
        into("fit/eveship/dogma/natives/${hostPlatform()}")
    }
    into(layout.buildDirectory.dir("natives"))
}

// -Pnatives=<dir> packs prebuilt libraries for every platform instead.
val prebuilt = providers.gradleProperty("natives").orNull

sourceSets.main {
    resources.srcDir(if (prebuilt != null) file(prebuilt) else hostNatives)
}

tasks.named<Jar>("sourcesJar") {
    exclude("fit/eveship/dogma/natives/**")
}

tasks.test {
    useJUnitPlatform()
}
