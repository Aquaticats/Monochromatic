# slint-lsp 1.17.0 reports `Unknown element 'FlexboxLayout'` until `SLINT_ENABLE_EXPERIMENTAL_FEATURES` is set in the LSP process environment

`FlexboxLayout` and the other experimental Slint builtins are fully implemented
in the slint-lsp shipped at the git rev the desktop apps pin
(`slint-ui/slint@85e3eb76819762cdcaa732fa87533ff896546bac`,
 crate version
`1.17.0`),
 but the compiler strips them from the default type register and only
restores them when `SLINT_ENABLE_EXPERIMENTAL_FEATURES` is present in the LSP
process environment.
 The slint-lsp binary has no CLI flag for this.
 In IntelliJ
IDEA via the third-party `kizeevov/slint-idea-plugin`,
 the element shows as
"Unknown Element" because the plugin (a) bundles an older slint-lsp by default
and (b) never sets that env var and exposes no field to set it.

## Symptom

Editing a `.slint` file that uses an experimental builtin,
 the editor underlines
the element with a diagnostic from the Slint compiler:

```text
Unknown element 'FlexboxLayout'
```

This is the bare variant (no trailing parenthetical).
 A sibling variant exists
for a type that is in scope but internal:

```text
Unknown element 'FlexboxLayout'. (The type exists as an internal type, but cannot be accessed in this scope)
```

Affected names are the experimental builtins removed from the default register:
`FlexboxLayout`,
 `ComponentContainer`,
 and the `FlexboxLayout*` enums
(`FlexboxLayoutDirection`,
 `FlexboxLayoutAlignContent`,
 `FlexboxLayoutWrap`,
`FlexboxLayoutAlignSelf`).
 The two desktop-app packages
(`package/terminal/app`,
 `package/music-player/desktop-app`) already
set `SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1"` in their `mise.toml [env]` for the
build,
 so `FlexboxLayout` compiles there;
 the editor is a separate process that
does not inherit that build env.

The gate is compiler-level (`lib.rs:248` below),
 so it also affects the
standalone `slint-viewer` previewer,
 not just the LSP.
 Verified at the pinned rev:
`slint-viewer flex.slint` with the env unset prints `error: Unknown element
'FlexboxLayout'`,
 while `mise exec -- slint-viewer flex.slint` (the repo root
`mise.toml [env]` injects the var) compiles past it and fails only at window
backend init under headless.
 Run the previewer via `mise exec -- slint-viewer ...`
so it inherits the var.

## Root cause

The element is defined and implemented;
 it is gated,
 not missing.
 Walking the
chain in `slint-ui/slint@85e3eb76`:

1. `FlexboxLayout` is a real exported builtin component.

   ```text
   internal/compiler/builtins.slint:2105
   export component FlexboxLayout {
   ```

2. The non-experimental type register explicitly removes it,
    while the
   experimental register keeps everything.

   ```rust
   // internal/compiler/typeregister.rs:655
   pub fn builtin_experimental() -> Rc<RefCell<Self>> {
       let register = Self::builtin_internal();
       Rc::new(RefCell::new(register))
   }

   // internal/compiler/typeregister.rs:660
   pub fn builtin() -> Rc<RefCell<Self>> {
       let mut register = Self::builtin_internal();
       register.elements.remove("ComponentContainer").unwrap();
       register.types.remove("component-factory").unwrap();
       register.elements.remove("FlexboxLayout").unwrap();   // :666
       register.types.remove("FlexboxLayoutDirection").unwrap();
       register.types.remove("FlexboxLayoutAlignContent").unwrap();
       register.types.remove("FlexboxLayoutWrap").unwrap();
       register.types.remove("FlexboxLayoutAlignSelf").unwrap();
       Rc::new(RefCell::new(register))
   }
   ```

3. The diagnostic text seen by the editor is emitted when lookup fails against
   the active register.

   ```rust
   // internal/compiler/typeregister.rs:725
   format!("Unknown element '{name}'")
   ```

   The parenthetical sibling variant lives at
   `internal/compiler/langtype.rs:587` and `:605`.

4. Which register is used is decided by `enable_experimental`,
    read from the
   environment when the default compiler configuration is constructed.

   ```rust
   // internal/compiler/lib.rs:248
   let enable_experimental = std::env::var_os("SLINT_ENABLE_EXPERIMENTAL_FEATURES").is_some();
   ```

5. The LSP deliberately leaves its own flag `false` and relies on that env-var
   path;
    there is no CLI flag and no other runtime switch.

   ```rust
   // tools/lsp/main.rs:451
   // The i_slint_compiler::CompilerConfiguration::default() will read the environment variable
   enable_experimental: false,
   ```

   The LSP's `Cli` struct (`tools/lsp/main.rs:69`) defines only `-I`,
    `-L`,
   `--style`,
    `--backend`,
    `--no-toolbar`,
    and subcommands.
    No `--experimental`.
   When experimental is on,
    the document cache swaps in the experimental
   register:

   ```rust
   // tools/lsp/common/document_cache.rs:321
   self.type_loader.compiler_config.enable_experimental = true;
   // :323
   Rc::into_inner(TypeRegister::builtin_experimental()).unwrap().into_inner();
   ```

On the editor side,
 the IntelliJ plugin `kizeevov/slint-idea-plugin` launches the
LSP without that env var and provides no way to add one:

- It bundles its own slint-lsp,
   version `1.16.0` by default (its `CHANGELOG.md`,
  plugin release 1.5.0),
   which predates `FlexboxLayout` entirely.
   An external LSP
  path is configurable (`README.md`:
   Settings,
   Languages and Frameworks,
   Slint,
  Slint-lsp path).
- It builds the process with `ParentEnvironmentType.CONSOLE` and never calls
  `withEnvironment`,
   so the LSP only inherits the login-shell environment
  IntelliJ captured at startup.

  ```kotlin
  // src/main/kotlin/dev/slint/ideaplugin/ide/lsp/CommandLineHandler.kt:54
  return GeneralCommandLine(path).apply {
      addParameters(parameters)
      withParentEnvironmentType(GeneralCommandLine.ParentEnvironmentType.CONSOLE)
      withCharset(Charsets.UTF_8)
  }
  ```

- Its settings model has no environment field
  (`src/main/kotlin/dev/slint/ideaplugin/ide/settings/SlintLspSettings.kt`:
  `path`,
   `args`,
   `style`,
   `backend`,
   `noToolbar`,
   `includePaths`,
  `useExternalLsp`,
   `providedByEditor`).
   The user-supplied `args` are split on
  whitespace and prepended before the plugin's own flags
  (`CommandLineHandler.kt:22`),
   which is the lever the workaround uses.

## Verification

Version under test:
 `slint-ui/slint@85e3eb76819762cdcaa732fa87533ff896546bac`,
`slint-lsp 1.17.0`,
 installed via mise (`cargo:https://github.com/slint-ui/slint`
at `rev:85e3eb76...`).
 `slint-lsp --version` prints `slint-lsp 1.17.0`,
 which
distinguishes the pinned master build from a crates.
io 1.16 release.

Fixture (`flex.slint`):

```slint
export component Demo inherits Window {
    FlexboxLayout {
        Rectangle { background: red; width: 50px; height: 50px; }
    }
}
```

Harness (`lsp-flex.ts`,
 run with `bun lsp-flex.ts <rootDir> <docPath> <cmd...>`).
It drives one LSP session over stdio and prints the `publishDiagnostics`
messages.
 `<cmd...>` is the command to launch,
 so the same harness tests the raw
binary,
 an `env`-prefixed launch,
 or a `mise exec` launch.

```ts
const rootDir = Bun.argv[2]; const docPath = Bun.argv[3]; const cmd = Bun.argv.slice(4);
const rootUri = `file://${rootDir}`; const docUri = `file://${docPath}`;
const docText = await Bun.file(docPath).text();
const proc = Bun.spawn(cmd, { cwd: rootDir, stdin: 'pipe', stdout: 'pipe', stderr: 'pipe' });
function frame(m: unknown) { const b = JSON.stringify(m); return new TextEncoder().encode(`Content-Length: ${Buffer.byteLength(b)}\r\n\r\n${b}`); }
const sink = proc.stdin; async function send(m: unknown) { sink.write(frame(m)); await sink.flush(); }
let initd = false; const diags: string[] = []; let buf = Buffer.alloc(0);
function drain() { for (;;) { const he = buf.indexOf('\r\n\r\n'); if (he === -1) return; const hdr = buf.subarray(0, he).toString(); const m = hdr.split('\r\n').find((l) => l.toLowerCase().startsWith('content-length:')); if (!m) { buf = buf.subarray(he + 4); continue; } const len = Number.parseInt(m.split(':')[1].trim(), 10); const s = he + 4; if (buf.length < s + len) return; const body = buf.subarray(s, s + len).toString(); buf = buf.subarray(s + len); handle(JSON.parse(body)); } }
function handle(msg: any) { if (msg.id === 1 && msg.result) initd = true; if (msg.method === 'textDocument/publishDiagnostics') for (const d of (msg.params.diagnostics ?? [])) diags.push(d.message ?? ''); }
const reader = proc.stdout.getReader();
const rl = (async () => { for (;;) { const { value, done } = await reader.read(); if (done) break; buf = Buffer.concat([buf, Buffer.from(value)]); drain(); } })();
async function waitFor(p: () => boolean, ms: number) { const t = Date.now(); while (!p() && Date.now() - t < ms) await Bun.sleep(50); }
await send({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { processId: process.pid, rootUri, capabilities: {}, workspaceFolders: [{ uri: rootUri, name: 'v' }] } });
await waitFor(() => initd, 10000);
await send({ jsonrpc: '2.0', method: 'initialized', params: {} });
await send({ jsonrpc: '2.0', method: 'textDocument/didOpen', params: { textDocument: { uri: docUri, languageId: 'slint', version: 1, text: docText } } });
await Bun.sleep(1500);
await send({ jsonrpc: '2.0', id: 2, method: 'shutdown' }); await send({ jsonrpc: '2.0', method: 'exit', params: {} });
await Bun.sleep(200); sink.end(); proc.kill(); await rl.catch(() => {});
console.log(`diagnostics (${diags.length}): ${diags.join(' | ') || '(none)'}`);
```

Patterns that fail (experimental builtin not recognised):

```bash
# Raw binary, env var unset -> "Unknown element 'FlexboxLayout'"
bin=$(mise which slint-lsp)
env -u SLINT_ENABLE_EXPERIMENTAL_FEATURES bun lsp-flex.ts "$PWD" "$PWD/flex.slint" "$bin"
# => diagnostics (1): Unknown element 'FlexboxLayout'
```

Patterns that work cleanly (recognised,
 zero diagnostics):

```bash
# Env var set in the LSP process environment
SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 bun lsp-flex.ts "$PWD" "$PWD/flex.slint" "$bin"
# => diagnostics (0): (none)

# Injected via /usr/bin/env as the launcher (shell var stays unset)
env -u SLINT_ENABLE_EXPERIMENTAL_FEATURES \
  bun lsp-flex.ts "$PWD" "$PWD/flex.slint" /usr/bin/env SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 "$bin"
# => diagnostics (0): (none)

# Injected by mise from a config [env] active at cwd (shell var stays unset)
#   (a throwaway mise.toml with [env] SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1")
env -u SLINT_ENABLE_EXPERIMENTAL_FEATURES \
  bun lsp-flex.ts "$PWD" "$PWD/flex.slint" mise exec -- "$bin"
# => diagnostics (0): (none)
```

## Verified workarounds

The editor must satisfy two conditions at once:
 run a slint-lsp new enough to know
the element (the pinned 1.17.0,
 not the plugin's bundled 1.16.0),
 and run it with
`SLINT_ENABLE_EXPERIMENTAL_FEATURES` in its environment.
 Point the plugin at the
external LSP (Settings,
 Languages and Frameworks,
 Slint,
 enable external LSP),
then choose one injection method.

1. `env` launcher through the plugin's existing path and args fields (confirmed
   working in IntelliJ).

   ```text
   Slint-lsp path: /usr/bin/env
   Arguments:      SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 /home/user/.local/share/mise/shims/slint-lsp
   ```

   The plugin runs `env SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 <slint-lsp> <its own
   -I/--style/...>`;
    `env` sets the var then execs slint-lsp with the rest.
   Tradeoffs:
    the LSP path is a machine-specific absolute path baked into IDE
   settings;
    using the mise shim (`.../mise/shims/slint-lsp`) keeps it tracking
   the mise tool pin instead of a frozen install path;
    this bypasses mise `[env]`
   entirely,
    so it does not depend on the LSP's working directory.

2. mise launcher so a mise `[env]` is the source of truth.

   ```text
   Slint-lsp path: /home/user/.local/bin/mise
   Arguments:      exec -- slint-lsp
   ```

   with `SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1"` in a mise `[env]` active at the
   LSP's working directory.
    Tradeoff:
    mise `[env]` is directory-activated,
    so this
   only works if the plugin gives the LSP the directory whose mise config carries
   the var (the monorepo root).
    In practice this did not take effect in IntelliJ,
   consistent with the plugin's LSP cwd not being the repo root;
    prefer method 1
   unless that cwd is confirmed.
    The repo sets this var in the generated root
   `mise.toml [env]` (via the `envSection` template in `file-enforcer.config.ts`),
   which covers `mise exec` and builds run from the repo root.

3. systemd user environment,
    for a session-wide setting independent of the editor.

   ```ini
   # ~/.config/environment.d/slint.conf
   SLINT_ENABLE_EXPERIMENTAL_FEATURES=1
   ```

   Then log out and back in.
    Tradeoff:
    global to the whole graphical session
   (every app inherits it;
    harmless here),
    and it needs a relogin.

## What does not work

- Setting the var only in mise config and pointing the plugin at the raw binary:
  mise `[env]` is directory-activated,
   not a persistent system variable,
   so a
  desktop-launched IntelliJ does not inherit it.
   Verified:
   the mise-launcher
  route (method 2) produced no effect in IntelliJ,
   while `mise exec` from the
  repo root in a shell did inject the var,
   isolating the cause to the LSP's
  working directory rather than mise.
- A plugin setting for the env var:
   none exists.
   `SlintLspSettings` has no
  environment field and `CommandLineHandler` never calls `withEnvironment`.
- A slint-lsp CLI flag:
   there is no `--experimental`.
   The only runtime gate is
  the env var.
- Keeping the plugin's bundled slint-lsp (1.16.0):
   it lacks `FlexboxLayout`
  regardless of the env var;
   an external 1.17 LSP is required.

## Why we do not file this upstream

Default policy is do not file.
 Both involved tools are checked against the five
constraints;
 neither qualifies.

slint-lsp (`slint-ui/slint`):

1. Upstream's fault?
    No. Stripping experimental builtins from the default
   register (`typeregister.rs:660`) and gating restoration on
   `SLINT_ENABLE_EXPERIMENTAL_FEATURES` (`lib.rs:248`,
    `main.rs:451`) is
   deliberate design,
    not a defect.
    The feature is experimental by intent.
2. Can they fix?
    Not applicable;
    there is nothing to fix.
3. Supporting the use case?
    The env var is the documented switch for exactly
   this;
    it works as designed.
4. Will they fix?
    Not applicable.
5. Minimal fix prototyped?
    Not applicable;
    no defect to fix.

Constraint 1 fails,
 so no prototype and no issue.

`kizeevov/slint-idea-plugin`:

1. Upstream's fault?
    Soft no. The plugin already exposes `path` and `args`,
    and
   the `/usr/bin/env` launcher (workaround 1) injects the env var through those
   existing fields.
    The use case is achievable today,
    so the absent dedicated
   env field is a convenience gap,
    not a fault.
    Bundling a stable 1.16 release as
   the default is reasonable.
2. Can they fix?
    Yes,
    trivially (add a settings map and a `withEnvironment`
   call),
    but see constraint 1.
3. Supporting the use case?
    Yes,
    through external-LSP path plus args.
4. Will they fix?
    No signal either way.
5. Minimal fix prototyped?
    Not warranted,
    because the workaround uses shipped
   features and constraint 1 does not hold.

Constraint 1 does not hold,
 so the auto-prototype trigger (constraints 1 to 4
holding or sorta-holding) is not met.
 No prototype,
 no issue.

Duplicate search (recorded per policy):
 `gh search issues --repo slint-ui/slint
'FlexboxLayout LSP experimental' --state all` and
`gh search issues --repo kizeevov/slint-idea-plugin 'environment variable
experimental'` (and a broader `env` query) both returned no matches on
2026-06-04.
 No existing thread to comment on.
 No new issue is drafted,
 because
neither tool has a fileable defect;
 the user-facing problem is solved at our
boundary by the editor configuration above.
