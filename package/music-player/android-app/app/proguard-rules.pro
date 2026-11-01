# R8 rules for the release build (isMinifyEnabled in build.gradle.kts).
#
# The Rust engine exports its entry points as Java_dev_monochromatic_musicplayer_NativeBridge_<method>, and the JVM
# finds them by that exact mangled name, so NativeBridge keeps its class name and the names of its native methods.
# The Rust side performs no JNI lookup of Kotlin classes, fields or methods (no FindClass or GetMethodID), so no
# other class needs a keep rule. includedescriptorclasses keeps the types in the native signatures too.
-keepclasseswithmembers,includedescriptorclasses class dev.monochromatic.musicplayer.NativeBridge {
    native <methods>;
}
