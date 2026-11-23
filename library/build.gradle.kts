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

// Android host tests run the androidMain bindings on the host JVM, but dev.oxikt:runtime-android only
// ships liboxikt_runtime for Android ABIs. Borrow the host builds bundled in runtime-jvm so JNA can
// find them on the test classpath.
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

kotlin.sourceSets.getByName("androidHostTest").resources.srcDir(extractOxiktHostRuntime)

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
