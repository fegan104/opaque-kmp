import org.gradle.kotlin.dsl.support.serviceOf
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.android.kotlin.multiplatform.library)
    alias(libs.plugins.vanniktech.mavenPublish)
    alias(libs.plugins.oxikt)
}

group = "com.frankegan"
version = "0.0.1"

kotlin {
    jvm()
    android {
        namespace = "com.frankegan.opaque_kmp"
        compileSdk = libs.versions.android.compileSdk.get().toInt()
        minSdk = libs.versions.android.minSdk.get().toInt()

        withJava() // enable java compilation support
        withHostTestBuilder {}.configure {}
        withDeviceTestBuilder {
            sourceSetTreeName = "test"
        }.configure {
            instrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        }

        compilerOptions {
            jvmTarget = JvmTarget.JVM_17
        }
    }
    iosArm64()
    iosSimulatorArm64()

    sourceSets {
        commonMain.dependencies {
            //put your multiplatform dependencies here
        }

        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }

        getByName("androidDeviceTest").dependencies {
            implementation(libs.androidx.test.runner)
            implementation(libs.androidx.test.ext.junit)
        }
    }
}

// Android host tests run the androidMain bindings on the host JVM, which needs host builds of both
// libopaquekmp and liboxikt_runtime. Two oxikt 1.4.0 gaps break that:
// - dev.oxikt:runtime-android only ships liboxikt_runtime for Android ABIs, so borrow the host
//   builds bundled in dev.oxikt:runtime-jvm.
// - The plugin hands the crate's host library to AGP as a host-test Java resource, and AGP drops
//   every .so file from Java resources, so on Linux neither library reaches the tests.
// AGP also overrides additions to the test classpath, so JNA is pointed at both directories
// directly with jna.library.path instead.
val oxiktHostRuntime = configurations.create("oxiktHostRuntime") {
    isCanBeConsumed = false
    isTransitive = false
}

dependencies {
    oxiktHostRuntime(libs.oxikt.runtime.jvm)
}

val extractOxiktHostRuntime = tasks.register<Sync>("extractOxiktHostRuntime") {
    val archives = serviceOf<ArchiveOperations>()
    from(oxiktHostRuntime.elements.map { files -> files.map { archives.zipTree(it.asFile) } }) {
        include("linux-*/**", "darwin-*/**", "win32-*/**")
    }
    into(layout.buildDirectory.dir("oxiktHostRuntime"))
}

// JNA's directory name for this machine, e.g. linux-x86-64 or darwin-aarch64.
val jnaHostPlatform: String = run {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val osPrefix = when {
        os.startsWith("mac") -> "darwin"
        os.startsWith("windows") -> "win32"
        else -> "linux"
    }
    val archSuffix = if (arch == "aarch64" || arch == "arm64") "aarch64" else "x86-64"
    "$osPrefix-$archSuffix"
}

tasks.withType<Test>().configureEach {
    if (name == "testAndroidHostTest") {
        val crateLibraries = tasks.named("mergeUniffiAndroidHostTestResources")
        dependsOn(crateLibraries, extractOxiktHostRuntime)
        val libraryDirs = listOf(
            layout.buildDirectory.dir("generated/resources/mergeUniffiAndroidHostTestResources/$jnaHostPlatform"),
            layout.buildDirectory.dir("oxiktHostRuntime/$jnaHostPlatform"),
        ).map { it.get().asFile.absolutePath }
        systemProperty("jna.library.path", libraryDirs.joinToString(File.pathSeparator))
    }
}

uniffi {
    generateFromLibrary {
        packageName = "com.frankegan.opaque_kmp"
    }
}

mavenPublishing {
    publishToMavenCentral()

    signAllPublications()

    coordinates(group.toString(), "opaque-kmp", version.toString())

    pom {
        name = "My library"
        description = "A library."
        inceptionYear = "2024"
        url = "https://github.com/kotlin/multiplatform-library-template/"
        licenses {
            license {
                name = "XXX"
                url = "YYY"
                distribution = "ZZZ"
            }
        }
        developers {
            developer {
                id = "XXX"
                name = "YYY"
                url = "ZZZ"
            }
        }
        scm {
            url = "XXX"
            connection = "YYY"
            developerConnection = "ZZZ"
        }
    }
}

tasks.withType<Test>().configureEach {
    testLogging {
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
        showCauses = true
        showStackTraces = true
    }
}
