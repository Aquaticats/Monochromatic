# mise 2026.10.0 github: backend installs openinterpreter rust-v0.0.55's wheel as a binary, so no CLI lands

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

Diagnosed 2026-10-02 against `mise 2026.10.0 linux-x64 (2026-10-02)` on Bazzite 44 (glibc 2.43),
 linux-x64.
 Source excerpts come from `jdx/mise` `main` at
 `b1b8d3e4aed6a0a610fdd1d845df11da470afd08` (2026-10-02),
 cited as repo-relative paths with line numbers.
 Release under test:
 `openinterpreter/openinterpreter` tag `rust-v0.0.55`.

## Symptom

`mise use -g github:openinterpreter/openinterpreter` reports success,
 then every plausible command name is missing:

```bash
# terminal transcript (verbatim)
❯ mise use -g github:openinterpreter/openinterpreter
✓ installed 1 tool in 8.0s: github:openinterpreter/openinterpreter@rust-v0.0.55
❯ openinterpreter
bash: command not found: openinterpreter
❯ i
bash: command not found: i
❯ interpreter
bash: command not found: interpreter
```

Three more surface patterns produce the same state,
 one diagnostic per pattern:

- `mise which interpreter` prints
  `mise ERROR interpreter is not a mise bin. Perhaps you need to install it first.`
- Executing the only installed file prints
  `/bin/bash: line 1: .../openai_codex_cli_bin-0.0.55-py3-none: cannot execute binary file: Exec format error`
  and exits 126,
  because that file is a Python wheel (zip magic `PK\x03\x04`),
  not a binary.
- `ls ~/.local/share/mise/installs/github-openinterpreter-openinterpreter/rust-v0.0.55`
  shows exactly one file,
  `openai_codex_cli_bin-0.0.55-py3-none`,
  139908647 bytes,
  byte-identical in size to the release asset
  `openai_codex_cli_bin-0.0.55-py3-none-manylinux_2_17_x86_64.whl`.

Trigger surface:
 any `mise use`/`mise install` of `github:openinterpreter/openinterpreter` with default tool options
 on linux-x64,
 at any version,
 because the release publishes a platform-tagged `.whl` alongside
 `open-interpreter-package-*-unknown-linux-musl` tarballs
 and asset scoring depends only on asset names.

## Root cause

The call chain is:
 asset scoring picks the wheel,
 format detection classifies it `Raw`,
 the `Raw` install path copies it as an "executable" with a cleaned name,
 and shims expose that name.

### Step 1: libc scoring prefers the `manylinux` wheel over the musl package

The picker's target libc defaults to `gnu` on Linux
 (`src/backend/asset_matcher.rs:220-228`):

```rust
        let target_libc = libc.unwrap_or_else(|| {
            if target_os == "windows" {
                "msvc".to_string()
            } else {
                "gnu".to_string()
            }
        });
```

`LIBC_PATTERNS` maps `manylinux` to `Gnu` and `musl` to `Musl`
 (`src/backend/asset_matcher.rs:181-189`):

```rust
        (
            AssetLibc::Gnu,
            Regex::new(r"(?i)(?:\b|_)(?:gnu|glibc|manylinux(?:[0-9_]+)?)(?:\b|_)").unwrap(),
        ),
        (
            AssetLibc::Musl,
            Regex::new(r"(?i)(?:\b|_)(?:musl|musllinux(?:[0-9_]+)?)(?:\b|_)").unwrap(),
        ),
```

`score_libc_match` then swings the ranking by 35 points
 (`src/backend/asset_matcher.rs:551-563`):

```rust
    fn score_libc_match(&self, asset: &str) -> i32 {
        let asset = self.platform_part(asset);
        for (libc, pattern) in LIBC_PATTERNS.iter() {
            if pattern.is_match(asset) {
                return if libc.matches_target(&self.target_libc) {
                    25
                } else {
                    -10
                };
            }
        }
        0
    }
```

So `openai_codex_cli_bin-0.0.55-py3-none-manylinux_2_17_x86_64.whl` scores `+25`
 while `open-interpreter-package-x86_64-unknown-linux-musl.tar.zst` scores `-10`.
 Format scoring only narrows the gap:
 `score_format_preferences` gives the zstd tarball `+12` and the wheel `0`
 (`src/backend/asset_matcher.rs:589-627`),
 whose comment states wheels are deliberately unbonused:

```rust
        // `.whl` and `.gem` are intentionally NOT in this list: both have
        // platform-tagged variants whose tokens OS_PATTERNS doesn't reliably
        // catch (`manylinux2014_x86_64`, `mingw32`), so granting the bonus
        // could let a wrong-platform variant be picked. Those cases should
        // use an explicit `asset_pattern`.
```

Nothing keeps the wheel out of the candidate set.
 `is_non_executable_asset` exists for exactly that job and names its purpose
 (`src/backend/asset_matcher.rs:702-725`):

```rust
/// Assets that cannot become a runnable tool through mise's normal extraction
/// path. Keep them out of automatic selection while still allowing an explicit
/// `asset_pattern` or URL to select one for custom installation logic.
fn is_non_executable_asset(asset: &str) -> bool {
    let asset = asset.to_lowercase();
    let is_package_or_installer = asset.split('.').any(|extension| {
        matches!(
            extension,
            "apk"
                | "appx"
                | "appxbundle"
                | "deb"
                | "dmg"
                | "mpkg"
                | "msi"
                | "msix"
                | "msixbundle"
                | "pkg"
                | "rpm"
        )
    });
```

`whl` and `gem` are absent from that list,
 so a wheel stays auto-selectable.

### Step 2: `.whl` is classified `Raw`, so neither extraction nor magic detection runs

`install_artifact` picks the format by file name only
 (`src/backend/static_helpers.rs:678-685`):

```rust
    let format = if let Some(format_opt) = lookup_with_fallback(opts, "format") {
        file::ExtractionFormat::from_ext(&format_opt).unwrap_or(file::ExtractionFormat::Raw)
    } else {
        file::ExtractionFormat::from_file_name(
            &file_path.file_name().unwrap_or_default().to_string_lossy(),
        )
    };
```

`ExtractionFormat::from_file_name` falls back to `Raw` for unknown extensions
 (`crates/mise-util/src/file.rs:2058-2072`),
 and the `ExtractionFormat` variant list carries no `whl` serialization:

```rust
    pub fn from_file_name(filename: &str) -> Self {
        let filename = filename.to_lowercase();
        ...
        if let Some(ext) = Path::new(&filename).extension().and_then(|s| s.to_str()) {
            Self::from_ext(ext).unwrap_or(ExtractionFormat::Raw)
        } else {
            ExtractionFormat::Raw
        }
    }
```

A content-based detector exists and would have recognized the wheel as a zip
 (`crates/mise-util/src/file.rs:2083-2103`),
 but `install_artifact` never calls it:

```rust
    pub fn detect(path: &Path, filename: &str) -> Result<Self> {
        match Self::from_file_name(filename) {
            ExtractionFormat::Raw => Ok(Self::from_magic(path)?.unwrap_or(ExtractionFormat::Raw)),
            format => Ok(format),
        }
    }
    ...
        if magic.starts_with(b"PK\x03\x04") {
            return Ok(Some(ExtractionFormat::Zip));
        }
```

### Step 3: the `Raw` path chmods the wheel executable and strips its platform tag

With `format == Raw`,
 `install_artifact` takes the plain-copy arm and renames the file
 (`src/backend/static_helpers.rs:724-755`):

```rust
        } else {
            // Always auto-clean binary names by removing OS/arch suffixes
            let original_name = file_path.file_name().unwrap().to_string_lossy();
            let cleaned_name = clean_binary_name(&original_name, Some(&tv.ba().tool_name));
            let dest = install_path.join(cleaned_name);
            file::copy(file_path, &dest)?;
            file::make_executable(&dest)?;
        }
```

`clean_binary_name` (`src/backend/static_helpers.rs:1404`) delegates to
 `strip_platform_token_suffix` (`src/backend/static_helpers.rs:1518-1557`),
 which cuts a trailing suffix only when every `-`/`_`-separated token is a
 platform or version token and the suffix holds both an OS and an arch token.
 For `openai_codex_cli_bin-0.0.55-py3-none-manylinux_2_17_x86_64.whl` the suffix
 `manylinux_2_17_x86_64.whl` tokenizes to `manylinux`,
 `2`,
 `17`,
 `x86`,
 `64.whl`,
 and all five pass because `is_platform_or_version_token` accepts any
 digit-initial token
 (`src/backend/platform_tokens.rs:29-45`),
 while `is_os_token` accepts the `manylinux` prefix
 (`src/backend/platform_tokens.rs:47-52`):

```rust
pub(super) fn is_platform_or_version_token(token: &str) -> bool {
    ...
    token.chars().next().is_some_and(|c| c.is_ascii_digit())
}

pub(super) fn is_os_token(token: &str) -> bool {
    token.starts_with("manylinux")
        || token.starts_with("musllinux")
        || BINARY_OS_TOKENS.contains(&token)
        || PREFERRED_NAME_OS_TOKENS.contains(&token)
}
```

The trailing `.whl` rides along with the cut,
 which is why the installed file is named
 `openai_codex_cli_bin-0.0.55-py3-none`
 rather than keeping its extension.

### Step 4: mise shims the fake binary and reports success

Bin discovery (`src/backend/github.rs:1982` `discover_bin_paths`) finds the single file in the
 install root,
 so the shim directory gains
 `~/.local/share/mise/shims/openai_codex_cli_bin-0.0.55-py3-none`,
 `mise use` prints `✓ installed 1 tool`,
 and no `interpreter`,
 `i`,
 or `openinterpreter` command exists at any version.
 Note `openinterpreter` is never an upstream command name:
 `install.sh:10-11` defaults `COMMAND_NAME` to `interpreter` and
 `ALIAS_COMMAND_NAMES` to `i`.

## Verification

Version under test:
 `mise 2026.10.0 linux-x64 (2026-10-02)`,
 `github:openinterpreter/openinterpreter@rust-v0.0.55`,
 glibc host.
 The harness runs mise against disposable `MISE_DATA_DIR`/`MISE_CACHE_DIR`,
 so no global tool state or `~/.config/mise/config.toml` is touched.

```bash
# /var/home/user/temp/agent/mise-verify: disposable fixture
mkdir --parents "${HOME}/temp/agent/mise-verify/data" "${HOME}/temp/agent/mise-verify/cache"
export MISE_DATA_DIR="${HOME}/temp/agent/mise-verify/data"
export MISE_CACHE_DIR="${HOME}/temp/agent/mise-verify/cache"
mise install 'github:openinterpreter/openinterpreter[matching=open-interpreter-package]@rust-v0.0.55'
mise exec 'github:openinterpreter/openinterpreter@rust-v0.0.55' -- interpreter --version
mise exec 'github:openinterpreter/openinterpreter@rust-v0.0.55' -- i --version
```

### Patterns that fail (default autodetection)

- `mise use -g github:openinterpreter/openinterpreter` installs
  `openai_codex_cli_bin-0.0.55-py3-none-manylinux_2_17_x86_64.whl`
  and reports `✓ installed 1 tool`.
- `openinterpreter`,
  `i`,
  `interpreter` all raise `bash: command not found`.
- `mise which interpreter` raises
  `mise ERROR interpreter is not a mise bin. Perhaps you need to install it first.`
- Running `.../rust-v0.0.55/openai_codex_cli_bin-0.0.55-py3-none --version`
  raises `cannot execute binary file: Exec format error`,
  exit 126.

### Patterns that work cleanly (`matching` tool option)

- The harness above installs
  `open-interpreter-package-x86_64-unknown-linux-musl.tar.zst`
  (`mise ✓ ... 8.2s`).
- `interpreter --version` prints `interpreter 0.0.55`.
- `i --version` prints `interpreter 0.0.55`.
- `mise ls-remote 'github:openinterpreter/openinterpreter[matching=open-interpreter-package]'`
  accepts the option and lists the `rust-v0.0.*` tags,
  so the option parses on mise 2026.10.0.

## Verified workarounds

### Narrow asset selection with `matching` (verified)

Status (2026-10-02):
 applied to `~/.config/mise/config.toml` as the live global entry,
 with
 `mise uninstall` of the wheel install,
 `mise install`,
 and `mise reshim`;
 `interpreter --version` and `i --version` both print `interpreter 0.0.55`
 from the shims.

```toml
# ~/.config/mise/config.toml
[tools]
"github:openinterpreter/openinterpreter" = { version = "latest", matching = "open-interpreter-package" }
```

Then `mise install github:openinterpreter/openinterpreter`.
 Verified end to end in the disposable fixture of Verification:
 `interpreter` and `i` both run and print `interpreter 0.0.55`.

Tradeoffs:

- The entry is coupled to the upstream asset stem `open-interpreter-package`.
 If upstream renames it,
 selection fails loudly with no matching asset,
 which is a better failure mode than the silent wrong asset.
- Platform autodetection is kept,
 so one entry covers every OS/arch;
 on linux-x64 it selects the `.tar.zst` variant.
- Every executable in the extracted `bin/` directory is exposed,
 including `codex-code-mode-host`.
 `filter_bins = ["i", "interpreter"]` would hide it,
 but whether the CLI spawns that helper by name is unverified,
 so leaving it exposed is the safe setting.

### Unverified alternatives (not run here)

- `asset_pattern` naming the per-platform package asset,
 which the `score_format_preferences` comment recommends for wheels.
 Tradeoff:
 it replaces autodetection,
 so the config must template or enumerate Rust target triples per platform,
 as the existing `github:lemonade-sdk/llamacpp-rocm` entry does.
- Upstream's own `install.sh`,
 which installs `interpreter` and the `i` alias into `~/.local/bin`
 plus release directories under `~/.openinterpreter`.
 Tradeoff:
 outside mise's version and PATH management.

## What does not work

- Guessing command names.
 `openinterpreter` is not an upstream command at all,
 and `i`/`interpreter` are absent because nothing runnable was installed.
- `mise which interpreter` or `mise reshim`.
 Both operate on installed binaries,
 and the install contains none.
- Executing the installed file directly.
 It is a zip archive,
 so the kernel rejects it with `Exec format error`.
- Pinning the version.
 `score_asset` (`src/backend/asset_matcher.rs:443`) scores names only,
 so `@rust-v0.0.55` re-picks the same wheel.

## Upstream filing artifact (do not file as-is)

### Upstream filing decision

`.out-of-scope/` checked first:
 the directory holds `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
 and other unrelated entries,
 with no mise,
 asset-selection,
 or packaging exemption,
 so no exemption applies.

1. **Is it really upstream's fault?**
   Yes for the wheel-as-binary gap.
   `is_non_executable_asset` states its job is to keep assets that "cannot become
   a runnable tool" out of automatic selection,
   and `ExtractionFormat::detect` already recognizes zip magic,
   yet `install_artifact` calls the name-only `from_file_name` and then
   `make_executable` on the result.
   The musl-vs-gnu preference itself is documented scoring with a documented
   remedy (`matching`/`asset_pattern`),
   so it is not part of the complaint.
2. **Can upstream fix it?**
   Yes.
   Adding `whl` and `gem` to the extension list in `is_non_executable_asset`
   keeps wheels out of auto-selection while preserving explicit
   `asset_pattern`,
   matching the function's stated purpose.
3. **Are they supporting this use case?**
   Yes.
   The `matching`,
   `asset_pattern`,
   and `bin` tool options are documented for multi-asset releases,
   and `is_non_executable_asset` exists to guard automatic selection.
4. **Would the repo welcome our contribution?**
   No.
   `CONTRIBUTING.md:4-10` restricts AI replies to Discussions and Issues to
   authors of the thread or accounts with a merged contribution,
   and calls violation "an instant ban across all of jdx's projects",
   covering "lightly edited, reviewed, or disclosed model output".
   `gh search prs --repo jdx/mise --author '@me' 'is:merged'` returns nothing,
   and PR `#13886` ("docs(contributing): make the restricted AI reply policy and
   instant ban unmissable") shows the policy is actively reinforced.
5. **Will they likely fix it?**
   Not assessed.
   Moot while constraint 4 fails.
   For the record:
   no tracker hit describes this gap
   (see the duplicate search),
   so no maintainer "won't fix" signal exists.
6. **Have we prototyped a minimal fix?**
   No,
   and deliberately so.
   The auto-prototype trigger requires constraints 1 through 5 to hold;
   constraint 4 fails hard and no filing can result,
   so a prototype would have no consumer.
   Recorded here so a future session does not read the missing prototype as an
   oversight.

Decision:
 do not file.
 The ban in `CONTRIBUTING.md` is dispositive,
 and the consumer-side `matching` workaround solves the user-facing problem
 regardless of upstream movement.

### Duplicate search

Run 2026-10-02 with `gh search issues --repo jdx/mise`:
 `'whl wheel asset'`,
 `'github backend wrong asset autodetect'`,
 `'wheel manylinux'`,
 `'github backend musl'` (all empty),
 `'whl'` and `'wheel'` (only Python packaging threads such as `#2969`).
 Positive control:
 the same harness returns hits for `'install'` and `'wheel'`,
 so empty multi-term results mean no match.
 Nearest neighbor is `#13700` "Wrong binary installed for
 `aqua:domcyrus/rustnet`" (closed),
 the mirror-image case where the aqua backend preferred a gnu binary over the
 registry-pinned musl one;
 different backend,
 different mechanism,
 not a duplicate.

### Draft (do not file as-is)

~~~md
Title: github: backend auto-selects a platform-tagged .whl and installs it as a Raw binary (chmod +x, shim created)

Labels: bug

Body:

With default tool options, `mise use -g github:openinterpreter/openinterpreter@rust-v0.0.55` on linux-x64
reports `✓ installed 1 tool` but installs only the release asset
`openai_codex_cli_bin-0.0.55-py3-none-manylinux_2_17_x86_64.whl`, renamed to
`openai_codex_cli_bin-0.0.55-py3-none`, chmod +x, and shimmed. No runnable command exists afterwards.

Trace (source at b1b8d3e4aed6a0a610fdd1d845df11da470afd08):

1. `src/backend/asset_matcher.rs:551` `score_libc_match` gives `manylinux*` a +25 gnu match while the
   product's `open-interpreter-package-*-unknown-linux-musl.tar.zst` takes -10, so the wheel wins the pick.
2. `is_non_executable_asset` (`src/backend/asset_matcher.rs:705`) excludes deb/rpm/msi/dmg and friends from
   auto-selection because they "cannot become a runnable tool", but not `whl` or `gem`.
3. `install_artifact` (`src/backend/static_helpers.rs:683`) classifies by name via
   `ExtractionFormat::from_file_name`, so `.whl` is `Raw` even though `ExtractionFormat::detect`
   (`crates/mise-util/src/file.rs:2083`) recognizes `PK\x03\x04` as Zip.
4. The `Raw` arm (`src/backend/static_helpers.rs:724`) copies the file, `make_executable`s it, and
   `clean_binary_name` strips `manylinux_2_17_x86_64.whl` as a platform suffix.

Repro:

```sh
MISE_DATA_DIR=/tmp/mise-repro mise install 'github:openinterpreter/openinterpreter@rust-v0.0.55'
ls /tmp/mise-repro/installs/github-openinterpreter-openinterpreter/rust-v0.0.55
# openai_codex_cli_bin-0.0.55-py3-none
```

Suggested fix: add `whl` and `gem` to the `is_package_or_installer` extension list in
`is_non_executable_asset`, so wheels stay out of automatic selection while explicit `asset_pattern`
still works, exactly as that function's doc comment prescribes. A complementary fix is routing
`install_artifact` through `ExtractionFormat::detect` so zip magic is honored when the name misleads.

Workaround: `matching = "open-interpreter-package"` on the tool entry.
~~~
