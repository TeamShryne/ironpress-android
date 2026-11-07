plugins {
    id("com.android.library") version "8.7.3"
}

android {
    namespace = "com.teamshryne.ironpress"
    compileSdk = 35
    defaultConfig { minSdk = 23 }
    sourceSets["main"].jniLibs.srcDirs("src/main/jniLibs")
}
