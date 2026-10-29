# mise 2026.10.0: launcher shims fail in the IDE sandbox and every mise command runs trusted `[env]` commands

## Symptom

The Slint IDE (`package/ide/app`) started from its desktop entry reports its language servers as missing,
or as failed to start, for a project whose tools come from mise.

- The launcher's `PATH` is the systemd user session's snapshot.
  It holds `~/.local/share/mise/shims` and the global tool directories, but not the tool directories of the repository.
- A server found as a shim (`~/.local/share/mise/shims/taplo` is a symbolic link to `~/.local/bin/mise`)
  is started inside bubblewrap and stops with "No version is set for shim",
  because the shim chooses its version from the working directory's mise configuration.
- A server that has no shim shows "the program 'X' for Y was not found on PATH".

## Root cause

Established by probes with mise 2026.10.0 (`linux-x64`, 2026-10-02) on the user's host.
The mise source was not read, so the explanation of each probe below is the observed behavior only.

- Shims: `readlink --canonicalize ~/.local/share/mise/shims/taplo` prints `~/.local/bin/mise`.
  The IDE's old lookup (`resolve_executable`) accepted any executable regular file, and a shim passes.
- Tool directories: `mise bin-paths` run in the project prints them (`.../installs/taplo/0.10.0`,
  `.../installs/node/26.11.1/bin`, `.../installs/npm-yaml-language-server/1.25.0/bin`).
  A Node server's launcher starts with `#!/usr/bin/env node`, so `node` has to be on the `PATH` inside the sandbox too.
- Every mise command evaluates a trusted configuration's `[env]` templates.
  A template `{{ exec(command='touch MARKER') }}` created its marker when `mise config ls --json`,
  `mise bin-paths`, `mise which taplo`, `mise env --json` and `mise exec -- true` ran.
  A `[hooks]` entry `enter = "touch MARKER2"` created no marker for any of them.
  Run on the bare host, that is project code with network and write access.
- `mise exec` and `mise env` bring the whole `[env]` of the project.
  In the Monochromatic repository `mise env --json` prints 42 variables, 26 of them named `...KEY...` or `...TOKEN...`
  (decrypted secrets; only the names were read).
  Handing those to a server would bypass the cleared environment of `src/language/confine/recipe.rs`.
- Trust: a configuration with `[env]`, `[hooks]` or `[settings]` is refused until trusted:
  exit status 1 and
  ``mise ERROR Config files in /path/mise.toml are not trusted.``
  followed by ``Trust them with `mise trust`. See https://mise.jdx.dev/cli/trust.html for more information.``
  A configuration with only `[tools]`, in a folder that is not trusted
  (`mise trust --show` printed `untrusted`), resolved without refusal.
- Settings: an environment variable wins over a project's `[settings]`.
  `mise settings get auto_install` printed `true` in the repository
  (its `mise.toml` sets `auto_install = true`) and `false` with `MISE_AUTO_INSTALL=false` in the environment.
  The same holds for `exec_auto_install`, `not_found_auto_install`, `task.run_auto_install`, `lockfile` and `offline`.
- Installing: in a disposable `MISE_DATA_DIR`, `mise exec -- true` with auto install on
  exited 1 (it tried to fetch); with `MISE_AUTO_INSTALL=false` it exited 0.
  With `MISE_AUTO_INSTALL=false`, a listed tool that is not installed gives exit 0 and the warning
  `mise WARN  missing: taplo@0.9.0` on standard error.
- A sandboxed mise cannot record the configuration: it prints
  `mise WARN  tracking config: failed to ln -sf ...` because the sandbox keeps `~/.local/state/mise` read-only.
- Unsandboxed probe runs with `MISE_TRUSTED_CONFIG_PATHS` set left trust records for three scratch folders
  (`~/.local/state/mise/trusted-configs/ide-mise-servers-proj-a-...`, `-proj-b-...`, `-proj-c-...`).
  Why the records appeared, and why one of them is for a folder whose path was not in that variable,
  was not determined.
  The trust page (https://mise.jdx.dev/cli/trust.html, fetched 2026-10-14) says commands that run project code,
  such as `mise run` and `mise install`, "trust their active config automatically";
  whether `config ls` or `bin-paths` do the same was not determined.
  Removing the three links restored the refusal for the folder with `[env]`.
  The sandboxed run in the IDE cannot write that directory, so it cannot trust anything.

## Verification

Measured with `hyperfine` (15 runs, 2 warmups), host load average about 60 to 80:

- `mise bin-paths`, project with two tools: 235 ms plain (standard deviation 43), 319 ms in the sandbox (41).
- `mise bin-paths`, the Monochromatic repository: 413 ms plain (80), 387 ms in the sandbox (152).
- `mise config ls --json`: 24 ms (4); `mise which taplo`: 35 ms (4); `mise exec -- true`: 114 ms (24).
- The IDE's resolution (three commands, sandboxed, on the language thread): 179 to 639 ms over four runs of
  `inspect:language-mise` (179, 479, 545, 639), and 415 ms in the headless window run.

`inspect:language-mise` (`package/ide/app/bin/inspect-language-mise.mjs`,
now `inspect-language-mise.ts`,
renamed 2026-10-22)
passed 43 of 43 checks
with the launcher's `PATH`:
`bash-language-server` answered a hover (`Function: **greet**`),
`taplo` and `marksman` started, and `marksman` answered a hover on a wiki link;
an untrusted configuration gave the notice with mise's message;
a folder without mise kept `was not found on PATH`;
a fake `mise` that sleeps was stopped at 10 seconds with no process left;
every project tree was the same before and after.

Marksman gives no hover when its workspace has no root marker:
a project with `notes.md` and `other.md` answered no hover on a wiki link until `.marksman.toml` was added,
then answered the linked note's text.

## Verified workarounds

The fix is in the IDE (`doc` of it: README section "Servers from mise"):

- Ask mise for the directories (`config ls --json` twice, `bin-paths` once), inside the servers' sandbox,
  with auto install, lockfile writes and network off, and look servers up in those directories first.
  Tradeoff: three runs at project start (about 0.2 to 0.6 s on the language thread, never the window thread),
  and a project `[env]` template runs confined, without network (a template that needs one was not tried).
- Skip mise shims in a project mise serves.
  Tradeoff: a tool that exists only as a shim is reported missing with the shim named, never started.
- Put mise's directories first on the sandbox `PATH`.
  Tradeoff: the global tools mise lists (for example `bun`, `gh`) are on the servers' `PATH`, read-only;
  variables a tool adds (`GOROOT`, `JAVA_HOME`) are not passed.

## What does not work

- `mise exec -- SERVER` as the launched command: it hands the project's `[env]`
  (the 26 secret-like variables above) to the server.
- Running mise on the bare host: the `[env]` templates of a trusted project run with network and write access.
- `mise which SERVER` for every server: about 35 ms each, and it does not give `node` for the launcher's shebang.
- `paranoid = true`: not tried; it would make every configuration need trust, including the tools-only ones.
- Detecting the project's configuration by file names: mise's own listing gives it without a name list.
- Using the project's listing alone: the user's global configuration is in every listing,
  so the files the listing from `/` lacks are the project's own.

## Upstream filing decision

Nothing to file; the audit stops at the first constraint, and the rest are walked for the record.

1. Is it really upstream's fault?
   No.
   The trust page (https://mise.jdx.dev/cli/trust.html, fetched 2026-10-14) says "Trust lets mise load the parts of a
   config file that can run code or change your environment", and exempts only plain-version `[tools]` entries.
   The behaviors above (refusal without trust, every command loading a trusted file) match that page;
   the evaluation of `[env]` templates by `config ls` and `bin-paths` was observed, not found stated on the pages read
   (https://mise.jdx.dev/configuration.html does not mention `exec()`).
   The failure the IDE had was its own: it accepted a shim and ran no resolution at all.
2. Can upstream fix it?
   Not applicable, nothing to fix upstream.
3. Are they supporting this use case?
   Not determined; the pages read do not address a sandboxed caller.
4. Would the repo welcome our contribution?
   Not checked; no issue is drafted.
5. Will they likely fix it?
   Not applicable.
6. Prototyped a minimal fix?
   Not applicable.

`.out-of-scope/` was not searched for mise, and no tracker search was made, because no issue is drafted.
