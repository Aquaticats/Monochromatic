package dev.monochromatic.detekt

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import java.io.File

/**
 * Verifies that the detekt version this rule set compiles against has exactly one
 * owner, the `detektVersion` entry in `gradle.properties`, and that both the Gradle
 * build and the Maven Central verification task in `mise.toml` read that entry.
 *
 * Two spellings drift apart silently: bumping one leaves the other resolving a
 * stale detekt CLI that no test exercises.
 */
class DetektVersionSingleOwnerTest {
    /** Gradle property that holds the only detekt version spelling. */
    private val propertyName = "detektVersion"

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
     * Resolves the build's detekt version through the Gradle property instead of a
     * string literal, so a version bump has one edit site.
     */
    @Test
    fun resolvesTheBuildDetektVersionFromTheGradleProperty() {
        val assignment = Regex("""val\s+$propertyName\b[^=\n]*=\s*(.+)""")
            .find(read("build.gradle.kts"))
            ?: error("build.gradle.kts must assign $propertyName")
        val rightHandSide = assignment.groupValues[1].trim()
        assertTrue(
            rightHandSide.contains("gradleProperty(\"$propertyName\")"),
            "build.gradle.kts must read $propertyName from the Gradle property, got: $rightHandSide",
        )
        assertTrue(
            !rightHandSide.startsWith("\""),
            "build.gradle.kts must not spell the detekt version as a literal, got: $rightHandSide",
        )
    }

    /**
     * Writes the verification task's detekt CLI coordinate as an interpolation of the
     * owned property, so the CLI it runs cannot lag behind the version the rules are
     * built and tested against.
     */
    @Test
    fun interpolatesTheOwnedDetektVersionInTheCentralVerification() {
        val resolvedVersions = Regex("""dev\.detekt:detekt-cli:([^'"\s]+)""")
            .findAll(read("mise.toml"))
            .map { match -> match.groupValues[1] }
            .toList()
        assertTrue(resolvedVersions.isNotEmpty(), "mise.toml must resolve a detekt CLI")
        val expected = "\${$propertyName}"
        for (version in resolvedVersions) {
            assertEquals(
                expected,
                version,
                "mise.toml must interpolate $propertyName instead of pinning its own detekt CLI version",
            )
        }
    }

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
