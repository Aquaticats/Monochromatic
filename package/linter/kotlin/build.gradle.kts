// What:     This import gives the build script Gradle's Maven publication record type.
//           An import makes a library name available without repeating its full package path.
// Why:      The publication block needs this type to describe one Maven component.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { MavenPublication } from "gradle";
// ```
import org.gradle.api.publish.maven.MavenPublication
// Import the task type that builds a documentation or source JAR archive.
import org.gradle.api.tasks.bundling.Jar
// Import the task type that builds the Central Publisher Portal ZIP archive.
import org.gradle.api.tasks.bundling.Zip
import org.gradle.process.CommandLineArgumentProvider
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

// Standalone Kotlin/JVM build for the monorepo-wide detekt rule set. It compiles the
// custom rules into a plugin jar and runs the detekt CLI over an arbitrary source
// tree, the Kotlin parallel of package/linter/monochromatic-lint (a standalone linter invoked by
// the root `lint:detekt` task over package/). detekt and the CLI resolve from
// mavenCentral; nothing here ships in any application.
plugins {
    kotlin("jvm") version "2.4.21"
    // What:     Backtick-delimited `maven-publish` applies Gradle's built-in Maven publication plugin.
    //           Backticks let Kotlin use a plugin name containing a hyphen.
    // Why:      The build needs standard POM generation and Maven repository layout staging.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plugins.apply("maven-publish");
    // ```
    `maven-publish`
    // Apply Gradle's built-in OpenPGP signing plugin to the Maven publication.
    signing
}

// detekt 2.0.x is the active upgrade target for the repo's custom Kotlin rule set.
val detektVersion = "2.0.0-alpha.6"

// Holds the detekt CLI and its dependencies, used as the classpath for the
// detektCheck runner. Kept separate from the compile classpath so the runner gets
// detekt-core (which detekt-api alone does not provide).
val detektCli: Configuration by configurations.creating

dependencies {
    // The rule compiles against detekt-api only; detekt-core provides it at analysis
    // time, so it must not be bundled into the plugin jar.
    compileOnly("dev.detekt:detekt-api:$detektVersion")
    detektCli("dev.detekt:detekt-cli:$detektVersion")
    testImplementation("dev.detekt:detekt-test:$detektVersion")
    testImplementation("dev.detekt:detekt-api:$detektVersion")
    testImplementation("org.junit.jupiter:junit-jupiter:6.1.3")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

// JVM 17 keeps Java and Kotlin targets consistent without a toolchain (only JDK 21
// is provisioned, so jvmToolchain(17) would force an unwanted JDK download).
java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    // What:     `withSourcesJar()` asks Gradle's Java component to publish a second JAR
    //           containing the Kotlin source files beside the compiled plugin JAR.
    // Why:      Maven Central requires a sources JAR for every non-POM component.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // publication.attachArchive("sources", sourceFiles);
    // ```
    withSourcesJar()
    // Add the required `javadoc` classifier JAR to the published Java component.
    withJavadocJar()
}

// What:     `tasks.named<Jar>(...) { ... }` selects the existing documentation archive task.
//           `<Jar>` is a generic type argument that checks the selected task's API.
// Why:      Kotlin has no Java Javadoc output, so README content makes the documentation JAR useful.
//
// In TS you'd write (pseudocode):
// ```ts
// tasks.named<JarTask>("javadocJar", task => task.from("README.md"));
// ```
tasks.named<Jar>("javadocJar") {
    from("README.md")
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

// What:     `val` creates a read-only Kotlin binding whose inferred value is a lazy Gradle directory provider.
//           Unlike `var`, the binding cannot be reassigned after creation.
// Why:      Publication and bundle tasks must share one staging location without duplicating its path.
//
// In TS you'd write (pseudocode):
// ```ts
// const centralStagingDirectory = buildDirectory.dir("central-staging");
// ```
val centralStagingDirectory = layout.buildDirectory.dir("central-staging")

publishing {
    publications {
        // What:     `create<MavenPublication>(...) { ... }` creates a typed Maven publication.
        //           Angle brackets pass `MavenPublication` as the generic record type.
        // Why:      Gradle can now generate the artifact, dependency metadata, and required POM.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // publications.create<MavenPublication>("mavenJava", publication => { /* metadata */ });
        // ```
        create<MavenPublication>("mavenJava") {
            artifactId = "detekt-rules"
            from(components["java"])
            pom {
                name = "Monochromatic detekt rules"
                description = "Detekt rules enforcing Monochromatic's Kotlin source standards."
                url = "https://github.com/Aquaticat/Monochromatic/tree/main/package/linter/kotlin"
                licenses {
                    license {
                        name = "GNU Lesser General Public License v3.0 or later"
                        url = "https://www.gnu.org/licenses/lgpl-3.0.html"
                        distribution = "repo"
                    }
                }
                developers {
                    developer {
                        id = "aquaticat"
                        name = "Aquaticat"
                        email = "an@aquati.cat"
                        url = "https://github.com/Aquaticat"
                    }
                }
                scm {
                    connection = "scm:git:https://github.com/Aquaticat/Monochromatic.git"
                    developerConnection = "scm:git:ssh://git@github.com/Aquaticat/Monochromatic.git"
                    url = "https://github.com/Aquaticat/Monochromatic"
                }
            }
        }
    }
    repositories {
        maven {
            name = "centralStaging"
            url = uri(centralStagingDirectory)
        }
    }
}

signing {
    // What:     `String?` is Kotlin's nullable string type, equivalent to a string-or-null union.
    //           `.orNull` extracts a configured Gradle property or produces null when absent.
    // Why:      CI can inject the armored private key while ordinary compile and test tasks remain usable locally.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const signingKey: string | null = properties.get("signingKey") ?? null;
    // ```
    val signingKey: String? = providers.gradleProperty("signingKey").orNull
    // Read the matching passphrase from a second nullable Gradle property.
    val signingPassword: String? = providers.gradleProperty("signingPassword").orNull
    useInMemoryPgpKeys(signingKey, signingPassword)
    sign(publishing.publications["mavenJava"])
}

// What:     `register<Delete>(...) { ... }` lazily creates a typed directory-cleaning task.
//           The generic argument provides Gradle's built-in delete-task API.
// Why:      Every bundle starts from an empty repository layout and cannot retain an older version.
//
// In TS you'd write (pseudocode):
// ```ts
// tasks.register<DeleteTask>("cleanCentralStaging", task => task.delete(stagingDirectory));
// ```
tasks.register<Delete>("cleanCentralStaging") {
    delete(centralStagingDirectory)
}

// Make repository staging wait until its destination has been cleared.
tasks.named("publishMavenJavaPublicationToCentralStagingRepository") {
    dependsOn("cleanCentralStaging")
}

// What:     `register<Zip>(...) { ... }` lazily creates a typed ZIP archive task.
//           The task packages files using paths relative to the staging directory.
// Why:      Sonatype's Portal API accepts one archive following Maven repository layout.
//
// In TS you'd write (pseudocode):
// ```ts
// tasks.register<ZipTask>("centralBundle", task => task.from(stagingDirectory));
// ```
tasks.register<Zip>("centralBundle") {
    group = "publishing"
    description = "Build a signed Maven Central Publisher Portal deployment bundle."
    dependsOn("publishMavenJavaPublicationToCentralStagingRepository")
    from(centralStagingDirectory)
    // Portal validation rejects Gradle's repository-maintenance metadata from a deployment bundle.
    exclude("**/maven-metadata.xml*")
    // Signatures are integrity files and do not need their own checksum sidecars.
    exclude("**/*.asc.md5", "**/*.asc.sha1", "**/*.asc.sha256", "**/*.asc.sha512")
    destinationDirectory.set(layout.buildDirectory.dir("central-bundle"))
    archiveFileName.set("detekt-rules-${project.version}-central-bundle.zip")
}

tasks.withType<Test>().configureEach {
    useJUnitPlatform()
}

// Run detekt with the custom require-kdoc rule set and detekt's default config
// over the tree named by the `detektInput` Gradle property (the root `lint:detekt`
// task passes the repo's package/ dir). Global --excludes drop build output,
// .gradle caches, build scripts (.kts), and test sources before parsing,
// mirroring how require-tsdoc skips .test.ts.
tasks.register<JavaExec>("detektCheck") {
    group = "verification"
    description = "Run detekt with the custom require-kdoc rule set over --input (detektInput property)."
    dependsOn(tasks.jar)
    classpath = detektCli
    mainClass.set("dev.detekt.cli.Main")

    val ruleJar = tasks.jar.flatMap { it.archiveFile }
    inputs.file(ruleJar)
    val inputPath = providers.gradleProperty("detektInput")
        .orElse(layout.projectDirectory.dir("src").asFile.absolutePath)
    val configFile = layout.projectDirectory.file("detekt.yml").asFile.absolutePath

    argumentProviders.add(
        CommandLineArgumentProvider {
            listOf(
                "--input", inputPath.get(),
                "--config", configFile,
                "--build-upon-default-config",
                "--plugins", ruleJar.get().asFile.absolutePath,
                "--excludes",
                "**/build/**,**/.gradle/**,**/node_modules/**,**/*.kts,**/src/test/**,**/src/androidTest/**",
            )
        },
    )
}
