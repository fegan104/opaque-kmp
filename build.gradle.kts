plugins {
    alias(libs.plugins.android.kotlin.multiplatform.library) apply false
    alias(libs.plugins.kotlinMultiplatform) apply  false
    alias(libs.plugins.vanniktech.mavenPublish) apply false
    id("dev.oxikt.gradle.plugin") version "1.4.0" apply false
}

//uniffi {
//    generateFromLibrary {
//        packageName = "com.frankegan.opaque_kmp"
//    }
//}