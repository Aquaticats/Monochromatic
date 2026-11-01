# Per-user `meow` configuration: discovery, precedence, trust, cache, and reload
Merged into "Per-user configuration" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17.
Spot-checked then:
the unset `XDG_CONFIG_HOME`,
the root-owned `XDG_CONFIG_DIRS` entries,
the `/home` symlink,
and the `??` home fallback,
which lives at `package/file-enforcer/core/src/jetbrains/options-dir.ts:334-338`
rather than the path this research cited.

Every decision it feeds applies to meow 0.x only (user,
 2026-09-17).

## Status

Design research for meow 0.x,
written 2026-09-17.
Nothing here is adopted.
Rule `DRR` requires the user's explicit acceptance before any `doc/decision/` record,
and rule `VRB` forbids non-document mutations for a research task,
so no product code,
 dependency,
 or decision record changed.

Every claim below is marked Verified with the command,
 file,
 or fetched page that backs it,
or Unverified.

## Settled requirements recorded, not re-asked

These come from the design and decision records and close the choices they touch
(rule `QGR`).

- Configuration syntax is OpenTofu-shaped HCL,
   parsed by a patched `hcl-edit` 0.9.7 under meow's own evaluator,
   with the full language allowed and meow-only functions in the `meow::` namespace,
   and no formatter ships
   (Verified:
   `doc/decision/monorepo-manager-hcl-front-end.md`).
- Every line meow writes to standard output or standard error is a JSON object
   (Verified:
   "Output format" in `doc/planning/monorepo-manager-from-scratch-design.md:131-160`).
- meow serves repositories other than Monochromatic,
   and ships as a single file for Linux x86_64 and aarch64,
   glibc and static musl
   (Verified:
   "Distribution" and "Platforms and builds",
   `doc/planning/monorepo-manager-from-scratch-design.md:82-130`).
- The evaluator carries a base directory per configuration file,
   because `file`,
   `fileset`,
   and `templatefile` resolve relative to it;
   the daemon's read set is the cache key and watch list,
   and it must cover the per-user configuration's reads
   (Verified:
   "Consequences and risks" in the same design,
   and `doc/planning/monorepo-manager-route-research/hcl-tooling.md:1948-1978`).
- Cache keys use XXH3-128 from `twox-hash` 2.1.4
   (Verified:
   `doc/decision/monorepo-manager-cache-key-hash.md`).
- Rules that write outside the repository live in a per-user `meow` configuration
   (Verified:
   "Answers on 2026-09-17" in
   `doc/planning/monorepo-manager-from-scratch-design.md:1221-1248`).
  A per-user configuration therefore exists;
   refusing to have one is closed,
   and is recorded below as a rejected design rather than ranked.
- Every decision applies to meow 0.x only.

## Measured environment

All measurements are from the development machine on 2026-09-17.

- `XDG_CONFIG_HOME` is not set.
  `XDG_CONFIG_DIRS` is
   `/home/user/.config/kdedefaults:/etc/xdg:/usr/share/kde-settings/kde-profile/default/xdg`,
   `XDG_RUNTIME_DIR` is `/run/user/1000`,
   and `HOME` is `/home/user`
   (Verified:
   `env | grep --extended-regexp '^(XDG_|HOME=|USER=|SHELL=)' | sort`).
- Of the three `XDG_CONFIG_DIRS` entries,
   two are owned by `root`:
   `/etc/xdg` and `/usr/share/kde-settings/kde-profile/default/xdg`.
  The first entry,
   `/home/user/.config/kdedefaults`,
   is owned by the user but written by KDE
   (Verified:
   `ls -ld` on the three paths).
- `/home` is a symbolic link to `var/home`,
   so `HOME` names the directory through a link while its real path is `/var/home/user`
   (Verified:
   `ls -ld /home /var/home /home/user && readlink /home`).
- The machine is XDG-oriented:
   `/home/user/.config` holds 159 entries,
   including `gh`,
   `git`,
   `mise`,
   and `starship.toml`,
   while the home directory holds only `.bashrc`,
   `.gitconfig`,
   and `.npmrc`
   as plain configuration dotfiles
   (Verified:
   `ls /home/user/.config | wc --lines`,
   `ls -d` on the four paths,
   and `ls -a /home/user | grep`).
- `git` uses both locations at once on this machine:
   `/home/user/.gitconfig` holds the configuration
   while `/home/user/.config/git/ignore` holds the ignore file
   (Verified:
   `ls -la /home/user/.gitconfig /home/user/.config/git/`).
- mise's per-user configuration is `/var/home/user/.config/mise/config.toml`,
   and its state directory holds 687 trusted-config entries and 1,316 tracked-config entries,
   so one per-user file already serves hundreds of repository paths on this machine
   (Verified:
   `mise config ls --json | grep '"path"'`,
   `ls /home/user/.local/state/mise/trusted-configs | wc --lines`,
   and the same for `tracked-configs`).

### What the repository writes outside itself today

- The only home-directory use in the whole file-enforcer implementation is the JetBrains
   options-directory lookup
   (Verified:
   an `rg` for `homedir()`,
   `os.homedir`,
   and `process.env.HOME`
   over `file-enforcer.config.ts` and `package/file-enforcer/core/src/`
   returns exactly one hit,
   `package/file-enforcer/core/src/jetbrains/options-dir.ts:336`).
- That lookup is
   `process.env.XDG_CONFIG_HOME ?? join(homedir(), '.config')`,
   then `JetBrains`
   (Verified:
   `package/file-enforcer/core/src/jetbrains/options-dir.ts:332-340`).
  It uses `??`,
   so an empty `XDG_CONFIG_HOME` is treated as set,
   and it does not check that the value is absolute.
- The forbidden-strings runtime cache is the repository's only other outside write,
   and it is a Rust implementation with a fuller policy
   (Verified:
   `package/forbidden-strings/cli/src/runtime_cache/path.rs:116-140`):
   the `FORBIDDEN_STRINGS_CACHE_DIR` override must be platform-absolute or the call fails with
   `CacheRootError::InvalidOverride`;
   Windows uses `%LOCALAPPDATA%`;
   macOS uses `$HOME/Library/Caches`;
   otherwise `XDG_CACHE_HOME` is used only `if path.is_absolute()`,
   and `$HOME/.cache` is the fallback.
  Its error type deliberately renders no environment values
   (Verified:
   the `Display` implementation is commented
   "Renders static root-resolution errors without environment values").
- The repository owns no XDG directory crate:
   no `dirs`,
   `dirs-next`,
   `directories`,
   `etcetera`,
   or `xdg` dependency appears in any manifest
   (Verified:
   `grep` over `package/forbidden-strings/cli/Cargo.toml`
   and `rg --glob '*.toml' 'etcetera|^dirs |"dirs"|directories-next|dirs-next' .` returning no match).
- The repository's TypeScript XDG helper for terminal lookup does implement
   `XDG_CONFIG_DIRS` and `XDG_DATA_DIRS` search lists,
   also with `??` and no absolute check
   (Verified:
   `package/terminal-exec/cli/src/xdg-paths.ts:85-135`).
- The repository's `.editorconfig` begins with `root = true`,
   and `/home/user/.editorconfig` does not exist
   (Verified:
   `head -12 .editorconfig` and `ls -la /home/user/.editorconfig`).
- file-enforcer writes managed files through a same-directory temporary file and an atomic rename,
   with an explicit "watch echo suppression" concern
   (Verified:
   `package/file-enforcer/core/src/io/write-atomic.ts:33`,
   `:75`,
   `:107`,
   and `package/file-enforcer/core/src/io/write.ts:138`).

### The concrete case: LI14

The Harper LSP4IJ rule is the one inventory entry that writes outside the repository.
It reads and writes `LanguageServersSettings.xml` and `UserDefinedLanguageServerSettings.xml`
in the latest `IntelliJIdea*` or `IdeaIC*` options directory,
and its `process.cwd()` becomes `path.root` in the HCL sketch
(Verified:
 `doc/planning/monorepo-manager-route-research/stack-declarative-config.md:927-985`).
The research already names the problem in that entry:

> Awkward:
> a repository configuration writing user-scope editor state outside the repository,
> which also sits outside the watched tree and the task sandbox's intent.

The rule's content is repository policy:
it is the repository that wants `AGENTS.md` and `CLAUDE.md` excluded from Harper's prose rules,
and that names the rule identifiers to disable
(Verified:
 `file-enforcer.config.ts:1307-1367`).
Only the destination is a fact about the user's machine.
That split is the hinge of the whole design and recurs in every question below.

## Comparable tools, measured

Each entry is either a live run against the installed binary or a quoted primary source.

### Discovery

- OpenTofu 1.12.6.
  The documentation says the CLI configuration file
   "configures per-user settings for CLI behaviors,
   which apply across all OpenTofu working directories.
   This is separate from your infrastructure configuration",
   that it must be `.tofurc` in the home directory
   "or be named `tofurc` and placed in a valid XDG Base Directory config directory
   such as `$XDG_CONFIG_HOME/opentofu`",
   and that "The configuration file uses the same HCL syntax as `.tf` and `.tofu` files,
   but with different attributes and blocks"
   (Verified:
   fetched
   `https://raw.githubusercontent.com/opentofu/opentofu/main/website/docs/cli/config/config-file.mdx`).
  The implementation reaches the XDG path only when `XDG_CONFIG_HOME` is non-empty
   and neither home dotfile exists
   (Verified:
   fetched `internal/command/cliconfig/config_unix.go`,
   function `configFile`).
  Measured live with `TF_LOG=DEBUG` against disposable homes
   (Verified:
   `scratchpad/per-user-config/tofu-discovery.ts`):
   an existing `.tofurc` is opened;
   with `XDG_CONFIG_HOME` set and no dotfile,
   `$XDG_CONFIG_HOME/opentofu/tofurc` is opened and loaded;
   with `XDG_CONFIG_HOME` unset and a file planted at `$HOME/.config/opentofu/tofurc`,
   OpenTofu attempts `$HOME/.tofurc` and never reaches the planted file;
   `TF_CLI_CONFIG_FILE` wins and also logs
   "Not reading CLI config directory because config location is overridden by environment variable";
   with both a dotfile and an XDG file,
   the dotfile wins.
  The source comments the override rationale:
   "we interpret the environment variable being set as an intention to ignore the default set of CLI config
   files because we're doing something special,
   like running OpenTofu in automation with a
   locally-customized configuration"
   (Verified:
   fetched `internal/command/cliconfig/cliconfig.go:166-181`).
- dprint 0.57.4.
  The per-user directory comes from `DPRINT_CONFIG_DIR` if set and non-empty,
   otherwise the platform config directory plus `dprint`,
   and the file names tried are `dprint.jsonc` then `dprint.json`
   (Verified:
   fetched `crates/dprint/src/configuration/resolve_main_config_path.rs`,
   functions `resolve_global_config_dir` and `resolve_global_config_path_and_text_detail`;
   the empty-value guard is `resolve_env_var_folder`'s `.filter(|f| !f.is_empty())`).
  Measured live:
   `$XDG_CONFIG_HOME/dprint/dprint.json` is found,
   and `$HOME/.config/dprint/dprint.json` is also found with `XDG_CONFIG_HOME` unset,
   which is the opposite of OpenTofu's measured behaviour
   (Verified:
   `scratchpad/per-user-config/dprint-discovery.ts`,
   cases A and B).
- Cargo.
  "It looks for configuration files in the current directory and all parent directories",
   then `$CARGO_HOME/config.toml` "which defaults to ... Unix:
   `$HOME/.cargo/config.toml`",
   with no XDG involvement
   (Verified:
   fetched `https://doc.rust-lang.org/cargo/reference/config.html`,
   "Hierarchical structure").
- Bazel.
  "The system RC file ... `/etc/bazel.bazelrc`",
   "The workspace RC file ... `.bazelrc` in your workspace
   directory",
   "The home RC file ... `$HOME/.bazelrc`",
   the `BAZELRC` environment variable,
   and `--bazelrc=file`,
   where "`/dev/null` indicates that all further `--bazelrc`s will be ignored,
   which is useful to disable the search for a user rc file,
   such as in release builds"
   (Verified:
   fetched `https://bazel.build/run/bazelrc`).
- ripgrep 15.2.0.
  Configuration comes only from `RIPGREP_CONFIG_PATH`,
   and `--no-config` disables it,
   with the help text adding
   "If ripgrep ever grows a feature to automatically read configuration files in pre-defined locations,
   then this flag will also disable that behavior as well"
   (Verified:
   `rg --help`).
- `gh` and starship on this machine use `$HOME/.config` with `XDG_CONFIG_HOME` unset:
   `/home/user/.config/gh/config.yml` exists at mode `0600` inside a `0710` directory,
   and `/home/user/.config/starship.toml` exists
   (Verified:
   `ls -la /home/user/.config/gh/` and `ls -d`).
- The XDG Base Directory Specification,
   version 0.8,
   08 May 2021,
   states that
   "All paths set in these environment variables must be absolute.
   If an implementation encounters a relative path in any of these variables it should consider the path
   invalid and ignore it",
   that `$XDG_CONFIG_HOME` defaults to `$HOME/.config` when "either not set or empty",
   that `$XDG_CONFIG_DIRS` defaults to `/etc/xdg`,
   that `$XDG_CONFIG_HOME` "is considered more important than any of the base directories defined by
   `$XDG_CONFIG_DIRS`",
   and that a missing destination directory should be created "with permission 0700"
   (Verified:
   fetched `https://specifications.freedesktop.org/basedir-spec/latest/`).

### Precedence and merging

- git 2.55.0.
  The manual lists `$(prefix)/etc/gitconfig`,
   then `$XDG_CONFIG_HOME/git/config` and `~/.gitconfig`,
   then `$GIT_DIR/config`,
   then `$GIT_DIR/config.worktree`,
   then `-c`,
   and says "The files are read in the order given above,
   with last value found taking precedence over values read earlier",
   and that "When the XDG_CONFIG_HOME environment variable is not set or empty,
   $HOME/.config/ is used as $XDG_CONFIG_HOME"
   (Verified:
   `man git-config`,
   `FILES`).
  The repository file therefore wins over the per-user file.
- Cargo.
  "If a key is specified in multiple config files,
   the values will get merged together.
   Numbers,
   strings,
   and booleans will use the value in the deeper config directory taking precedence over
   ancestor directories,
   where the home directory is the lowest priority.
   Arrays will be joined together with higher precedence items being placed later in the merged array",
   and "Configuration values specified this way take precedence over environment variables,
   which take precedence over configuration files"
   (Verified:
   fetched Cargo configuration reference).
- npm.
  Project `.npmrc`,
   then `~/.npmrc`,
   then `$PREFIX/etc/npmrc`,
   then the builtin file,
   "resolved in priority order" with the project file highest
   (Verified:
   fetched `https://docs.npmjs.com/cli/v11/configuring-npm/npmrc`).
- mise 2026.9.5,
   measured live on a disposable fixture with its own `MISE_CONFIG_DIR` and `MISE_STATE_DIR`
   (Verified:
   `scratchpad/per-user-config/mise-precedence.ts`).
  Environment values merge per key:
   a key defined only in the per-user file and a key defined only in the repository file both survive,
   and a key defined in both resolves to the repository value.
  Tasks merge by name:
   a task defined only in the per-user file is listed and runs inside the repository,
   and a task name defined in both runs the repository's body.
  So mise's per-user configuration can add tasks that run in every repository the user visits.
- Bazel reverses the direction:
   the home rc file is interpreted after the workspace rc file,
   and "options in later files can override a value from an earlier file if a conflict arises"
   (Verified:
   fetched `https://bazel.build/run/bazelrc`).
- Turborepo reverses it too.
  Its sources,
   listed highest priority first,
   are
   `Cli`,
   `Environment`,
   `OverrideEnvironment`,
   `LocalConfig`,
   `GlobalAuth`,
   `GlobalConfig`,
   `TurboJson`,
   which places the repository's `turbo.json` below the per-user `~/.turbo/config.json`
   (Verified:
   fetched `crates/turborepo-config/src/lib.rs:705-715`).
  Its module documentation states the merge rule and one coupling rule:
   "Scalar fields use first-writer-wins (`overwrite_none`).
   Nested config objects like OTEL options are deep-merged so that a higher-priority source overriding a
   single field does not shadow unrelated fields from lower-priority sources",
   and "Security-sensitive fields (`headers`,
   `use_remote_cache_token`) are coupled to `endpoint`:
   once an endpoint is set by a source,
   credentials from lower-priority sources are discarded"
   (Verified:
   same file,
   lines 1 to 18).
- OpenTofu's own merge is per-field and hand-written,
   and the source admits the precedence was never specified:
   "NOTE:
   The order of arguments to merge below seems confusing ...
   It was unfortunately never well specified what is overriding what here",
   one boolean "saturates to on;
   once either configuration sets it,
   there is no way to override it back to off again",
   and a map merge carries "We just clobber an entry from the other file right now.
   Will improve on this later"
   (Verified:
   fetched `internal/command/cliconfig/cliconfig.go:150-158`,
   `:417-468`).
- EditorConfig is the one measured precedent where the repository can cut off outer configuration.
  `root` is "Set to `true` to tell the core not to check any higher directory for EditorConfig settings",
   and otherwise "pairs in closer files take precedence"
   (Verified:
   fetched `https://spec.editorconfig.org/`).
- ESLint removed its per-user configuration on purpose.
  The RFC's summary is "This RFC deprecates the personal config that is `.eslintrc` files on home directory",
   and the deprecation message directs users to
   "config files for each project or '--config' option"
   (Verified:
   fetched `https://github.com/eslint/rfcs/blob/main/designs/2019-deprecating-personal-config/README.md`
   and the search result quoting the runtime warning).
  Unverified:
   the reproducibility argument usually attributed to this change is paraphrase,
   not a quote from the RFC,
   whose stated motivation is global-installation support.

### Trust

- direnv states the threat model directly:
   "This is the security mechanism to avoid loading new files automatically.
   Otherwise any git repo that you pull,
   or tar archive that you unpack,
   would be able to wipe your hard drive once you cd into it".
  Allow records live in `$XDG_DATA_HOME/direnv/allow`,
   and a per-user `direnv.toml` at `$XDG_CONFIG_HOME/direnv/direnv.toml` may carry a `[whitelist]`
   whose `prefix` entries mean
   "If any of the strings in this list are a prefix of an .envrc file's absolute path,
   that file will be implicitly allowed"
   (Verified:
   fetched `https://direnv.net/man/direnv.1.html` and `https://direnv.net/man/direnv.toml.1.html`).
- git splits scopes by trust.
  "Protected configuration refers to the system,
   global,
   and command scopes.
   For security reasons,
   certain options are only respected when they are specified in protected
   configuration,
   and ignored otherwise",
   with the rationale
   "Git treats these scopes as if they are controlled by the user or a trusted administrator".
  `safe.directory`,
   `safe.bareRepository`,
   and `uploadpack.packObjectsHook` each carry
   "This prevents untrusted repositories from tampering with this value"
   or "This is a safety measure against fetching from untrusted repositories"
   (Verified:
   `man git-config`,
   `SCOPES` and the option entries).
- VS Code does the same at setting granularity:
   "For enhanced security,
   such settings can only be defined in user settings and not at workspace scope",
   and the named examples are `git.path` and `terminal.external.*Exec`,
   with the reason that they "contain paths to executables ...
   which if set to point to malicious code,
   could cause damage".
  Restricted Mode "tries to prevent automatic code execution by disabling or limiting the operation of
   several VS Code features:
   AI agents,
   terminal,
   tasks,
   debugging,
   workspace settings,
   and extensions",
   trust is keyed to a folder,
   a parent folder can be trusted for all its subfolders,
   and policy settings "always override other setting values"
   (Verified:
   fetched `https://code.visualstudio.com/docs/configure/settings`
   and `https://code.visualstudio.com/docs/editing/workspaces/workspace-trust`).
- mise implements the same split,
   and it is measurable.
  A repository-local config may set ordinary settings:
   `jobs = 7` written into a project's `mise.toml` is reported by `mise settings get jobs`.
  A repository-local config may not set the trust list:
   mise emits
   `mise WARN trusted_config_paths in non-global config <path> is ignored for security reasons`
   (Verified:
   `scratchpad/per-user-config/mise-settings-scope.ts`).
  mise's own settings list contains `ignored_config_paths`,
   `trusted_config_paths`,
   `override_config_filenames`,
   and `default_config_filename`
   (Verified:
   `mise settings ls --all --json`).
  Caveat:
   the probe ran with `MISE_YES=1`,
   so it does not prove that an ungranted nested config would have been refused;
   that refusal is already recorded from a separate probe in
   `doc/troubleshooting/mise-disposable-worktree-trust.md`.
- Cargo has no trust model.
  Its configuration reference contains no occurrence of "trust",
   "security",
   or "untrusted",
   although `[alias]` and `target.*.runner` name programs to run
   (Verified:
   regular-expression scan over the fetched page text;
   scope limited to that one page).

### Behaviour when the per-user file is malformed

Measured live on disposable homes with a valid repository configuration present
(Verified:
 `scratchpad/per-user-config/malformed-per-user.ts`).

- git 2.55.0 exits 128 with `fatal: bad config line 1 in file <home>/.gitconfig`,
   for a query the repository file alone could have answered.
  The same failure also blocks `git init`,
   which is why the probe had to create the repository before planting the broken file.
- mise 2026.9.5 exits 1,
   emitting both `mise WARN Error loading settings file ...`
   and a span-annotated `mise::config::parse_error` diagnostic pointing at line 1 column 5.
- dprint 0.57.4 exits 0 and formats normally,
   because default discovery stops at the repository configuration and never reads the per-user file.
  Forcing `--config-discovery=global` surfaces the error as exit 11 with file,
   line,
   and column.

### Watching a configuration file that is replaced atomically

Measured with `inotifywait --monitor` against a watch on the file and a watch on its directory
(Verified:
 `scratchpad/per-user-config/inotify-rename.ts`).

- Positive control,
   an in-place append:
   both watches report `OPEN`,
   `MODIFY`,
   `CLOSE_WRITE,CLOSE`.
- Same-directory temporary file plus `rename`,
   which is this repository's own writer's pattern:
   the file watch reports only `ATTRIB` and `DELETE_SELF`,
   while the directory watch reports `CREATE`,
   `MODIFY`,
   and `CLOSE_WRITE` on the temporary name,
   then `MOVED_FROM` and `MOVED_TO`.
- A later append reaches only the directory watch.
  The file watch is dead after one atomic save.

So a watch on the per-user configuration file alone stops working the first time an editor saves it.

## Question 1: discovery

### Options designed

- Q1A,
   single home dotfile.
  `$HOME/.meowrc`,
   nothing else,
   no override.
- Q1B,
   single XDG file with an explicit override.
  `$XDG_CONFIG_HOME/meow/meow.hcl`,
   defaulting to `$HOME/.config/meow/meow.hcl`
   when the variable is unset,
   empty,
   or relative,
   plus `MEOW_CONFIG` and `--user-config <path>` naming a file that must exist,
   plus `--no-user-config` and `MEOW_NO_USER_CONFIG`.
  No `XDG_CONFIG_DIRS`.
- Q1C,
   XDG file plus the `XDG_CONFIG_DIRS` search list.
  As Q1B,
   then each `XDG_CONFIG_DIRS` entry in order,
   first match wins or all merged.
- Q1D,
   home dotfile first with an XDG fallback.
  OpenTofu's measured shape:
   `$HOME/.meowrc`,
   and `$XDG_CONFIG_HOME/meow/meow.hcl`
   only when the dotfile is absent and the variable is non-empty.
- Q1E,
   XDG directory with fragments.
  `$XDG_CONFIG_HOME/meow/meow.hcl` plus `$XDG_CONFIG_HOME/meow/conf.d/*.hcl`
   merged in lexicographic order.
- Q1F,
   no default path.
  Only `MEOW_CONFIG` and `--user-config`,
   ripgrep's shape.

Rejected without ranking:
having no per-user configuration at all,
 ESLint's answer.
The user settled on 2026-09-17 that rules writing outside a repository live in a per-user configuration,
so this option is closed by a settled requirement.

### Pros and cons

Q1A.
Pros:
one path,
 nothing to resolve;
survives an unset or hostile `XDG_CONFIG_HOME`.
Cons:
this machine holds 159 entries under `.config` and only three configuration dotfiles in `$HOME`
 (Verified),
 so meow would land in the minority location;
no override,
 so a reproducible or automated run cannot ignore the file;
a daemon has no way to be told to use a different file.

Q1B.
Pros:
matches where the user's other tools already are,
 measured;
the absolute-path check follows the XDG specification's explicit instruction
 and reuses the repository's own Rust policy in
 `package/forbidden-strings/cli/src/runtime_cache/path.rs:116-140`,
 so no new dependency;
the override pair covers both "use this file instead" and "use no file",
 which are the two needs Bazel covers with `--bazelrc` and `--nohome_rc`
 and ripgrep covers with `RIPGREP_CONFIG_PATH` and `--no-config`.
Cons:
a user who expects a dotfile finds nothing,
 and meow must say where it looked;
a single file means machine-specific and user-specific settings share one document.

Q1C.
Pros:
follows the specification's lookup rule literally;
would let an administrator ship a site-wide default.
Cons:
disqualifying for this design.
On this machine two of the three `XDG_CONFIG_DIRS` entries are owned by `root` and the third is
 written by KDE (Verified),
 so a file discovered there is not "the user's own file",
 which is the premise question 3 rests on.
A configuration that decides what may be written outside a repository
 and which repositories may run tasks must not arrive from a directory the user does not author.

Q1D.
Pros:
matches the settled HCL ancestor exactly;
a user migrating a `.tofurc` habit finds the same shape.
Cons:
disqualifying on measured behaviour.
With `XDG_CONFIG_HOME` unset,
 which is this machine's state,
 a file at `$HOME/.config/meow/meow.hcl` is never read,
 and the tool reports nothing because the dotfile is simply absent
 (Verified:
 case C of the OpenTofu probe).
The obvious location becomes a silent no-op.

Q1E.
Pros:
separates concerns inside one directory,
 for instance machine paths against personal preferences;
lets a dotfile manager drop a fragment in without rewriting a shared file.
Cons:
reintroduces an ordering question inside one author's own configuration,
 which is precisely where OpenTofu's merge admits it
 "was unfortunately never well specified what is overriding what here" (Verified);
diagnostics must name which fragment set a value;
nobody has asked for it in 0.x.

Q1F.
Pros:
smallest possible surface;
no path policy to get wrong;
an explicit path is always unambiguous in a diagnostic.
Cons:
the only path comes from the environment,
 and a daemon's environment is fixed when it starts,
 so a per-user file created later is invisible to a running daemon until it is restarted;
the user must edit a shell profile before meow has any per-user configuration at all.

### Ranking

Q1B > Q1E > Q1D > Q1A > Q1F > Q1C.

- Q1B over Q1E:
   Q1E reopens a precedence question inside a single author's own configuration,
   and the only measured tool that layers configuration fragments this way documents its own order as
   never specified;
   Q1B has one file and therefore nothing to order.
- Q1E over Q1D:
   Q1E always finds a file placed in the directory the user's other tools already use,
   while Q1D was measured to ignore that directory entirely when `XDG_CONFIG_HOME` is unset,
   which is this machine's state.
- Q1D over Q1A:
   Q1D reaches the XDG directory when the variable is set,
   and Q1A never does;
   on a machine with 159 `.config` entries against three dotfiles,
   never reaching it is worse than
   sometimes reaching it.
- Q1A over Q1F:
   Q1A has a discoverable default the user can create with one `mkdir`,
   while Q1F requires editing a shell profile and leaves a running daemon unable to see a file created
   after it started.
- Q1F over Q1C:
   Q1F only makes discovery inconvenient,
   while Q1C sources a trust-bearing file from `root`-owned directories on this machine,
   which contradicts the premise that the per-user file is the user's own.

## Question 2: precedence and merging

This question is split from the question of who may initiate an outside-the-repository write,
which is a separable decision and belongs to question 3 (rule `QSP`).
What follows decides only what happens when both files set the same thing.

### Options designed

- Q2A,
   whole-file replacement.
  When a per-user file exists and names a target,
   its ruleset for that target replaces the repository's.
- Q2B,
   per-key merge,
   repository wins.
  git,
   Cargo,
   npm,
   and mise's measured direction.
- Q2C,
   per-key merge,
   per-user file wins.
  Bazel's and Turborepo's measured direction.
- Q2D,
   disjoint block kinds.
  The per-user file may declare only user-scope block kinds and the repository only repository-scope kinds,
   so an overlap cannot be written.
  OpenTofu's CLI configuration is this shape:
   "the same HCL syntax ... but with different attributes and blocks" (Verified).
- Q2E,
   refusal.
  A target named by both files is an error naming both paths and both byte spans.
- Q2F,
   per-attribute declared scope.
  Every attribute in meow's schema carries a layer scope:
   `user_only`,
   `repository_only`,
   or `both` with a stated winner.
  A value written in a layer that may not set it is ignored with a diagnostic naming the file and the span,
   which is mise's measured behaviour for `trusted_config_paths`
   and VS Code's documented behaviour for `git.path`.

### Pros and cons

Q2A.
Pros:
one file decides,
 so no merge semantics exist to specify;
dprint's measured fallback shows the shape working in practice.
Cons:
a repository always has a configuration,
 so under a strict reading the per-user file would never apply;
under the looser per-target reading,
 a per-user typo can silently delete rules the repository needs;
it is the only option where adding a per-user file can reduce what the repository enforces.

Q2B.
Pros:
matches four measured incumbents the user already runs;
a repository can guarantee its own policy holds on every machine.
Cons:
lets a repository override a fact about the user's machine,
 for instance which JetBrains options directory exists,
 which is the failure Turborepo's coupled-field rule exists to prevent (Verified source comment).

Q2C.
Pros:
matches the two measured incumbents that are closest to meow by category,
 Bazel and Turborepo;
the per-user file's reason to exist is machine-scope facts,
 and those should not be overridable by a repository.
Cons:
gives the user's file the last word on attributes the repository is entitled to fix,
 such as which prose rules a repository's writing policy disables;
a per-user file that drifts silently weakens every repository it touches.

Q2D.
Pros:
no merge rule to write,
 no diagnostics to design;
the smallest surface of any option;
follows the settled HCL ancestor's own answer.
Cons:
cannot express LI14,
 the one measured case that motivated the per-user file,
 where the repository owns the rule content and only the destination is user-scope;
forces the whole Harper policy into every developer's personal file,
 so the repository stops being able to state it at all.

Q2E.
Pros:
never resolves an ambiguity silently;
the diagnostic can carry both byte spans,
 which `hcl-edit` provides on every node (Verified).
Cons:
turns any accidental overlap into a hard stop across every repository the per-user file serves;
the four measured merging incumbents all resolve overlap without stopping and remain usable.

Q2F.
Pros:
answers per attribute rather than per file,
 so the LI14 split is expressible directly:
 rule names and exclusion patterns are `both` with the repository winning,
 while the options-directory destination is `user_only`;
the wrong-layer diagnostic is already proven in a shipping tool,
 measured as mise's `ignored for security reasons` warning;
it composes with whichever answer question 3 gets,
 because scope is a schema property.
Cons:
the schema grows a column that must be decided for every attribute meow ever adds,
 and getting one wrong is a security decision made by omission;
a reader must consult the schema to know what a configuration does.

### Ranking

Q2F > Q2D > Q2C > Q2B > Q2E > Q2A.

- Q2F over Q2D:
   Q2D cannot express the one measured case that created this work,
   where the repository owns a rule's content and the user owns only its destination;
   Q2F expresses it by fixing the destination attribute to the user layer.
- Q2D over Q2C:
   Q2D removes the overlap instead of resolving it,
   and a rule that never has to be applied cannot be applied wrongly;
   Q2C resolves every overlap in the user's favour,
   including attributes the repository should fix.
- Q2C over Q2B:
   the per-user file exists to carry facts a repository cannot know,
   and Q2B lets a repository override them,
   which is the exact coupling failure Turborepo's source comments describe guarding against.
- Q2B over Q2E:
   Q2E converts a benign overlap into a stop for every repository the per-user file serves,
   while Q2B's measured incumbents resolve overlap silently and are in daily use on this machine.
- Q2E over Q2A:
   Q2A is the only option under which adding a per-user file can remove rules the repository still needs,
   and Q2E at least fails loudly rather than dropping them.

## Question 3: trust

Two separable decisions (rule `QSP`):
which layer may write outside the repository,
and what gates running a repository's tasks at all.
They are not alternatives,
 and answering both is reachable.

### Question 3-i: which layer may write outside the repository

Options.

- Q3iA,
   per-user file only.
  Only blocks in the per-user file may name a path outside the repository root.
- Q3iB,
   repository only,
   as today.
  Rejected by the user's 2026-09-17 answer;
   listed so the ranking is complete.
- Q3iC,
   both layers,
   unrestricted.
- Q3iD,
   proposal and acceptance.
  The repository may declare the content of an outside write as a proposal that names no destination
   outside its own tree;
  the per-user file supplies the destination and the consent;
  the daemon executes only proposals an acceptance matched.

Pros and cons.

Q3iA.
Pros:
simplest rule to state and to check;
a cloned repository cannot reach outside its tree by any spelling.
Cons:
the repository loses the Harper policy it holds today,
 so every developer must maintain the rule names and exclusion patterns personally;
the repository cannot tell a new contributor what the policy is except in prose.

Q3iC.
Pros:
no new concepts.
Cons:
this is the direnv scenario verbatim:
 "any git repo that you pull ... would be able to wipe your hard drive once you cd into it" (Verified);
it also contradicts the settled 2026-09-17 answer.

Q3iD.
Pros:
keeps the repository as the author of policy and the user as the author of consent,
 which is the split the LI14 case actually has;
matches Turborepo's measured coupling rule,
 where a destination set by one layer discards
 credentials supplied by a lower-priority layer;
the acceptance can name a path prefix,
 so one acceptance can serve many repositories.
Cons:
two block kinds instead of one,
 and a matching rule between them;
a proposal without an acceptance is a silent no-op unless meow reports it,
 so an extra diagnostic is mandatory rather than optional.

Ranking:
 Q3iD > Q3iA > Q3iC > Q3iB.

- Q3iD over Q3iA:
   Q3iA discards the repository's ability to state a policy that today lives in
   `file-enforcer.config.ts:1307-1367`,
   with nothing replacing it;
   Q3iD keeps the policy in the repository and moves only the consent and the destination.
- Q3iA over Q3iC:
   Q3iA makes the dangerous construction unwritable in the untrusted layer,
   while Q3iC relies entirely on whatever question 3-ii decides.
- Q3iC over Q3iB:
   Q3iB is closed by a settled user answer,
   so it is not available at any price,
   while Q3iC is merely unsafe.

### Question 3-ii: what gates running a repository's tasks

Options.

- Q3iiA,
   nothing.
  Cargo's measured position.
- Q3iiB,
   trust on first use,
   keyed by the repository root path,
   recorded under `$XDG_STATE_HOME/meow`,
   mise's shape.
- Q3iiC,
   trust keyed by path plus the configuration's content digest,
   re-asked whenever the configuration changes.
- Q3iiD,
   per-user prefix whitelist only,
   no prompt,
   direnv's `[whitelist] prefix` shape.
- Q3iiE,
   restricted mode.
  An untrusted repository is discovered,
   parsed,
   and reported,
   but running tasks,
   writing outside the repository,
   and impure functions are refused
   until the path is trusted.

Pros and cons.

Q3iiA.
Pros:
no friction,
 no state,
 no command.
Cons:
meow serves repositories other than Monochromatic by settled requirement,
 so a fresh clone is exactly the input direnv's security note describes;
Cargo's own reference never mentions trust,
 and adding one later is a breaking change.

Q3iiB.
Pros:
proven in a tool the user runs daily,
 with 687 trusted paths already recorded on this machine;
keyed by absolute path,
 so a new worktree is a new decision,
 which the repository already documented as expected behaviour
 (`doc/troubleshooting/mise-disposable-worktree-trust.md`).
Cons:
a prompt needs a client,
 and 0.x emits only JSON with no TUI;
a trusted path stays trusted after its configuration changes.

Q3iiC.
Pros:
a configuration change is a new decision,
 which is the honest reading of "trust this content".
Cons:
this repository's configuration changes constantly,
 so the user would answer the same question repeatedly;
a digest prompt trains the user to accept without reading.

Q3iiD.
Pros:
no prompt at all,
 which fits JSON-only output perfectly;
one line in the per-user file covers a whole directory of repositories;
direnv ships exactly this.
Cons:
a prefix grants trust to repositories that do not exist yet,
 so the clone scenario direnv's own documentation names is unguarded inside the prefix.

Q3iiE.
Pros:
an untrusted repository stays useful:
 meow can still list tasks,
 report the graph,
 and answer queries,
 which a daemon watching many repositories needs;
maps cleanly onto JSON output,
 since the refusal is an object naming what would be needed;
matches the measured VS Code shape,
 including trusting a parent folder for its subfolders.
Cons:
every capability must be classified as restricted or not,
 and the list must stay correct;
the restricted state is a second mode to test.

Ranking:
 Q3iiE > Q3iiB > Q3iiD > Q3iiC > Q3iiA.

- Q3iiE over Q3iiB:
   Q3iiB makes an untrusted repository useless until answered,
   while Q3iiE keeps read-only work available,
   which matters because the daemon watches repositories rather than being invoked per command.
- Q3iiB over Q3iiD:
   Q3iiD grants trust to repositories that do not yet exist under a prefix,
   which is the clone case direnv's own security note names,
   while Q3iiB decides each path once.
- Q3iiD over Q3iiC:
   Q3iiC re-asks on every legitimate configuration edit in a repository whose configuration changes
   constantly,
   which trains acceptance without reading;
   Q3iiD asks once,
   in a file the user wrote deliberately.
- Q3iiC over Q3iiA:
   Q3iiC records a decision that can be audited and revoked,
   while Q3iiA records nothing and has no upgrade path.

### What the daemon does when a repository reaches outside its own tree

Recommended in all cases:
refuse the evaluation,
 not the whole configuration,
and emit a JSON object naming the repository root,
 the offending block,
 its byte span,
the resolved absolute destination,
 and the acceptance that would permit it,
following rules `DGT` and `DNL`.

Containment must be decided on canonicalized real paths.
On this machine `HOME` is `/home/user` while its real path is `/var/home/user` (Verified),
so a prefix test between `$HOME`-derived strings and a repository root discovered through a different
spelling disagrees with itself.
The check resolves both sides before comparing,
and compares path components rather than string prefixes,
so that `/var/home/user/Monochromatic-notes` is not treated as inside `/var/home/user/Monochromatic`.

## Question 4: identity and the cache

### Options designed

- Q4A,
   whole-file digest in every cache key.
- Q4B,
   per-user file excluded from cache keys.
- Q4C,
   consumed-subset fingerprints.
  The evaluator's read set already records `File`,
   `Absent`,
   `Directory`,
   `Glob`,
   and `Env` entries
   (Verified:
   "Recommended shape" in the design);
  extend it with a `UserValue` entry carrying the block address,
   the attribute,
   and the value digest,
   and key on the consumed subset only.
- Q4D,
   per-block address digest.
  Key on the digest of whole per-user blocks that the rule names.
- Q4E,
   two-level.
  The whole-file digest keys configuration evaluation;
   consumed-subset fingerprints key task entries.

### Pros and cons

Q4A.
Pros:
one digest,
 impossible to get subtly wrong;
satisfies the settled requirement that the cache key include both configurations' digests.
Cons:
every repository's cache is invalidated by any per-user edit,
 which is the outcome the brief asks to avoid;
a comment added to the per-user file rebuilds unrelated work everywhere.

Q4B.
Pros:
no per-repository coupling at all.
Cons:
stale hits after a per-user change;
contradicts the settled statement that the read set is the cache key and watch list.

Q4C.
Pros:
the finest instrument available,
 and it reuses machinery the design already has;
an edit to a block nothing consumed invalidates nothing.
Cons:
configuration evaluation itself needs a key,
 and deciding whether to re-evaluate by consuming a record produced by evaluation is circular
 unless evaluation is always re-run;
`UserValue` entries multiply the read set's size.

Q4D.
Pros:
cheaper bookkeeping than Q4C;
invalidation stays confined to repositories whose rules name the edited block.
Cons:
a comment or an unrelated attribute inside a named block still invalidates,
 and meow ships no formatter,
 so the file keeps whatever the author wrote,
 which makes incidental edits common.

Q4E.
Pros:
puts the cheap instrument where the work is cheap and the precise one where it is expensive:
 a 12,618-byte configuration parses in 371 microseconds median (Verified),
 while tasks are the expensive thing the cache exists for;
an editing session re-evaluates configuration freely without touching task entries;
it keeps the settled "both configurations' digests" property at the evaluation layer,
 where it is true.
Cons:
two key shapes to document and to explain in a diagnostic;
a task entry's key no longer contains the per-user file's digest,
 so "why did this not rebuild" needs the consumed-subset record to answer.

### Ranking

Q4E > Q4C > Q4D > Q4A > Q4B.

- Q4E over Q4C:
   Q4C must decide whether to re-enter the evaluator using a record that only the evaluator produces,
   so it either re-evaluates unconditionally or keeps a second key anyway;
   Q4E names that second key and puts the cheap digest exactly where parsing is measured in microseconds.
- Q4C over Q4D:
   Q4D invalidates on any byte inside a named block,
   including comments,
   and meow ships no formatter,
   so incidental edits inside a block are expected;
   Q4C keys on consumed values.
- Q4D over Q4A:
   Q4A couples every repository's cache to every per-user edit,
   the outcome the brief asks to avoid;
   Q4D confines invalidation to repositories whose rules name the edited block.
- Q4A over Q4B:
   Q4B leaves stale hits and contradicts the settled read-set requirement,
   while Q4A is merely wasteful.

### Watch-list consequences

- The per-user configuration's containing directory must be watched,
   not the file.
  A single atomic save kills a watch on the file inode,
   measured above.
- The per-user file's own reads resolve relative to its own base directory,
   by settled requirement,
   so `file`,
   `fileset`,
   and `templatefile` calls inside it produce read-set entries outside the repository
   that must also be watched.
- A per-user `fileset` over a large tree can exhaust the watch budget.
  Measured context:
   99,416 directories exist under the repository with only `.git` excluded,
   against `max_user_watches` of 524,288
   (Verified:
   "Watching and affected work" in the design).
  meow should bound the per-user file's watch footprint and diagnose the bound rather than fail later.
- `Absent` entries already cover a per-user file that does not exist,
   so creating one invalidates what depended on its absence.

## Question 5: multiple repositories and multiple daemons

Three separable decisions.

### Question 5-i: daemon identity

- Q5iA,
   one daemon per repository root,
   with a socket under `$XDG_RUNTIME_DIR` named by a digest of the canonical root,
   each daemon reading the per-user file itself.
- Q5iB,
   one daemon per user,
   serving every repository from one process.
- Q5iC,
   one daemon per repository plus a separate per-user configuration service.

Pros and cons.

Q5iA.
Pros:
one repository's broken configuration,
 frozen task,
 or crash does not reach another;
matches the existing design,
 which has the user starting the daemon in its own terminal
 under a delegated cgroup (Verified);
the per-user file must be evaluated per repository anyway,
 see below,
 so a shared reader saves nothing.
Cons:
the per-user file is parsed once per repository;
a per-user change must reach every running daemon.

Q5iB.
Pros:
one reader,
 one watch,
 one reload;
one socket,
 which is what `$XDG_RUNTIME_DIR` naturally holds.
Cons:
the failure domain is every repository at once,
 and the measured malformed-configuration outcomes show shared readers failing whole tools;
one cgroup tree for unrelated repositories complicates the per-task sandbox.

Q5iC.
Pros:
a single authoritative reader without a single failure domain for tasks.
Cons:
a second process,
 a second socket,
 and a protocol between them,
 for a file whose parse is measured in microseconds;
buys nothing Q5iB does not,
 at higher cost.

Ranking:
 Q5iA > Q5iB > Q5iC.

- Q5iA over Q5iB:
   Q5iB shares one failure domain across repositories,
   and the saving it offers,
   a single parse of the per-user file,
   does not exist,
   because `path.root` inside the per-user file must name the repository being served,
   which makes its evaluation per-repository regardless.
- Q5iB over Q5iC:
   Q5iC adds a process and a protocol to solve a problem measured in microseconds,
   and Q5iB solves it with a function call.

The per-repository evaluation point is load bearing.
The LI14 sketch uses `path.root` inside the block that moved to the per-user file
(Verified:
 `doc/planning/monorepo-manager-route-research/stack-declarative-config.md:943`,
"`process.cwd()` becomes `path.root`",
and `:962`,
 where the exclusion patterns interpolate it;
`:363` defines `path.root` as the repository root),
and Bazel's rc files use `%workspace%` the same way (Verified).
So `path.root` inside the per-user file names the repository currently being served,
the per-user file is evaluated once per repository,
and its read set and cache participation are per-repository.

### Question 5-ii: reload on change

- Q5iiA,
   read at startup only,
   restart to pick up changes.
- Q5iiB,
   re-read on every command.
- Q5iiC,
   watch the directory,
   debounce,
   re-evaluate.
- Q5iiD,
   as Q5iiC,
   plus a configuration generation number,
   where in-flight tasks keep the generation they started under.

Ranking:
 Q5iiD > Q5iiC > Q5iiB > Q5iiA.

- Q5iiD over Q5iiC:
   a task's cache key was computed under a configuration snapshot,
   so changing the snapshot mid-run makes the recorded key not describe the run;
   the generation number is what lets the record stay truthful.
- Q5iiC over Q5iiB:
   the daemon already coalesces change bursts for the repository (Verified),
   and re-reading per command duplicates that work while still missing changes between commands.
- Q5iiB over Q5iiA:
   Q5iiA makes a per-user edit invisible until the user notices and restarts,
   which for a long-lived daemon means noticing that nothing happened.

### Question 5-iii: malformed while the daemon is running

- Q5iiiA,
   exit.
  git's measured behaviour,
   exit 128 on every command.
- Q5iiiB,
   fail every command until fixed.
  mise's measured behaviour,
   exit 1 with a span.
- Q5iiiC,
   keep the last good snapshot,
   serve from it,
   and emit a JSON diagnostic naming the file,
   the span,
   and what is being served instead.
- Q5iiiD,
   drop the per-user file and continue with the repository configuration only.

Ranking:
 Q5iiiC > Q5iiiB > Q5iiiD > Q5iiiA.

- Q5iiiC over Q5iiiB:
   a daemon holds a snapshot that was valid moments earlier and has clients attached,
   so it can keep answering while naming the break;
   mise's measured behaviour fails a run the repository configuration alone could have served.
- Q5iiiB over Q5iiiD:
   Q5iiiD silently changes which writes are permitted and which repositories are trusted,
   so a typo would change the security posture without saying so;
   Q5iiiB says so.
- Q5iiiD over Q5iiiA:
   git's measured `fatal: bad config line 1` stops operations that never needed the file,
   and for a daemon it would also drop every attached client and every queued task.

Under Q5iiiC the cache stays correct without special handling,
because task keys are consumed-subset fingerprints taken from the last good snapshot (question 4),
not a digest of whatever bytes are on disk right now.
That is a second reason to prefer Q4E over Q4A:
under Q4A a broken file has a new digest and would invalidate every entry while being unusable.

## Recommended design

### Discovery

```text
1.  --user-config <path>              file must exist; otherwise a hard error naming the path
2.  MEOW_CONFIG=<path>                same rule
3.  $XDG_CONFIG_HOME/meow/meow.hcl    only when the value is absolute; empty and relative are ignored
4.  $HOME/.config/meow/meow.hcl       the specification's default when the variable is unset or unusable
```

`--no-user-config` and `MEOW_NO_USER_CONFIG=1` skip all four and record an `Absent` read-set entry.
Naming an explicit file also disables any future search,
following OpenTofu's measured rationale for the same switch.
No `XDG_CONFIG_DIRS` search list in 0.x.

Other platforms follow the shape of the repository's own incumbent resolver,
explicit override,
 then platform-native directory,
 then XDG,
 then the home fallback,
rather than inventing a second policy
(Verified shape:
 `package/forbidden-strings/cli/src/runtime_cache/path.rs:116-140`).
The concrete per-platform values differ from that incumbent's,
because it resolves a cache directory and this resolves a configuration directory:
macOS uses `$HOME/Library/Application Support/meow/meow.hcl`
and Windows uses `%APPDATA%\meow\meow.hcl`,
with `XDG_CONFIG_HOME` honoured first on macOS as dprint was measured to do.
Those two values are this appendix's proposal,
 not a Verified incumbent choice.
These are code paths only;
0.x guarantees Linux,
 and an issue on another system does not block publishing.

Resolution is hand written over `std::env::var_os`,
matching the incumbent,
 so no dependency and no binary-size change.
Rule `RCI` applies:
 the responsibility already has an owner in this repository.
A missing directory is created with mode `0700` when meow first writes state,
which is what the specification instructs.

Every diagnostic that mentions a configuration names the absolute path,
because a repository file and a per-user file may both be called `meow.hcl`.

### Layering

Per-attribute declared scope (Q2F).
Each attribute in meow's schema carries `user_only`,
 `repository_only`,
 or `both`,
and `both` carries which layer wins.
A value in a layer that may not set it is ignored,
with a JSON diagnostic naming the file,
 the byte span,
 the attribute,
 and the reason,
in the shape mise was measured to emit.

### Outside-the-repository writes

Proposal and acceptance (Q3iD).
A repository declares content and names no destination outside its own tree.
A per-user block supplies the destination and the consent,
and may match a path prefix so one acceptance serves many repositories.
An unmatched proposal is reported,
 never silently skipped.
For LI14 this means the repository keeps the Harper rule names and exclusion patterns,
while the per-user file names the JetBrains options directory and accepts the proposal.

### Trust

Restricted mode (Q3iiE) with trust keyed by canonical repository root path,
recorded under `$XDG_STATE_HOME/meow`,
 mise's measured location for the same purpose.
In the restricted state meow parses,
 reports the task graph,
 and answers queries,
but refuses to run tasks,
 to write outside the repository,
 and to evaluate impure functions.
Trusting a parent path covers its children,
 as VS Code documents.
The per-user file may also carry prefix acceptances,
which is direnv's whitelist by another name;
the prefix grants the outside-write acceptance,
 not the task-running trust,
so the clone scenario stays gated.

### Cache and watching

Two-level keys (Q4E).
The whole per-user file digest,
 computed with XXH3-128 like every other meow key,
participates in the configuration evaluation key.
Task entries carry consumed-subset fingerprints:
`UserValue` read-set entries naming the block address,
 the attribute,
 and the value digest.
The per-user configuration's containing directory is watched,
 never the file inode.
The per-user file's own `file`,
 `fileset`,
 and `templatefile` reads join the watch list,
under a declared bound with a diagnostic when the bound is reached.

### Daemons and reload

One daemon per canonical repository root (Q5iA),
socket under `$XDG_RUNTIME_DIR` named by a digest of that root.
`path.root` inside the per-user file names the repository being served,
so the per-user file is evaluated once per repository.
Watch and reload with a configuration generation number (Q5iiD);
in-flight tasks keep the generation they started under.
A malformed per-user file keeps the last good snapshot and emits a diagnostic (Q5iiiC).

## Interactions with settled requirements

- Single-file shipping:
   the recommendation adds no dependency,
   because the repository already owns an equivalent Rust resolver and no XDG crate is in any manifest
   (Verified).
  Binary size is unchanged by this design.
- JSON-only output:
   there is no interactive trust prompt in the daemon.
  meow emits a decision-needed object and a client decides;
   `meow trust <path>` is the non-interactive path.
  This is what makes restricted mode preferable to a prompt in 0.x.
- Base directory per configuration file:
   the per-user file's base directory is its own directory,
   so `file("harper-extra.txt")` inside it resolves under `$XDG_CONFIG_HOME/meow/`,
   not under the repository.
  This is what keeps the per-user file portable across repositories,
   and it is why `path.root` must be the way to name the repository instead.
- meow serves other repositories:
   the per-user file must not assume this repository's layout,
   which is why acceptances and trust entries are path-prefix shaped.
- No formatter:
   meow should not write to the per-user configuration at all.
  Trust and acceptance records live in `$XDG_STATE_HOME/meow`,
   so the user's own file is never rewritten and its style is never touched.
  The alternative,
   editing the per-user HCL with the comment-preserving writer,
   would put meow and the user in the same file.
- Uncacheable evaluations:
   a per-user file calling `timestamp`,
   `uuid`,
   or `bcrypt` makes evaluations uncacheable in every
   repository that user opens,
   not just one.
  The existing diagnostic already names the function and the block;
   it should also name that the call sits in the per-user file.
- Nesting pre-scan:
   `hcl-edit` aborts the process at nesting depth 5,000 with no knob (Verified),
   and the per-user file is read by a long-lived daemon,
   so the pre-scan must cover it as well as repository files.
- Cache key hash:
   per-user digests and `UserValue` fingerprints use XXH3-128 from `twox-hash` 2.1.4,
   as settled.
- One recorded statement needs refining if Q4E is taken.
  The design currently says "the cache key includes both the repository and per-user configuration digests"
   (Verified:
   "Consequences and risks" in
   `doc/planning/monorepo-manager-from-scratch-design.md`).
  That sentence is a consequence the HCL research recorded,
   not a user answer,
   and under Q4E it holds for the configuration evaluation key but not for a task entry's key,
   which carries consumed-subset fingerprints instead.
  Merging this appendix should reword it rather than leave two readings in the design.

## Risks

- The schema scope column is a security decision made by omission.
  An attribute added without a scope defaults to something,
   and whichever default is chosen will be wrong for some future attribute.
  Mitigation:
   make the scope a required field of the schema definition so omission does not compile.
- Proposal and acceptance can drift.
  A repository can rename a proposal and every user's acceptance stops matching,
   silently reducing
   enforcement.
  Mitigation:
   report unmatched proposals and unmatched acceptances,
   both,
   on every evaluation.
- The per-user file becomes a second place to look when something goes wrong,
   and it is not in version control.
  This is the objection that removed ESLint's personal configuration.
  Mitigation:
   every diagnostic names the absolute path of the file that decided the value,
   and `meow doctor` reports which per-user file is in effect.
- Watching outside the repository crosses the boundary the sandbox design assumes.
  The design says the daemon watches the repository;
   watching `$XDG_CONFIG_HOME/meow` is a second root with different exclusion rules.
- Unverified:
   whether `$XDG_RUNTIME_DIR` exists in the GitHub runner environment,
   already listed as a risk in the design,
   applies to the per-repository socket naming too.
- Unverified:
   the cost of evaluating the per-user file once per repository has not been measured,
   because meow does not exist.
  The only measured number is that a 12,618-byte configuration parses in 371 microseconds median
   with a 30.5 percent run-to-run band.

## Questions for the user

Each hinges on preference or authority,
 not on something measurable.
Ranking and adjacent reasons are given for each.

### May the repository declare outside-the-repository write proposals

- Option A,
   proposal and acceptance (Recommended).
  The repository declares content,
   the per-user file declares destination and consent.
  Pros:
   the Harper policy stays in the repository where a new contributor can read it;
   the repository can change rule names without asking every developer to edit a personal file;
   matches the measured coupling rule that keeps a destination and its payload from being chosen by
   different layers.
  Cons:
   two block kinds and a matching rule;
   an unmatched proposal is a no-op that only a diagnostic makes visible.
- Option B,
   per-user file only.
  Pros:
   one rule,
   nothing to match,
   nothing to diagnose;
   the strictest reading of the 2026-09-17 answer.
  Cons:
   the repository stops being able to state the policy at all;
   every developer maintains the same rule names by hand and they drift.
- Option C,
   both layers may write outside,
   with trust as the only gate.
  Pros:
   no new concepts.
  Cons:
   a cloned repository can name any path on the machine as soon as it is trusted for tasks,
   which conflates two permissions that the rest of this design keeps separate.

Ranking:
 A > B > C.
A over B because B discards a policy that exists in the repository today with nothing replacing it.
B over C because B makes the dangerous construction unwritable rather than relying on one trust decision
to cover two very different permissions.

### What gates running tasks in a repository meow has not seen before

- Option A,
   restricted mode until the path is trusted (Recommended).
  Pros:
   an untrusted repository is still useful for reading,
   listing,
   and diagnosing;
   fits JSON-only output,
   because the refusal is an object rather than a prompt;
   a trusted parent path covers its children.
  Cons:
   every capability must be classified,
   and a second mode must be tested.
- Option B,
   trust on first use,
   keyed by path,
   no restricted mode.
  Pros:
   the shape already proven on this machine across 687 recorded paths;
   one concept instead of two.
  Cons:
   an untrusted repository answers nothing,
   which is worse for a daemon that watches many trees.
- Option C,
   per-user prefix whitelist only,
   no per-repository decision.
  Pros:
   no prompts,
   no state file,
   one line covers a directory of repositories.
  Cons:
   a repository cloned into a whitelisted directory runs its tasks unasked,
   which is the case direnv's own documentation warns about.
- Option D,
   nothing in 0.x.
  Pros:
   no friction while meow only serves this repository.
  Cons:
   the settled requirement says meow serves other repositories;
   adding a gate later changes behaviour for existing users.

Ranking:
 A > B > C > D.
A over B because restricted mode keeps an untrusted repository readable.
B over C because C grants trust to repositories that do not exist yet.
C over D because C records an explicit decision that can be audited and revoked,
 while D records nothing.

### May the per-user file define tasks and checks that run in every repository

mise allows this,
 measured:
 a task defined only in the per-user config is listed and runs inside a project.

- Option A,
   no.
  The per-user file may declare only user-scope kinds:
   acceptances,
   destinations,
   trust,
   preferences.
  Pros:
   the per-user file cannot change what a repository's build does,
   so a shared cache key and a shared build result stay explainable;
   removes a whole class of "works on my machine".
  Cons:
   loses a genuinely useful facility the user already has in mise.
- Option B,
   yes,
   but only in a separate kind that cannot shadow a repository task name.
  Pros:
   personal helpers everywhere,
   without changing any repository's graph;
   a name collision is impossible by construction.
  Cons:
   a second task kind to document,
   schedule,
   and cache.
- Option C,
   yes,
   same kinds,
   repository wins on a name collision.
  Pros:
   matches mise exactly,
   so the migration is familiar.
  Cons:
   a per-user task participates in the same graph and cache as repository tasks,
   so a per-user edit can change a repository build's result.

Ranking:
 B > A > C.
B over A because B delivers the facility without letting it touch a repository's graph.
A over C because C lets a file outside version control change a repository build,
which is the property that makes a cache result explainable.

### Should a reproducible or automated run ignore the per-user file by default

- Option A,
   flag and environment variable,
   default on everywhere (Recommended).
  `--no-user-config` and `MEOW_NO_USER_CONFIG`,
   but the per-user file is read unless asked otherwise.
  Pros:
   one behaviour to explain;
   a user who set something in the per-user file gets it in every context;
   the explicit switch covers the reproducible case,
   as Bazel's `/dev/null` sentinel does.
  Cons:
   a per-user file silently participates in a CI run if one ever exists there.
- Option B,
   flag and environment variable,
   and automatically off when a CI environment is detected.
  Pros:
   reproducibility by default where it matters most.
  Cons:
   environment detection is a guess,
   and a detection change silently changes what a build reads;
   the user may legitimately want a per-user file on a self-hosted runner.
- Option C,
   no switch at all.
  Pros:
   nothing to document.
  Cons:
   no way to reproduce a run without the per-user file,
   which is the complaint that ended ESLint's personal configuration.

Ranking:
 A > B > C.
A over B because B decides a build's inputs from an inferred environment rather than from what someone
wrote down.
B over C because B at least has a switch,
 inferred or not,
 while C has no path to a reproducible run.

## Probe inventory

All probes are in the session scratchpad under `per-user-config/`
and are not durable;
the recorded commands and outputs above are the evidence.

- `tofu-discovery.ts`:
   five disposable homes against `tofu` 1.12.6 with `TF_LOG=DEBUG`.
- `dprint-discovery.ts`:
   four disposable homes against `dprint` 0.57.4.
- `mise-precedence.ts`:
   per-user against repository env keys and task names,
   `mise` 2026.9.5.
- `mise-settings-scope.ts`:
   settings written from a repository config,
   including `trusted_config_paths`.
- `malformed-per-user.ts`:
   git 2.55.0,
   mise 2026.9.5,
   and dprint 0.57.4 against a broken per-user file.
- `inotify-rename.ts`:
   file watch against directory watch under a same-directory temporary file and rename.

Fetched pages are cached under `~/temp/agent/` as
`opentofu-config-file.mdx`,
 `tofu-cliconfig.go`,
 `tofu-config-unix.go`,
`cargo-config.html`,
 `dprint-resolve.rs`,
 `bazelrc.html`,
 `xdg-basedir.txt`,
`turbo-commands.rs`,
 and `turbo-config-lib.rs`.
Every request used a generic User-Agent with no personal identifier.
