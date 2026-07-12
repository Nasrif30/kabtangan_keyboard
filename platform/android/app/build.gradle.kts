plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.rust.android.gradle)
}

android {
    namespace = "com.kabtangan.keyboard"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.kabtangan.keyboard"
        minSdk = 26          // Android 8.0 — covers ~95% of active devices
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            // Build the Rust core for all common ABI targets
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    buildFeatures {
        viewBinding = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

// Rust core build via cargo-ndk
cargo {
    module = "../../../core"   // Path to the Rust crate
    libname = "kabtangan_core"
    targets = listOf("arm", "arm64", "x86_64")
    profile = "release"
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    implementation(libs.androidx.preference.ktx)
}
