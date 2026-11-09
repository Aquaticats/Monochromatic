# IntelliJ IDEA 2026.2 EAP shows red Gradle Kotlin DSL imports when the repo is opened through a symlinked home path

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

IntelliJ IDEA 2026.2 EAP Build `IU-262.8117.19` can keep Gradle Kotlin DSL script
classpath state on a different filesystem spelling from the editor file when this repo is opened through
`/home/user/Monochromatic` while Gradle and VCS state canonicalize to
`/var/home/user/Monochromatic`.
In that split state,
 `package/linter/kotlin/build.gradle.kts` can show red imports even though Gradle
CLI compilation succeeds.

## Symptom

The user-visible symptoms in this repo were:

- `package/linter/kotlin/build.gradle.kts` showed
  `Unresolved reference 'CommandLineArgumentProvider'` for
  `org.gradle.process.CommandLineArgumentProvider`.
- After the Gradle API import recovered,
   the same file still showed red
  `org.jetbrains.kotlin` in `import org.jetbrains.kotlin.gradle.dsl.JvmTarget`.
- The Gradle tool window showed duplicate `kotlin-linter` entries during part of the recovery.
- Running `idea invalidateCaches --help` did not print help.
  It executed cache invalidation,
   and IDEA restored default configuration.
  The plugin install path then contained only part of the previous plugin set until restored from backup.

The command-line build was not broken.
This command passed before and after the IDE recovery:

```shell
# /var/home/user/Monochromatic
mise run //package/linter/kotlin:lint
```

```text
BUILD SUCCESSFUL in 3s
4 actionable tasks: 4 up-to-date
```

## Root cause

This is not a custom user-created path alias.
The host reports itself as Bazzite.
Bazzite's manual partitioning documentation lists `/var/home` as the home subvolume mount point,
 and this
Bazzite host presents `/home` as a symlink into `/var/home`:

```shell
grep --extended-regexp '^(NAME|ID|VARIANT|PRETTY_NAME)=' /etc/os-release
readlink /home
stat --format='%N' /home
```

```text
NAME="Bazzite"
ID=bazzite
PRETTY_NAME="Bazzite"
VARIANT="Kinoite"
var/home
'/home' -> 'var/home'
```

The official Bazzite page used for that check is:

```text
https://docs.bazzite.gg/General/Installation_Guide/manual_partitioning/
```

That means these platform-normal path strings name the same files on this host but are not the same string:

```text
/home/user/Monochromatic
/var/home/user/Monochromatic
```

IDEA exposed that split in its own log.
Before the final fix,
 the editor was opened on the symlink spelling while Gradle and VCS state used the
canonical spelling:

```text
InlinePromptListener is installed to editor=EditorImpl[
  file:///home/user/Monochromatic/package/linter/kotlin/build.gradle.kts
]
Symlink mapping for VCS is used,
  original file: file:///home/user/Monochromatic/...
  canonical file: file:///var/home/user/Monochromatic/...
```

After reopening the root project with the canonical spelling,
 the editor and Gradle sync used the same path:

```text
Opening existing project with .idea at /var/home/user/Monochromatic
Gradle project sync
  phase = SCRIPT_MODEL_PHASE
  projectPath = /var/home/user/Monochromatic/package/linter/kotlin
InlinePromptListener is installed to editor=EditorImpl[
  file:///var/home/user/Monochromatic/package/linter/kotlin/build.gradle.kts
]
```

The IntelliJ VCS symlink resolver explicitly canonicalizes symlinked virtual files and logs when it does so.
The public source trace used `JetBrains/intellij-community@f10b97f07f9add4ab0bfe55e16dce1aa37581d3c`.
That is a current upstream snapshot,
 not a proven exact source match for installed build `IU-262.8117.19`.
In that snapshot,
 `platform/vcs-impl/src/com/intellij/vcs/DefaultVcsSymlinkResolver.java:41-45` returns a
canonical file when one exists:

```java
// platform/vcs-impl/src/com/intellij/vcs/DefaultVcsSymlinkResolver.java:41-45
public @Nullable VirtualFile resolveSymlink(@NotNull VirtualFile file) {
  VirtualFile canonicalFile = resolveFile(file);
  if (canonicalFile != null) {
    logSymlinkMappingWasUsed(file, canonicalFile);
  }
```

The same source file logs the exact message seen in the IDEA log:

```java
// platform/vcs-impl/src/com/intellij/vcs/DefaultVcsSymlinkResolver.java:75-78
private void logSymlinkMappingWasUsed(@NotNull VirtualFile file, @NotNull VirtualFile canonicalFile) {
  if (mySymlinkMappingWasUsed) return;
  mySymlinkMappingWasUsed = true;
  LOG.info("Symlink mapping for VCS is used, original file: " + file + ", canonical file: " + canonicalFile);
```

The project opener also logs and operates on the `Path` it was asked to open.
`platform/platform-impl/src/com/intellij/ide/impl/ProjectUtil.kt:187-190` reuses an already opened project for
that path when it finds one:

```kotlin
// platform/platform-impl/src/com/intellij/ide/impl/ProjectUtil.kt:187-190
suspend fun openOrImportAsync(file: Path, options: OpenProjectTask = OpenProjectTask()): Project? {
  if (!options.forceOpenInNewFrame) {
    findAndFocusExistingProjectForPath(file)?.let {
      LOG.info("Reusing already opened project $file")
```

When the path is a valid project directory,
 `ProjectUtil.kt:224-230` opens that path as an existing `.idea`
project:

```kotlin
// platform/platform-impl/src/com/intellij/ide/impl/ProjectUtil.kt:224,226-230
LOG.info("Opening existing project with .idea at $file")
val options = options.copy(
  runConfigurators = true,
  projectRootDir = descriptor.historicalProjectBasePath,
)
return (serviceAsync<ProjectManager>() as ProjectManagerEx).openProjectAsync(file, options)
```

The red imports were path-dependent IDE script-classpath symptoms.
The source path is:

```text
plugins/gradle/tooling-extension-impl/src/com/intellij/gradle/toolingExtension/impl/model/
buildScriptClasspathModel/GradleBuildScriptClasspathModelProvider.java:18-20
```

That source places build-script classpath collection in `SCRIPT_MODEL_PHASE`:

```java
// plugins/gradle/tooling-extension-impl/src/com/intellij/gradle/toolingExtension/impl/model/
// buildScriptClasspathModel/GradleBuildScriptClasspathModelProvider.java:18-20
@Override
public GradleModelFetchPhase getPhase() {
  return GradleModelFetchPhase.SCRIPT_MODEL_PHASE;
```

Kotlin script definitions then derive definition classes and classpaths from project state and module order
entries.
The source path is:

```text
plugins/kotlin/base/scripting/scripting.k2/src/org/jetbrains/kotlin/idea/core/script/k2/definitions/
DefinitionFromDependenciesProvider.kt:47-52
```

That source combines explicit,
 discovered,
 and guessed classpath values:

```kotlin
// plugins/kotlin/base/scripting/scripting.k2/src/org/jetbrains/kotlin/idea/core/script/k2/definitions/
// DefinitionFromDependenciesProvider.kt:47-52
override fun getDefinitionsClassPath(): Iterable<File> {
    val settings = ScriptDefinitionSettingsStateComponent.getInstance(project).state
    val explicitClasspath = settings.parsedClasspath
    val discoveredClasspath = ScriptTemplatesFromDependenciesCache.getOrDiscover(project).classpath
    val autoResolvedClasspath = tryToGuessClasspath(settings.parsedClassNames)
    return (explicitClasspath + discoveredClasspath + autoResolvedClasspath).distinct().map { File(it) }
```

The same file reads class roots from order entries at `DefinitionFromDependenciesProvider.kt:113-130`.
That matches the observed behavior:
 when the editor file and imported Gradle model used different path strings,
script classpath resolution could stay stale for the editor even while Gradle itself compiled the script.

## What fixed it

Use the canonical `/var/home/user/Monochromatic` spelling everywhere IDEA opens this repo.
The successful recovery sequence was:

```shell
# /var/home/user/Monochromatic
rm --recursive --force package/linter/kotlin/.idea
idea /var/home/user/Monochromatic
```

Then reload the Gradle project from IDEA.
Let IDEA own the Gradle module metadata it writes under `.idea/`.
Do not hand-add duplicate module entries for `package/linter/kotlin`.

The committed project metadata now keeps the nested Gradle build linked from the root project:

```xml
<!-- .idea/gradle.xml -->
<option name="externalProjectPath" value="$PROJECT_DIR$/packages/linter/kotlin" />
```

The final VCS mapping also records the canonical root,
 which is the path spelling IDEA used after recovery:

```xml
<!-- .idea/vcs.xml -->
<mapping directory="/var/home/user/Monochromatic" vcs="Git" />
```

The user confirmed the `org.jetbrains.kotlin` import was no longer red after the canonical reopen.

## What did not fix it

Do not replace source imports to work around this symptom before proving the IDE is healthy.
A temporary worktree tested source-side alternatives:

- Removing `JvmTarget` made Gradle report a Kotlin and Java JVM target mismatch.
- Adding `jvmToolchain(17)` failed on this machine because no JDK 17 installation was available.
- A reflection-based workaround compiled,
   but it only hid the IDE issue and made the build script worse.

Do not open the nested package as its own IDEA project while debugging this root project.
That created `package/linter/kotlin/.idea` and duplicate Gradle entries.
Remove that nested `.idea` directory and link the package Gradle build from the root project instead.

`package/linter/kotlin/.idea` is not in the working tree and no commit in this repository carries
it.

Do not use `idea invalidateCaches --help` to inspect the command.
In this environment it executed cache invalidation rather than printing help,
 then IDEA restored default settings.
If that happens again,
 treat it as configuration recovery,
 not as a Kotlin or Gradle problem.

## Plugin recovery after accidental cache invalidation

The active plugin directory for this install was:

```text
/var/home/user/.local/share/JetBrains/IntelliJIdea2026.2
```

The configuration backup lived at:

```text
/var/home/user/.config/JetBrains/IntelliJIdea2026.2-backup/2026-06-18-14-44
```

Copying configuration files alone did not restore plugins,
 because IDEA loads plugins from the active plugin
path above.
The successful recovery copied plugin directories from the backup into the active plugin path while IDEA was
closed.
After recovery,
 this check found 56 plugin directories and confirmed the expected plugins existed:

```shell
# /var/home/user/Monochromatic
find /var/home/user/.local/share/JetBrains/IntelliJIdea2026.2 -mindepth 1 -maxdepth 1 -type d | wc --lines
find /var/home/user/.local/share/JetBrains/IntelliJIdea2026.2 -maxdepth 1 \
  \( -name intellij-rust -o -name pkl-intellij -o -name lsp4ij \) -print
```

Do not touch `/var/home/user/.config/JetBrains` or `/var/home/user/.cache/JetBrains` again without explicit
approval.
Snapshot first if recovery is approved.

## Verification checklist

After recovering IDEA,
 verify the repo and the nested Gradle build from the command line:

```shell
# /var/home/user/Monochromatic
xmllint --noout .idea/vcs.xml .idea/gradle.xml
mise run //package/linter/kotlin:lint
git status --short --untracked-files=all
```

Expected results:

- `xmllint` exits successfully.
- Gradle reports `BUILD SUCCESSFUL`.
- `git status --short --untracked-files=all` is clean after committing the metadata and this document.

Also verify the user boundary in IDEA:

- Open `/var/home/user/Monochromatic`,
   not `/home/user/Monochromatic`.
- Open `package/linter/kotlin/build.gradle.kts`.
- Confirm `org.gradle.process.CommandLineArgumentProvider` and
  `org.jetbrains.kotlin.gradle.dsl.JvmTarget` are not red.
- Confirm the Gradle tool window shows the linked package build once.

## Upstream filing decision

The corrected decision is:
 this is upstream-worthy behavior,
 not merely local path hygiene.
The user-facing workaround is still to open the project through `/var/home/user/Monochromatic`,
 but Bazzite
provides `/home -> /var/home`,
 so opening the repo through `/home/user/Monochromatic` is a normal platform path.
It is surprising for that normal path spelling to make Gradle Kotlin DSL imports red while the Gradle CLI build
succeeds.

Do not post the draft below as-is from an automated session.
It is a YouTrack draft,
 not a GitHub issue draft,
 because JetBrains has GitHub issues disabled for
`JetBrains/intellij-community` and `CONTRIBUTING.md` asks contributors to use YouTrack tickets.
It is marked not-ready only because this troubleshooting policy requires a prototyped upstream fix before a fully
fileable artifact.
A human may still decide that a bug-only YouTrack report is appropriate.

### Out-of-scope check

The local `.out-of-scope/` exemption directory had no IntelliJ,
 JetBrains,
 Kotlin,
 Gradle,
 or Bazzite entry for
this case.
Files checked included:

```text
.out-of-scope/bun-install.md
.out-of-scope/cargo-workspace.md
.out-of-scope/claude-code-upstream-bugs.md
.out-of-scope/codex-harness.md
.out-of-scope/jsr.md
.out-of-scope/lightningcss.md
.out-of-scope/low-impact-typescript-formatting.md
.out-of-scope/module-es-monolith.md
.out-of-scope/pi-gpt55-long-context.md
.out-of-scope/terminal-title-fork-parity-tests.md
.out-of-scope/typescript-project-references.md
```

### Duplicate search

GitHub issue search is not the right primary tracker for this repository.
This command showed GitHub issues are disabled:

```shell
gh repo view JetBrains/intellij-community \
  --json url,isArchived,hasIssuesEnabled,viewerPermission,defaultBranchRef
```

Relevant output:

```json
{
  "hasIssuesEnabled": false,
  "isArchived": false,
  "url": "https://github.com/JetBrains/intellij-community",
  "viewerPermission": "READ"
}
```

GitHub pull request searches for these terms returned empty arrays:

```shell
gh search prs 'symlink Gradle Kotlin DSL unresolved reference' \
  --repo JetBrains/intellij-community --state open
gh search prs 'symlink Gradle Kotlin DSL unresolved reference' \
  --repo JetBrains/intellij-community --state closed
gh search prs '"Symlink mapping for VCS" Gradle' \
  --repo JetBrains/intellij-community --state open
gh search prs '"Symlink mapping for VCS" Gradle' \
  --repo JetBrains/intellij-community --state closed
```

YouTrack API searches found related but non-duplicate tickets:

```shell
python - <<'PY'
import json, urllib.parse, urllib.request
queries = [
    'Gradle Kotlin DSL symlink unresolved reference',
    'Bazzite /home /var/home IntelliJ',
    'Symlink mapping for VCS Gradle',
    'Gradle Kotlin DSL unresolved reference ignored .gradle',
]
for query in queries:
    params = urllib.parse.urlencode({
        'query': query,
        'fields': 'idReadable,summary,resolved',
        '$top': '8',
    })
    url = f'https://youtrack.jetbrains.com/api/issues?{params}'
    req = urllib.request.Request(url, headers={'Accept': 'application/json'})
    with urllib.request.urlopen(req, timeout=20) as response:
        print(query, json.load(response))
PY
```

Closest hits were read through their descriptions and public comments via the YouTrack API:

- `KTIJ-768`,
   open,
   Gradle Kotlin DSL unresolved references when `.gradle` is ignored.
  This is the same symptom class,
   but a different trigger.
  Its comments discuss `.gradle` ignoring,
   config and system directory resets,
   and persistent unresolved
  `build.gradle.kts` symbols.
- `IJPL-222918`,
   open,
   duplicated changed files for a Git symlinked project.
  This is a symlink path-identity issue,
   but in VCS change display,
   not Gradle Kotlin DSL script classpath.
  Its comments mention duplicate Git views and 2025-era symlink regressions.
- `IJPL-80700`,
   resolved,
   symlinked directory treated as a separate Git root.
  This is a related VCS path-identity issue,
   not this Gradle Kotlin DSL red-import behavior.
  Its comments focus on duplicate VCS roots and version-control mapping.
- `IDEA-367587`,
   resolved,
   WSL Gradle daemon toolchain import failure involving SDKMAN symlinks.
  This is Gradle plus symlink-adjacent,
   but it is a WSL toolchain failure,
   not Bazzite `/home` aliasing.
  Its comments focus on WSL Gradle JDK/toolchain import failures.

No searched ticket matched this exact combination:
Bazzite or Fedora Atomic `/home -> /var/home`,
 IDEA opened through `/home`,
 VCS or Gradle state canonicalized to
`/var/home`,
 and valid Gradle Kotlin DSL imports shown red while CLI Gradle succeeds.

### Constraint check

1.  **Is it really upstream's fault?
    **
    Yes.
    Bazzite supplies the `/home -> /var/home` alias,
     and IDEA owns the project-open,
     VFS,
     VCS,
     Gradle sync,
     and
    Kotlin script-classpath state that diverged across those path spellings.
    Gradle CLI compiled the script successfully,
     so the failure was IDE code insight state,
     not Gradle source.

2.  **Can upstream fix it?
    **
    Yes.
    Upstream controls `DefaultVcsSymlinkResolver`,
     `ProjectUtil.openOrImportAsync`,
     Gradle script model import,
    and Kotlin script definition classpath resolution.
    A valid fix could ensure those systems use one project-root identity when a project is opened through a
    symlinked path.

3.  **Are they supporting this use case?
    **
    Yes.
    IntelliJ IDEA supports Linux projects,
     Gradle Kotlin DSL,
     and VCS symlink resolution.
    The source already contains an explicit VCS symlink resolver and Gradle build-script classpath provider.
    A platform-normal home-directory symlink should not make valid Gradle imports red.

4.  **Would the repo welcome our contribution?
    **
    Yes,
     with JetBrains' process constraints.
    `CONTRIBUTING.md` says bug fixes are preferred,
     most accepted contributions are fixes for reproducible issues,
    tests should supply fixes if possible,
     and contributors should have or create a YouTrack ticket.
    GitHub issues are disabled,
     so YouTrack is the right filing surface.
    No policy banning AI-assisted investigation was found in `CONTRIBUTING.md`,
     `README.md`,
     repository metadata,
    or the searched tracker results.

5.  **Will they likely fix it?
    **
    Soft yes.
    No matching duplicate or won't-fix signal was found.
    Related symlink path-identity issues exist,
     including open `IJPL-222918` and resolved `IJPL-80700`,
     which shows
    the general class is within JetBrains' tracker scope.

6.  **Have we prototyped a minimal fix compatible with their architecture?
    **
    No.
    Prototype attempt recorded:
     a fresh source clone was created at
    `/tmp/agent/intellij-symlink-prototype-FlHf0uP1`,
     with origin
    `https://github.com/JetBrains/intellij-community.git` and commit
    `f10b97f07f9add4ab0bfe55e16dce1aa37581d3c`.
    The trace found the relevant boundary spans project opening,
     VCS symlink resolution,
     Gradle script model
    import,
     and Kotlin script definition classpath resolution.
    A patch that simply canonicalizes every opened project path would be speculative,
     because some users may
    intentionally open symlinked roots and expect that spelling to remain meaningful.
    A safe prototype needs a small integration reproduction that opens a Gradle Kotlin DSL project through the
    symlink spelling,
     lets script models import,
     and asserts the editor-visible script classpath uses the same
    root identity as the imported Gradle model.
    That verification would require running IDEA or an IntelliJ integration test harness against IDE config and
    cache state.
    This session must not touch `/var/home/user/.config/JetBrains` or `/var/home/user/.cache/JetBrains` again
    without explicit approval,
     so the prototype was abandoned before writing an unverified patch.

### YouTrack draft, do not file as-is

~~~md
Title:
Gradle Kotlin DSL imports turn red when a Bazzite project is opened through /home symlink

Project:
IntelliJ IDEA, Kotlin IDE, or IntelliJ Platform Gradle integration

Description:
On Bazzite Kinoite, `/home` is a platform-provided symlink to `/var/home`:

```text
NAME="Bazzite"
ID=bazzite
VARIANT="Kinoite"
readlink /home -> var/home
stat /home -> '/home' -> 'var/home'
```

Opening a Gradle Kotlin DSL project through `/home/user/Monochromatic` caused valid imports in
`package/linter/kotlin/build.gradle.kts` to be marked unresolved in the editor, even though the Gradle CLI build
succeeded.

Observed red imports:

```kotlin
import org.gradle.process.CommandLineArgumentProvider
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
```

Command-line verification from the same project succeeded:

```shell
mise run //package/linter/kotlin:lint
```

```text
BUILD SUCCESSFUL
```

Relevant IDEA log evidence before recovery showed the editor using `/home` while VCS state canonicalized to
`/var/home`:

```text
InlinePromptListener is installed to editor=EditorImpl[
  file:///home/user/Monochromatic/package/linter/kotlin/build.gradle.kts
]
Symlink mapping for VCS is used,
  original file: file:///home/user/Monochromatic/...
  canonical file: file:///var/home/user/Monochromatic/...
```

After closing the nested package project, deleting `package/linter/kotlin/.idea`, and reopening the root project
as `/var/home/user/Monochromatic`, the same file no longer showed the Kotlin import as red.
The log then showed the project and editor using the canonical root:

```text
Opening existing project with .idea at /var/home/user/Monochromatic
Gradle project sync
  phase = SCRIPT_MODEL_PHASE
  projectPath = /var/home/user/Monochromatic/package/linter/kotlin
InlinePromptListener is installed to editor=EditorImpl[
  file:///var/home/user/Monochromatic/package/linter/kotlin/build.gradle.kts
]
```

Source trace:

- `platform/vcs-impl/src/com/intellij/vcs/DefaultVcsSymlinkResolver.java:41-45` resolves a symlinked
  `VirtualFile` to its canonical file.
- `DefaultVcsSymlinkResolver.java:75-78` logs `Symlink mapping for VCS is used`, matching the observed log.
- `platform/platform-impl/src/com/intellij/ide/impl/ProjectUtil.kt:187-190` reuses an already opened project for
  the path it was asked to open.
- `ProjectUtil.kt:224-230` opens a valid `.idea` project at the supplied path.
- `plugins/gradle/tooling-extension-impl/src/com/intellij/gradle/toolingExtension/impl/model/
  buildScriptClasspathModel/GradleBuildScriptClasspathModelProvider.java:18-20` puts build-script classpath
  collection in `SCRIPT_MODEL_PHASE`.
- `plugins/kotlin/base/scripting/scripting.k2/src/org/jetbrains/kotlin/idea/core/script/k2/definitions/
  DefinitionFromDependenciesProvider.kt:47-52` combines explicit, discovered, and guessed Kotlin script
  definition classpaths.

Expected result:
Opening through `/home/user/...` or `/var/home/user/...` should not change Gradle Kotlin DSL code insight for the
same physical project on Bazzite.

Actual result:
Opening through `/home/user/...` could leave Gradle Kotlin DSL imports unresolved in the editor while the CLI build
succeeded.

Workaround:
Open the project through the canonical `/var/home/user/...` path and let IDEA regenerate the Gradle link metadata.
For this repo, also remove an accidentally created nested `package/linter/kotlin/.idea` before reloading Gradle.

Suggested fix direction:
Audit path identity across project open, VCS symlink resolution, Gradle script model import, and Kotlin script
classpath resolution.
The fix should make the editor-visible Gradle Kotlin DSL script classpath and the imported Gradle model agree on
one project-root identity when the OS supplies a home-directory symlink.
It should not require Bazzite users to know that `/home` is a symlink before opening a project.
~~~
