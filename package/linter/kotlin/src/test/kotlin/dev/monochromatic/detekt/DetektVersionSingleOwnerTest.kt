package dev.monochromatic.detekt

import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import java.io.File

/**
 * Verifies the invariant the detekt version needs: one spelling, the `detektVersion`
 * entry in `gradle.properties`, consumed by the Gradle build and by the Maven Central
 * verification task rather than repeated.
 *
 * Two spellings drift apart silently: bumping one leaves the other resolving a stale
 * detekt CLI that no test exercises.
 */
class DetektVersionSingleOwnerTest {
    /** Gradle property that holds the only detekt version spelling. */
    private val propertyName = "detektVersion"

    /**
     * Files that define the build or its tasks, so each may only consume the owned
     * version. `task.ts` is listed beside `mise.toml` because either may hold the
     * verification task while the package moves its task bodies between them.
     */
    private val consumerFiles = listOf("build.gradle.kts", "mise.toml", "task.ts")

    /**
     * Declares the detekt version in `gradle.properties`, the file both the Gradle
     * build and the verification task can read.
     */
    @Test
    fun declaresTheDetektVersionInGradleProperties() {
        val declared = readProperties()[propertyName]
        assertTrue(
            declared != null && declared.isNotBlank(),
            "gradle.properties must declare $propertyName",
        )
    }

    /**
     * Repeats the version literal in no other file, so the owned entry stays the only
     * place a bump is needed and no consumer can pin a different release unnoticed.
     */
    @Test
    fun repeatsTheDetektVersionLiteralInNoOtherFile() {
        val declared = ownedDetektVersion()
        for (file in existingConsumerFiles()) {
            assertTrue(
                !read(file).contains(declared),
                "$file must consume $propertyName instead of repeating $declared",
            )
        }
    }

    /**
     * Interpolates the owned property wherever a detekt CLI is resolved, so the CLI the
     * published rules are verified through cannot lag behind the version the rules are
     * built and tested against.
     */
    @Test
    fun interpolatesTheOwnedDetektVersionInTheCentralVerification() {
        val coordinates = existingConsumerFiles().flatMap { file ->
            Regex("""dev\.detekt:detekt-cli:([^'"\s]+)""")
                .findAll(read(file))
                .map { match -> file to match.groupValues[1] }
                .toList()
        }
        assertTrue(coordinates.isNotEmpty(), "the verification task must resolve a detekt CLI")
        // A consumer names the property in its own language's syntax: `$name` in Kotlin,
        // `${name}` in the JavaScript task bodies.
        val interpolations = setOf("\$$propertyName", "\${$propertyName}")
        for ((file, version) in coordinates) {
            assertTrue(
                version in interpolations,
                "$file must interpolate $propertyName instead of pinning its own detekt CLI version, got: $version",
            )
        }
    }

    /** Returns the owned detekt version, failing when `gradle.properties` lacks it. */
    private fun ownedDetektVersion(): String {
        val declared = readProperties()[propertyName]
        if (declared.isNullOrBlank()) {
            error("gradle.properties must declare $propertyName")
        }
        return declared
    }

    /** Returns the consumer files that exist, since a package task file may be absent. */
    private fun existingConsumerFiles(): List<String> =
        consumerFiles.filter { name -> File(packageRoot(), name).isFile }

    /** Reads one package file by name, naming the resolved path when it is missing. */
    private fun read(name: String): String {
        val file = File(packageRoot(), name)
        assertTrue(file.isFile, "missing package file: ${file.path}")
        return file.readText()
    }

    /** Parses the package's `gradle.properties` into its key and value entries. */
    private fun readProperties(): Map<String, String> = read("gradle.properties")
        .lineSequence()
        .filter { line -> line.isNotBlank() && !line.trimStart().startsWith("#") }
        .mapNotNull { line ->
            val separator = line.indexOf('=')
            if (separator < 0) {
                null
            } else {
                line.substring(0, separator).trim() to line.substring(separator + 1).trim()
            }
        }
        .toMap()

    /**
     * Walks up from the Gradle test working directory to the folder holding both
     * `mise.toml` and `gradle.properties`.
     */
    private fun packageRoot(): File {
        val start = File(System.getProperty("user.dir")).absoluteFile
        val found = generateSequence(start) { directory -> directory.parentFile }
            .firstOrNull { directory ->
                File(directory, "mise.toml").isFile && File(directory, "gradle.properties").isFile
            }
        if (found == null) {
            error("no package folder with mise.toml and gradle.properties above ${start.path}")
        }
        return found
    }
}
