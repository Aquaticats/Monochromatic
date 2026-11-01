# Troubleshooting Guide

This troubleshooting guide has been organized into focused categories for easier navigation and maintenance.

## Categories

### [TypeScript & Configuration](typescript.md)

Issues related to TypeScript configuration,
 dprint,
 and other development tools:

- TypeScript path warnings with dprint
- [Learning Rust canonical HTML conflicts with dprint and Stylelint](dprint.md#bug-4-learning-rusts-canonical-compact-html-conflicts-with-repository-formatting-policy)
- Type predicate assignment errors with complex conditional types

### [Testing](testing.md)

Problems with test execution and test suite organization:

- Duplicate describe blocks causing missing or misattributed test output
- Test suite organization best practices
- [Sinon drops an injected undefined exception](sinon-undefined-exception-sentinel.md)
- [chai 6.3.0 `truncateThreshold` cuts failing compare values](chai-failure-value-truncation.md),
   and any finite replacement threshold is non-monotonic

### [Editor Setup](editors.md)

Editor installation and configuration issues:

- VS Code development container setup
- Helix editor installation on Debian/Ubuntu
- Editor-specific configuration problems

### [Dependencies & Package Management](dependencies.md)

Package management and dependency resolution problems:

- pnpm workspace configuration issues
- Dependency resolution conflicts
- [PNPM offline verifier metadata mirrors](pnpm-offline-verifier-metadata-mirrors.md)
- [Pi update allowScripts warnings](pi-update-allow-scripts.md)

### [Performance Optimization](performance.md)

Performance-related issues and optimizations:

- Build preparation optimization strategies
- WSL performance improvements
- File system vs binary execution trade-offs

### [Configuration Format Issues](toml.md)

Problems with configuration file formats and why TOML can be problematic:

- TOML table repetition and silent overwrites
- Configuration debugging nightmares
- Better alternatives to TOML (JSON,
   YAML,
   JSON5)
- Real-world examples of TOML-caused production incidents

### [Figma and browser automation](figma-browser-automation.md)

The WebGL wall:
 why AI agents cannot meaningfully automate Figma through browser tools:

- Canvas content is invisible to DOM inspection and browser automation
- Figma MCP covers node structure but not comments,
   prototypes,
   or annotations
- No automated solution exists for prototype review or comment extraction
- Workarounds require human-in-the-loop screenshotting

### [pi-safeguard](pi-safeguard.md)

Security guardrail false positives and judge model selection:

- `pathSignals` flags every file as a system path when home is under `/var/home/` (Fedora,
   NixOS)
- pi-budget-model fails to find a judge model when the active model is the latest major version

### [Pi auto-mode repeated Git path probes](pi-auto-mode-repeated-git-path-probes.md)

Why guarded reads emit missing-Git debug records for expected workspace bin paths:

- auto-mode probes every `PATH` directory concurrently before selecting real Git
- linked-worktree allowlist discovery repeats resolution for each guarded read
- broad workspace-bin `PATH` inheritance is expected and needed,
   not the defect

### [Pi 0.87 extension API breakage](pi-0-87-extension-api-breakage.md)

Why auto-mode failed every guarded call and reviewers lost their prompt after the Pi 0.87 update:

- hand-copied transcript projection threw on the new `system` message role
- direct provider dispatch skipped `normalizeContext()`,
   sending no system prompt and no tools
- silent `lint:types` hid the accompanying type errors

### [`lint:types` silent pass](task-tsc-shared-chunk-silent-pass.md)

Why every package type check exited 0 without running `tsc`:

- `testing.ts` imported the `tsc-filter.ts` command-line entry
- rolldown moved its `import.meta.main` guard into a shared chunk where it is false

### [Pi goal stale global blocker](pi-goal-stale-global-blocker.md)

Retired `@narumitw/pi-goal` behavior and migration:

- paused or interrupted goals block unrelated built-in and custom tools
- cached package files are not active configuration
- migrate globally to repository-owned `@monochromatic-dev/pi-goal`

### [OpenAI OAuth requests hit Cloudflare 520](openai-api-oauth-cloudflare-520.md)

Why pi's OpenAI models fail while `status.openai.com` reads fully operational:

- valid ChatGPT OAuth tokens get Cloudflare `520` origin errors on `api.openai.com`
- unauthenticated and invalid-credential requests to the same host still return clean `401` JSON
- the same model ids answer through the `llmgateway-devpass` provider and through `codex exec`

### [C-Like Comment Syntax](c-like-comments.md)

Issues with C-style comment syntax that affect multiple programming languages:

- Block comment nesting limitations and why `/* */` comments cannot be nested
- Commenting out code that contains existing block comments
- IDE comment features and best practices
- Language-specific comment syntax differences

### [Aquascope connectors in Firefox](aquascope-firefox-connectors.md)

Aquascope interpreter connector lines can disappear in Firefox:

- Chromium paints generated LeaderLine SVG connectors while an observed Firefox view omits them
- Pointer dots and targets still describe the same embedded interpreter state
- Exact Firefox rendering mechanism remains unconfirmed

### [iOS Safari touch events](ios-safari-touch.md)

iOS Safari claims `touch-action: none` support but does not reliably honor it:

- `pointercancel` fires despite `touch-action: none` on the target element
- Long-press triggers system context menu instead of application gesture
- Three-part workaround:
   `touchstart`/`touchmove` preventDefault,
   `pointerdown` preventDefault,
   `-webkit-touch-callout: none`
- Twelve years of WebKit bug 133112 and counting

### [Bundling](bundling.md)

Client-side bundling issues where Node.
js code leaks into browser bundles:

- `node:` protocol dynamic imports cause CORS errors in browser consoles
- Environment guards for dual-target modules

### [Barrel `export *` drops an ambiguous name](barrel-star-export-drops-ambiguous-names.md)

A symbol exported by two barrels vanishes from the package at runtime:

- `oxlint` and the bundler pass,
  and the built file contains both function bodies
- `tsc` DOES catch it,
  as TS2308,
  for values as well as types;
   an earlier claim to the contrary is corrected in the document with a reduction
- The only symptom otherwise is `does not provide an export named`
   at an import three steps away
- How to ask the built package what it actually exports

### [A finished process that will not exit](finished-process-does-not-exit.md)

A run wrote its results,
printed its completion line,
and held the process for three more minutes:

- `Promise.race` does nothing to the loser,
  and `wait` is a bare `setTimeout` with no handle,
   so a won race leaves a live timer holding the event loop
- The first diagnosis blamed leaked sockets and was wrong;
   the source said otherwise in a minute
- Why the leak test had to spawn a child process rather than count timers in-process

### [NUL bytes land invisibly in source](nul-bytes-land-invisibly-in-source.md)

A control character written where a space was intended,
committed,
and found only when an `Edit` would not apply:

- `rg` and `grep --perl-regexp` both report nothing on a file full of them
- `tr --delete --complement`,
  `od --format=c` and `cat --show-all` do find them
- Why a working NUL separator is still a defect,
   and the shared escaped constant that fixes it

### [V8 first-call compile at the stack edge](v8-first-call-compile-at-the-stack-edge.md)

A handler that turns a stack exhaustion into its own error
let the engine's `RangeError` escape instead:

- V8 compiles a function on its first call,
  and a compile with too little stack raises the stack-exhaustion `RangeError`
- The failure depended on load and test order,
  and a smaller `--stack-size` did not reproduce it
- The module-load warm-up that fixed it,
  and what it still cannot cover

### [dprint-plugin-exec](dprint-exec.md)

Why the exec plugin silently does nothing and why certain tools are incompatible:

- Plugin name resolution selects only one plugin per extension;
   chaining requires `associations`
- Include-only associations exclude extension matches for unlisted extensions
- Exec reads stdout as formatted content;
   in-place file modifiers (oxlint --fix) are incompatible
- `cacheKeyFilesHash: null` in resolved config is expected (hash is moved to global `cacheKey`)

### [mise detached-worktree trust](mise-detached-worktree-trust.md)

A detached worktree fails `mise run` before task execution when its copied config requires trust and its new canonical
path is not trusted:

- trust is bound to canonical config paths,
  not equal Git content;
- command-scoped `MISE_TRUSTED_CONFIG_PATHS` trusts one inventoried disposable root without persistent trust-state
  mutation,
  but descendants can inherit the variable;
- dependency availability remains a separate prerequisite after trust succeeds.

### [mise watch](mise-watch.md)

mise watch flag forwarding bugs and dev-mode restart suppression:

- `mise watch` silently drops `--no-meta` and `-J` (filter-prog) flags
- Unnecessary restarts on metadata-only or same-content file writes
- Server restart generates fresh auth token,
   breaking client reconnection

### [dsh fixture-profile plugin verification](dsh-fixture-profile-plugin-verification.md)

Disposable dsh fixture profiles for plugin verification
and the surfaces they miss out of the box:

- Headless-template sessions mount no `cordis_inspect_*` tools
   until host and preset rows are inserted
- Fixture profiles resolve no live route credentials;
   `MISSING_CREDENTIAL` sanctions exporting keys
   in the launching environment
- Removing `apiKeyEnv` from a patch-declared provider
   fails with `PI_AI_ERROR: Provider is not configured`

### [`cordis_inspect_query` input coerced to string](dsh-cordis-inspect-query-input-coerced-to-string.md)

dsh 0.2.1-alpha.4 inspect marshaling defect:

- Every filtered or paged `cordis_inspect_query` call fails with
   `Host Cordis inspect ... rejected input: "input" must be an object`
- Input-less discovery still works,
   so unfiltered `Tool.listTools` remains usable

### [Bash and CLI](bash.md)

Bash shell and CLI tool quirks that cause confusing behavior:

- `2>&1 > file` splits stderr and stdout instead of merging them,
   producing interleaved output that misrepresents execution order
- rg `--glob` finds files but `-l` with a content pattern does not (content-vs-filename search confusion)
- [Clack note formatter snapshots need newer nested `styleText()` behavior](clack-note-nested-styletext-node-floor.md)
- [GitHub CLI implicit repository lookup invokes a PATH-shadowed Git wrapper](gh-implicit-repository-git-wrapper.md)
- [GitHub CLI JSON readbacks rewrite literal control escapes](github-cli-json-control-escapes.md)
- [A `pgrep --full` wait loop matches its own shell and never exits](pgrep-wait-loop-matches-itself.md),
   so the step it was gating silently never runs
- [GitHub REST Issue creation should be serial and paced](github-issue-creation-concurrency.md)
- [GitHub REST omits its Issue title length limit](github-issue-title-length.md)
- [nano-spawn timeout is not a hard child-process deadline](nano-spawn-timeout-is-not-a-hard-deadline.md)
- [OpenCodeReview does not create regular GitHub Issues from findings](open-code-review-github-issue-routing.md)

### [CLI bin entries](cli-bin.md)

Problems with CLI tools installed via `node_modules/.bin/`:

- Missing shebang causes Unix to fall back to `/bin/sh`,
   triggering ImageMagick `import` hangs
- Cross-platform shebang and `.cmd` wrapper interaction

### [VSCode-Specific](vscode.md)

VSCode-specific troubleshooting (already exists):

- Multiple workspace instances in WSL
- Remote development setup
- VSCode extension issues

### [virt-manager SPICE auto resize](virt-manager-spice-agent-xwayland-resize-race.md)

Guest resize failures in a labwc session using xwayland-satellite:

- `spice-vdagent.service` races xwayland-satellite and exits against stale
  `DISPLAY=:0`
- A user-unit dependency and explicit `DISPLAY=:12` restore the SPICE channel
- Cold-reboot and multiple-window-size verification

### [QEMU virtio-gpu host-memory capsets](qemu-11-virtio-gpu-hostmem-capsets.md)

GPU and display capability adoption for the labwc migration VM:

- A `qemu:override` adds a 2 GiB host-memory window,
  Venus,
  and AMD native context
- libvirt's default `spawn=deny` seccomp policy blocks the Venus render server
- A session-driver workaround enables reboot-verified OpenGL 4.6,
  Venus Vulkan,
  native RADV,
  and the existing SPICE resize path

### Comment-preserving structured edits in Rust

Quirks found while choosing TOML,
JSONC,
and XML editing crates for the monorepo manager
(`doc/planning/monorepo-manager-from-scratch-design.md`,
"Managed file editing"):

- [`toml_edit` 0.25.15 drops or moves comments](toml-edit-comment-loss.md)
  on index assignment,
  removal,
  and `Table::insert`
- [taplo 0.14.0 rejects TOML 1.1 inline-table trailing commas](taplo-toml-1-1-inline-table-trailing-comma.md)
- [`json-five` 0.3.1 drops block comments' closing slash](json-five-unterminated-block-comment.md)
  and panics on multibyte line-comment ends
- [`jsonc-parser` 0.33.2 parses loose syntax by default](jsonc-parser-json5-defaults.md)
  and ignores `allow_comments: false` in the CST parser
- [Biome JSON crates 0.5.7 need exact dependency pins](biome-json-crates-exact-pins.md)
  after breaking patch releases
- [XML serializers write literal tabs and newlines into attribute values](xml-attribute-whitespace-serialization.md),
  which reparse as spaces
- [`hcl-edit` drops comments inside binary operations](hcl-edit-binary-operator-decor.md)
- [`hcl-edit` rewrites `<<-` heredoc introducers and body indentation](hcl-edit-heredoc-dedent.md)
- [`hcl-rs` evaluation diverges from the HCL reference](hcl-rs-eval-arithmetic-and-iteration-order.md)
  on arithmetic,
  object iteration order,
  and `% 0`

### Hash selection and cross-machine probes

Found while choosing meow's cache key hash and measuring it on a second machine:

- [`gxhash` needs the `aes` target feature](gxhash-aes-target-feature.md)
  and has no software fallback (historical:
  no repository code uses `gxhash` any more)
- [Rust hash crates hide backends behind features and cfg](rust-hash-crate-build-and-test-quirks.md),
  and some test suites pass while testing nothing
- [`rustup` proxies auto-install into the default home](rustup-proxy-auto-install-default-home.md)
  when a chained remote step drops `RUSTUP_HOME`
- [macOS `pmset -g thermlog` never returns](macos-pmset-thermlog-never-returns.md);
  `pmset -g therm` is the one-shot form
- [`rustix` configures `fs::fadvise` out on Apple targets](rustix-apple-fadvise-gate.md)

## Quick Links

For common issues:

- **Slow builds?
  ** → [Build performance](performance.build.md)
- **Test failures not showing?
  ** → [Duplicate describe blocks](testing.md#duplicate-describe-blocks-causing-missing-or-misattributed-test-output)
- **A failing comparison prints `[ …(N) ]` instead of the values?
  ** → [chai `truncateThreshold` cut](chai-failure-value-truncation.md)
- **TypeScript path warnings?
  ** → [dprint configuration](typescript.md#typescript-path-warnings-with-dprint)
- **Learning Rust HTML fails dprint and Stylelint?
  ** → [Canonical HTML source-policy conflict](dprint.md#bug-4-learning-rusts-canonical-compact-html-conflicts-with-repository-formatting-policy)
- **`Cannot find name` for DOM types in a non-browser package?
  ** → [All packages must extend config-typescript/dom](typescript.md#all-packages-must-extend-config-typescriptdom)
- **Type errors from `node_modules` JSR packages?
  ** → [JSR `.ts` files and `skipLibCheck`](typescript.md#jsr-packages-ship-ts-source-files-that-skiplibcheck-cannot-skip)
- **Configuration not working as expected?
  ** → [TOML issues and alternatives](toml.md#silent-overwrites-the-configuration-killer)
- **Figma automation not working?
  ** → [The WebGL wall](figma-browser-automation.md)
- **CORS errors for `node:` imports in browser?
  ** → [node: protocol in bundles](bundling.md#node-protocol-imports-cause-cors-errors-in-browser-bundles)
- **CLI hangs with ImageMagick errors?
  ** → [Missing shebang](cli-bin.md#cli-command-hangs-on-unix-with-imagemagick-errors)
- **pi-safeguard blocks every file?
  ** → [pathSignals false positive on /var/home](pi-safeguard.md#pi-safeguard-flags-every-file-under-varhome-as-a-system-path)
- **Unrelated Pi tools blocked after `/goal` interruption?
  ** → [Retired goal package blocker](pi-goal-stale-global-blocker.md)
- **Aquascope pointer lines missing in Firefox?
  ** → [Aquascope connector discrepancy](aquascope-firefox-connectors.md)
- **Touch gestures broken on iPhone?
  ** → [iOS Safari touch-action betrayal](ios-safari-touch.md)
- **dprint exec plugin not running?
  ** → [Plugin silently does nothing](dprint-exec.md#exec-plugin-silently-does-nothing)
- **Dev server restarts on every save?
  ** → [mise watch flag dropping + content-hash filter](mise-watch.md)
- **Output order looks wrong with `2>&1 > file`?
  ** → [Redirect ordering splits streams](bash.md#2>&1--file-splits-stderr-and-stdout-producing-interleaved-output)
- **rg missing files with spaces in paths?
  ** → [Content-vs-filename search confusion](bash.md#rg---glob-finds-files-but--l-with-a-content-pattern-does-not)
- **GitHub issue commands fail in the repository's Git wrapper?
  ** → [Pass explicit repository context](gh-implicit-repository-git-wrapper.md)
- **Need to route OpenCodeReview findings into GitHub Issues?
  ** → [Add a fail-closed adapter](open-code-review-github-issue-routing.md)
- **`pi update` reintroduces `@google/genai`,
   `koffi`,
   or `protobufjs`?
  ** →
  [Pi update allowScripts warnings](pi-update-allow-scripts.md)
- **virt-manager console does not resize a labwc guest?
  ** →
  [Order spice-vdagent after xwayland-satellite](virt-manager-spice-agent-xwayland-resize-race.md)
- **labwc VM uses virgl but has no hardware Vulkan?
  ** →
  [Add virtio-gpu host-memory capsets](qemu-11-virtio-gpu-hostmem-capsets.md)
- **Slint nested Wayland client still inherits host dark or light preference?
  ** →
  [Serve a private XDG Settings portal](slint-nested-color-scheme-portal.md)
- **Mise inline TypeScript task reports an already-declared imported binding?
  ** →
  [Keep interpolated imports in one scope owner](mise-inline-task-import-collision.md)
- **Mise inline Node task receives no declared usage arguments?
  ** →
  [Parse mise's `usage_args` value](mise-usage-args-inline-node.md)
- **Nested Wayland quit makes hosted winit client report broken pipes?
  ** →
  [Close the client before stopping the compositor](nested-wayland-client-shutdown-order.md)
- **Escape does not reach applications inside Ghostty?
  ** →
  [Distinguish keybind consumption from PTY encoding](ghostty-escape-keybind-consumption.md)
- **Escape fails across Firefox and Ghostty after agent input automation?
  ** →
  [Reset interrupted ydotool virtual-key state](ydotool-interrupted-key-release.md)
- **Caller `AbortSignal` getters leak error metadata into an operation?
  ** →
  [Own live signal observations](node-abort-signal-owned-observation.md)
- **Android `uiautomator dump` cannot get idle state while an app loads?
  ** →
  [Wait for stable rendered content](android-uiautomator-dump-idle-state.md)

- **Definition-order indexes vanish in persisted preparation inputs?
  ** →
  [Use explicit JSON data for root definition indexes](translation-repair-root-order-serialization.md)

- **A rendered check shows prose and code swapped inside one long paragraph?
  ** →
  [Keep every code span on one source line](marked-list-html-line-ends-item.md)

## Contributing

When adding new troubleshooting content:

1. Choose the most appropriate category file
2. Follow the existing format with clear problem/solution structure
3. Include reproduction steps and root cause analysis when possible
4. Update this index file if adding new categories
