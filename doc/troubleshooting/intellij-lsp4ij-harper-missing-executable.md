# LSP4IJ 0.21.0 fails to start Harper when the configured executable is missing

## Symptom

IntelliJ IDEA build `IU-263.7307.200` logged this exception while rendering an editor diff.

```text
com.intellij.execution.process.ProcessNotCreatedException: Cannot run program "/home/user/.lsp4ij/lsp/harper-ls/harper-ls": Exec failed, error: 2 (No such file or directory)
```

## Root cause

Both saved Harper definitions in `UserDefinedLanguageServerSettings.xml` used the same command.
One definition covered general documents and one covered `AGENTS.md` and `CLAUDE.md`.

```text
$USER_HOME$/.lsp4ij/lsp/harper-ls/harper-ls --stdio
```

The entire `$HOME/.lsp4ij` directory was absent.
During the repair, both saved definitions showed `installAlreadyDone` set to `true`
and `executeOnStartServer` set to `false`.
How the directory came to be absent was undetermined.

The bundled template also disables installation at startup.

```json
"executeOnStartServer": false,
```

[LSP4IJ `src/main/resources/templates/lsp/harper-ls/installer.json:4`][installer]

The stack trace reached the process launch at `OSProcessStreamConnectionProvider.java:114`.
The provider passes the configured command to IntelliJ and wraps the launch exception at line `130`.

```java
processHandler = new OSProcessHandler(commandLine) {
```

```java
} catch (Exception e) {
    throw new CannotStartProcessException(e);
}
```

[LSP4IJ `src/main/java/com/redhat/devtools/lsp4ij/server/OSProcessStreamConnectionProvider.java:109-130`][launch]

## Verification

Run the configured executable directly to test the path.

```sh
# Terminal
"$HOME/.lsp4ij/lsp/harper-ls/harper-ls" --version
```

- **Missing path.** Before repair, launching the configured executable with `--version` returned `ENOENT`.
- **System binary.** `/usr/bin/harper-ls --version` printed `harper-ls 3.2.0`.
  `pacman --query --owns /usr/bin/harper-ls` identified package `harper 3.2.0-1.1`.
- **Repaired path.** The configured path printed the same version.
  A separate `--stdio` session initialized, published article and spelling diagnostics for `This is an sentance.`,
  and exited successfully with empty stderr after LSP shutdown.
- **Live IntelliJ session.** After opening Markdown files, `ps` showed the configured `harper-ls --stdio` command
  running as a child of the existing IntelliJ process. `/proc/<pid>/exe` resolved to `/usr/bin/harper-ls`.
  Live verification covered one IntelliJ child process.

## Verified workaround

When `/usr/bin/harper-ls --version` works and the configured executable is absent,
create the expected path as a symbolic link to the system binary.

```sh
# Terminal
mkdir --parents "$HOME/.lsp4ij/lsp/harper-ls" &&
  ln --symbolic --no-target-directory /usr/bin/harper-ls "$HOME/.lsp4ij/lsp/harper-ls/harper-ls"
```

The link serves both definitions. Harper updates now follow the system package,
and removing that package would leave a dangling link.

## What does not work

Invoking the launch command while the configured path is absent still fails when `/usr/bin/harper-ls` exists.
The saved command supplies an absolute path after home-directory expansion, so `PATH` lookup cannot supply a substitute.

## Upstream filing decision

The evidence supports a local repair. No filing artifact is needed.
Searches for `harper-ls` and `"No such file or directory" harper` in the LSP4IJ tracker found a prior
Harper installer report on macOS. The local incident supplied no additional evidence about that installer failure.
[LSP4IJ issue #1494][prior-report]

1.  Fault attribution stops at the missing local path. Its removal history is unknown.
2.  An upstream fix would require evidence of a defect in a supported installation or launch path.
3.  The bundled template covers launching Harper. The observed repair uses the existing launch command.
4.  Contribution eligibility was outside the scope of the local repair.
5.  The tracker search showed prior work on the installer. Maintainer interest in this local incident was undetermined.
6.  The repair was verified in the running IDE. An upstream prototype depends on an attributed defect.

[installer]: https://github.com/redhat-developer/lsp4ij/blob/b30f69dd1285f201d53ce360b7fad0bba96f0d58/src/main/resources/templates/lsp/harper-ls/installer.json#L1-L9
[launch]: https://github.com/redhat-developer/lsp4ij/blob/b30f69dd1285f201d53ce360b7fad0bba96f0d58/src/main/java/com/redhat/devtools/lsp4ij/server/OSProcessStreamConnectionProvider.java#L109-L130
[prior-report]: https://github.com/redhat-developer/lsp4ij/issues/1494#:~:text=Harper%20Language%20Server%3A%20Cannot%20start%20server%20Android%20studio
