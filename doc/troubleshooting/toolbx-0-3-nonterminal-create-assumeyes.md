# Toolbx 0.3 non-terminal create without assumeyes refuses a required image download

## Symptom

On Bazzite 44 with `toolbox-0.3-4.fc44.x86_64`,
a managed process ran this while no default Toolbx image was present:

```sh
toolbox create firefox-installer
```

Toolbx wrote no standard output and exited with code `1` after this exact standard error:

```text
Error: image required to create Toolbx container.
Use option '--assumeyes' to download the image.
Run 'toolbox --help' for usage.
```

The same command can show a confirmation prompt in an interactive terminal.
The failure is specific to a missing image when Toolbx cannot use both standard input and standard output as terminal
devices.
It surfaced while testing a candidate fallback during preparation of
[`doc/runbook/firefox-esr-nightly-fedora-atomic.md`](../runbook/firefox-esr-nightly-fedora-atomic.md).
That fallback was rejected and does not appear in the runbook.

A separate package-list error appeared after fixing image creation.
The default Fedora 42 Toolbx rejected `gnupg2-verify` with:

```text
No match for argument: gnupg2-verify
```

Fedora 44 splits the host's `/usr/bin/gpgv` into `gnupg2-verify`,
but the Fedora 42 Toolbx already supplied `gpgv` through its installed packages.
The tested workaround therefore installed `gnupg2` without naming the release-specific split package.

## Root cause

The source investigation used `containers/toolbox` tag `0.3`,
commit `c66ddd12d9b7dd4b0ea16c2696ea6ed506180959`.
The clone's origin was `https://github.com/containers/toolbox.git`.
That tag matches the installed `toolbox --version` output,
`toolbox version 0.3`.

The earlier hypothesis that Bazzite failed to identify a default image was wrong.
Toolbx had resolved an image and reached the missing-image download decision.
The failure came from the deliberate non-terminal guard before any pull.

`src/cmd/create.go:709` to `src/cmd/create.go:725` first defaults to prompting.
It bypasses the prompt when `rootFlags.assumeYes` is true,
but otherwise rejects non-terminal standard input or standard output:

```go
promptForDownload := true
var shouldPullImage bool

if rootFlags.assumeYes || domain == "localhost" {
    promptForDownload = false
    shouldPullImage = true
}

if promptForDownload {
    if !term.IsTerminal(os.Stdin) || !term.IsTerminal(os.Stdout) {
        var builder strings.Builder
        fmt.Fprintf(&builder, "image required to create Toolbx container.\n")
        fmt.Fprintf(&builder, "Use option '--assumeyes' to download the image.\n")
        fmt.Fprintf(&builder, "Run '%s --help' for usage.", executableBase)

        errMsg := builder.String()
        return false, errors.New(errMsg)
    }
```

`src/cmd/root.go:102` to `src/cmd/root.go:108` registers `--assumeyes` as a persistent root option:

```go
persistentFlags := rootCmd.PersistentFlags()

persistentFlags.BoolVarP(&rootFlags.assumeYes,
    "assumeyes",
    "y",
    false,
    "Automatically answer yes for all questions")
```

The version-matched system test makes the failure part of Toolbx's contract.
`test/system/101-create.bats:119` to `test/system/101-create.bats:126` asserts all diagnostic lines:

```bash
assert_failure
assert [ ${#lines[@]} -eq 0 ]
lines=("${stderr_lines[@]}")
assert_line --index 0 "Error: image required to create Toolbx container."
assert_line --index 1 "Use option '--assumeyes' to download the image."
assert_line --index 2 "Run 'toolbox --help' for usage."
assert [ ${#stderr_lines[@]} -eq 3 ]
```

Upstream merged [pull request 1428][toolbx-pr-1428] to require `--assumeyes` for an image pull when the command is not
connected to a terminal.
Its commit message explains that a non-interactive caller must make the download decision before invoking Toolbx.

[toolbx-pr-1428]: https://github.com/containers/toolbox/pull/1428

## Verification

The host under test reported:

```text
toolbox version 0.3
toolbox-0.3-4.fc44.x86_64
Bazzite 44.20260721.0 (Kinoite)
```

The verification started without `registry.fedoraproject.org/fedora-toolbox:42` in local Podman image storage.
The image and `firefox-installer` container were removed after the checks.

### Patterns that work cleanly

- A missing image with an explicit non-interactive decision:

  ```sh
  toolbox --assumeyes create firefox-installer
  ```

  Expected and observed final output:

  ```text
  Created container: firefox-installer
  Enter with: toolbox enter firefox-installer
  ```

- The complete prerequisite check after creation:

  ```sh
  toolbox run --container firefox-installer \
    sudo dnf install --assumeyes \
      coreutils curl desktop-file-utils file gawk gnupg2 grep procps-ng tar
  toolbox run --container firefox-installer \
    bash -c 'command -v gpgv && command -v desktop-file-validate && command -v update-desktop-database'
  ```

  Expected and observed command paths:

  ```text
  /usr/bin/gpgv
  /usr/bin/desktop-file-validate
  /usr/bin/update-desktop-database
  ```

### Patterns that fail

#### Toolbx non-terminal diagnostic

```sh
toolbox create firefox-installer
```

When the image is missing and the process lacks terminal standard streams,
this exits `1` with the three-line `image required` diagnostic in `Symptom`.

#### Fedora Toolbx package diagnostic

```sh
toolbox run --container firefox-installer \
  sudo dnf install --assumeyes gnupg2-verify
```

The Fedora 42 Toolbx exits nonzero with `No match for argument: gnupg2-verify`.
The package name exists on the Fedora 44 host,
so copying the host package list into the Toolbx is not portable across the releases selected by Toolbx.

## Verified workarounds

Use the persistent global flag before the subcommand:

```sh
toolbox --assumeyes create firefox-installer
```

This was verified to pull the missing image and create the container from a non-terminal managed process.
The tradeoff is explicit consent to the image download without seeing or answering the interactive size prompt.
It does not suppress later `dnf` decisions,
which need their own `--assumeyes` flag.

For the Firefox installer prerequisites,
install `gnupg2` and verify the executable instead of naming `gnupg2-verify`:

```sh
toolbox run --container firefox-installer \
  sudo dnf install --assumeyes gnupg2
toolbox run --container firefox-installer command -v gpgv
```

The tradeoff is capability-based rather than package-identity verification.
That is intentional because Fedora releases can move `gpgv` between packages while the runbook needs the executable,
not a particular RPM decomposition.

## What does not work

- `toolbox create firefox-installer` from a process without terminal standard streams does not prompt or pull.
  Toolbx refuses by design and names `--assumeyes` in the diagnostic.
- Adding `--assumeyes` only to `dnf` does not authorize Toolbx's earlier image pull.
  Toolbx and `dnf` are separate decision boundaries.
- Adding Fedora 44's `gnupg2-verify` package to the default Fedora 42 Toolbx transaction fails package resolution.
  Installing `gnupg2` and checking `command -v gpgv` covers both package layouts.
- Treating the first failure as a Bazzite image-resolution bug was incorrect.
  The source path and merged pull request show an intentional non-terminal guard after image resolution.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry matches Toolbx or this diagnostic.
The upstream tracker search covered open and closed issues and pull requests containing `assumeyes` and
`non-terminal image pull`.
[Pull request 1428][toolbx-pr-1428] is the exact change that introduced and justified the observed behavior.

1.  **Is it really upstream's fault?**
    No bug was found.
    Toolbx emits its documented and tested non-terminal safety diagnostic.
2.  **Can upstream fix it?**
    Upstream could choose different behavior,
    but no correction is needed for the observed contract.
3.  **Are they supporting this use case?**
    Yes.
    `doc/toolbox.1.md` documents `--assumeyes`,
    and the system test covers non-terminal creation.
4.  **Would the repo welcome our contribution?**
    `CONTRIBUTING.md` welcomes bug reports and pull requests,
    and `.github/ISSUE_TEMPLATE/bug-report.md` requests a reproduction and environment details.
    No AI-assistance prohibition was found.
5.  **Will they likely fix it?**
    No change is indicated.
    Maintainers merged the current behavior deliberately in pull request 1428.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not applicable because constraint 1 fails.
    A non-interactive consumer can use the supported flag,
    and the runbook's candidate fallback was removed.

There is nothing additive to post on pull request 1428 and no new issue to file.
The upstream filing artifact is therefore intentionally empty.
