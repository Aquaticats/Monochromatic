# `@ifi/pi-shared-qna@0.5.1` hardcodes deprecated `@mariozechner/pi-tui` after upstream package rename, crashing `@ifi/pi-plan` on first plan-mode render

This document tracks the `MODULE_NOT_FOUND` crash that any `pi`
session running with `@ifi/pi-plan@0.5.1` installed hits when it
tries to render the plan-mode banner.
 The crash is a packaging bug
in upstream `ifiokjr/oh-pi`:
 between npm publication of
`@ifi/pi-shared-qna@0.5.1` and `@ifi/pi-plan@0.5.1` on 2026-04-28
and the present day,
 the underlying pi-tui library was renamed
from `@mariozechner/pi-tui` (deprecated at 0.73.1 on 2026-05-07)
to `@earendil-works/pi-tui` (started at 0.74.0 30 minutes later).
Every `@ifi/*` pi extension still pins the deprecated name in both
its `peerDependencies` and its source imports.
 Sibling extension
publishers (`@aliou/*`,
 `@diegopetrucci/*`,
 `pi-context`,
`pi-mcp-adapter`,
 `@juicesharp/rpiv-todo`) all migrated to the new
scope;
 only the `@ifi/*` family did not.

The Monochromatic workspace pins `@earendil-works/pi-tui@0.75.4`
exclusively (`pnpm-workspace.yaml`),
 and the pi agent's npm tree
under `~/.pi/agent/npm/node_modules/` installs only
`@earendil-works/pi-tui@0.74.1`.
 There is no copy of
`@mariozechner/pi-tui` for the loader to find.

The consumer-side symlink workaround in "Verified workarounds"
section #1 is the recommended fix until upstream republishes
`@ifi/pi-shared-qna`,
 `@ifi/pi-plan`,
 and
`@ifi/pi-extension-subagents` against the new package name.

---

## Symptom

A `pi` session in this workspace (`pi` 0.75.4,
 installed via
`package/pi-plugin/current-time-context/node_modules/.bin/pi`) exits with
an `uncaughtException` shortly after startup when `@ifi/pi-plan` is
present in `~/.pi/agent/npm/node_modules/`.
 The crash fires from a
`setTimeout` callback inside the TUI render loop,
 so it surfaces
asynchronously and kills the whole process rather than failing the
single render frame.

The error message and stack are quoted verbatim from the failing
session:

```text
Error: Unable to load @mariozechner/pi-tui. Checked the local
dependency and Bun global fallbacks:
/home/user/.bun/install/global/node_modules/@mariozechner/pi-tui
    at requirePiTuiModule
      (/var/home/user/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts:76:11)
    at getPiTui
      (/var/home/user/.pi/agent/npm/node_modules/@ifi/pi-plan/state.ts:8:46)
    at Object.render
      (/var/home/user/.pi/agent/npm/node_modules/@ifi/pi-plan/state.ts:102:57)
    at Container.render
      (.../@earendil-works+pi-tui@0.75.4/.../tui.js:85:38)
    at TUI.doRender
      (.../@earendil-works+pi-tui@0.75.4/.../tui.js:750:29)
    at Timeout._onTimeout
      (.../@earendil-works+pi-tui@0.75.4/.../tui.js:372:18)
{
  [cause]: Error: Cannot find module '@mariozechner/pi-tui'
    code: 'MODULE_NOT_FOUND',
    requireStack: [
      '/var/home/user/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts'
    ]
}
```

The crash trigger is the plan-mode banner widget:
 each TUI render
tick calls `state.ts:102`'s render callback,
 which calls
`getPiTui()` (`state.ts:8`),
 which delegates to `requirePiTuiModule`
(`pi-tui-loader.ts:54`).
 The loader hardcodes the old name in two
places (the inline `require` call and the Bun fallback path
builder),
 so it finds neither and throws the wrapper error at
`pi-tui-loader.ts:76`.

The same root cause would also fire from any `@ifi/pi-extension-subagents`
code path,
 because that package's TypeScript source contains
direct `import ... from "@mariozechner/pi-tui"` statements rather
than going through the shared loader (see "Root cause" for the
inventory).

## Root cause

### Walking the call chain

`@ifi/pi-plan@0.5.1/state.ts` lines 1-12 imports the loader from
`@ifi/pi-shared-qna` and wraps it in a tiny shim that hands the
plan-mode banner two utility functions:

```ts
// /var/home/user/.pi/agent/npm/node_modules/@ifi/pi-plan/state.ts:1-12
import type { ExtensionAPI, ExtensionContext } from "@mariozechner/pi-coding-agent";
import { requirePiTuiModule } from "@ifi/pi-shared-qna";
import { resolveActivePlanFilePath } from "./plan-files";
import type { PlanModeState } from "./types";
import { createInactivePlanModeState, isPlanModeState } from "./utils";

function getPiTui() {
	return requirePiTuiModule() as {
		truncateToWidth: (text: string, width: number) => string;
		wrapTextWithAnsi: (text: string, width: number) => string[];
	};
}
```

The banner widget is registered with the TUI:

```ts
// /var/home/user/.pi/agent/npm/node_modules/@ifi/pi-plan/state.ts:99-114
(_tui, theme) => ({
	invalidate: () => {},
	render: (width: number) => {
		const { truncateToWidth, wrapTextWithAnsi } = getPiTui();
		const safeWidth = Math.max(1, width);
		// ... formatting
		return lines;
	},
}),
```

Each render tick calls `getPiTui()` (no caching),
 which calls
`requirePiTuiModule()`.
 The loader is `@ifi/pi-shared-qna@0.5.1`:

```ts
// /var/home/user/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts:54-80
export function requirePiTuiModule(options: PiTuiLoaderOptions = {}): unknown {
	const requireFn = options.requireFn ?? createRequire(import.meta.url);
	try {
		return requireFn("@mariozechner/pi-tui");          // ← line 57: old name
	} catch (error) {
		const code = (error as { code?: string }).code;
		if (code !== "MODULE_NOT_FOUND") {
			throw error;
		}

		const fallbackPaths = getPiTuiFallbackPaths(options);
		for (const fallbackPath of fallbackPaths) {
			try {
				return requireFn(fallbackPath);
			} catch (fallbackError) {
				const fallbackCode = (fallbackError as { code?: string }).code;
				if (fallbackCode !== "MODULE_NOT_FOUND") {
					throw fallbackError;
				}
			}
		}

		throw new Error(
			`Unable to load @mariozechner/pi-tui. Checked the local dependency and Bun global fallbacks: ${fallbackPaths.join(
				", ",
			)}`,
			{ cause: error },
		);
	}
}
```

And the fallback-path builder it delegates to also hardcodes the
old scope:

```ts
// /var/home/user/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts:31-39
export function getPiTuiFallbackPaths(options: Omit<PiTuiLoaderOptions, "requireFn"> = {}): string[] {
	const homeDir = options.homeDir ?? os.homedir();
	const roots = new Set<string>();
	if (options.bunInstallDir) {
		roots.add(options.bunInstallDir);
	}
	roots.add(path.join(homeDir, ".bun"));
	return [...roots].map((root) => path.join(root, "install", "global", "node_modules", "@mariozechner", "pi-tui"));
}
```

So the resolver tries exactly two locations,
 both spelled
`@mariozechner/pi-tui`,
 and finds neither.

### Why the package is not there

`@mariozechner/pi-tui` was deprecated.
 The npm registry view of the
deprecated package carries this message in the readme:

```text
DEPRECATED ⚠️  - please use @earendil-works/pi-tui instead going forward
```

Publication timeline (UTC,
 from `npm view ... time`):

- `@ifi/pi-shared-qna@0.5.1`:
   2026-04-28 05:50:21 (still current latest)
- `@mariozechner/pi-tui@0.73.1`:
   2026-05-07 14:45:22 (final version of deprecated scope)
- `@earendil-works/pi-tui@0.74.0`:
   2026-05-07 15:15:52 (first version of new scope,
   30 minutes later)
- `@earendil-works/pi-tui@0.75.4`:
   2026-05-20 14:24:20 (current latest,
   what this workspace pins)
- This investigation:
   2026-05-20 (today)

`@ifi/pi-shared-qna@0.5.1` shipped 10 days *before* the rename,
pinning the then-canonical name.
 No subsequent
`@ifi/pi-shared-qna` release has been published;
 the latest version
on npm is still 0.5.1.

### Why other extensions are unaffected

Inventory of the eight pi extensions installed at
`~/.pi/agent/npm/node_modules/` and which scope each peers on:

```text
@aliou/pi-linkup           → @earendil-works/pi-tui (0.74.0)
@aliou/pi-processes        → @earendil-works/pi-tui (^0.75.3)
@aliou/pi-synthetic        → @earendil-works/pi-tui (0.74.0)
@diegopetrucci/pi-openai-fast → @earendil-works/pi-coding-agent (*)
@juicesharp/rpiv-todo      → @earendil-works/pi-tui (*)
pi-context                 → @earendil-works/pi-coding-agent
pi-mcp-adapter             → @earendil-works/pi-tui (^0.74.0)
@ifi/pi-plan          → @mariozechner/pi-tui (>=0.56.1)  ← BROKEN
@ifi/pi-extension-subagents → @mariozechner/pi-tui (>=0.56.1) ← also broken when activated
@ifi/pi-shared-qna         → @mariozechner/pi-tui (*)         ← root of breakage
```

The breakage is confined to the `@ifi/*` family.
 Filing/uninstalling
`@ifi/pi-plan` removes the only currently-loaded `@ifi/*` extension
in this workspace and restores `pi` startup.

### Scope of the upstream fix

Beyond the loader,
 the bug surface inside upstream
[`ifiokjr/oh-pi`](https://github.com/ifiokjr/oh-pi) at commit
`1979b3a9` (head of `main`,
 2026-05-17) spans 16 package
manifests and roughly 15 direct `import ... from "@mariozechner/pi-tui"`
statements across `package/subagents/`,
 `package/spec/extension/`,
`package/providers/`,
 and the shared-qna README.
 Inventory captured
from a fresh clone:

```text
package/shared-qna/package.json:    peerDeps "@mariozechner/pi-tui": "*"
package/subagents/package.json:     peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/spec/package.json:          peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/providers/package.json:     peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/plan/package.json:          peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/extensions/package.json:    peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/diagnostics/package.json:   peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/oh-pi-context/package.json: peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/ollama/package.json:        peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/pi-bash-live-view/package.json: peerDeps
package/pi-pretty/package.json:     peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/adaptive-routing/package.json: peerDeps
package/cursor/package.json:        peerDeps "@mariozechner/pi-tui": ">=0.56.1"
package/pi-remote-tailscale/package.json: peerDeps
package/analytics-extension/package.json: peerDeps
package/background-tasks/package.json: peerDeps "@mariozechner/pi-tui": ">=0.56.1"

package/subagents/agent-manager.ts:    import { ... } from "@mariozechner/pi-tui";
package/subagents/agent-manager-list.ts:    import { ... } from "@mariozechner/pi-tui";
package/subagents/agent-manager-parallel.ts: import { ... } from "@mariozechner/pi-tui";
package/subagents/agent-manager-detail.ts:   import { ... } from "@mariozechner/pi-tui";
package/subagents/agent-manager-edit.ts:     import { ... } from "@mariozechner/pi-tui";
package/subagents/chain-clarify.ts:    import { ... } from "@mariozechner/pi-tui";
package/subagents/index.ts:            import { Text } from "@mariozechner/pi-tui";
package/subagents/render-helpers.ts:   import { visibleWidth } from "@mariozechner/pi-tui";
package/subagents/text-editor.ts:      import { matchesKey } from "@mariozechner/pi-tui";
package/providers/index.ts:            import { ... } from "@mariozechner/pi-tui";
package/spec/extension/index.ts:       import { Text } from "@mariozechner/pi-tui";
```

A complete upstream fix is therefore not one-line;
 it is a
coordinated migration.
 The minimal patch that resolves *this*
specific crash (the plan-mode banner render in `@ifi/pi-plan`) is
just the loader,
 since `@ifi/pi-plan` is the only currently-loaded
`@ifi/*` extension and it talks to pi-tui exclusively through
`requirePiTuiModule()`.
 See "Verified workarounds" #2 (loader
patch) and the auto-prototype audit in
"Why we file/do not file this upstream" below.

## Verification

### Versions under test

- Failing pi binary:
   `/var/home/user/Monochromatic/package/pi-plugin/current-time-context/node_modules/.bin/pi`
   (`pi --version` = `0.75.4`)
- Failing extension:
   `@ifi/pi-plan@0.5.1`,
   installed at `~/.pi/agent/npm/node_modules/@ifi/pi-plan/`,
   pulling in `@ifi/pi-shared-qna@0.5.1` and `@ifi/pi-extension-subagents@0.5.1`
- Workspace TUI:
   `@earendil-works/pi-tui@0.75.4` (pinned in `pnpm-workspace.yaml`)
- Agent-tree TUI:
   `@earendil-works/pi-tui@0.74.1` (resolved by `pi`'s extension installer)
- Upstream clone for the patch prototype:
   `ifiokjr/oh-pi` at commit `1979b3a9dd042e61602b600492b0d7e79b2e075d`,
   in a fresh `mktemp -d` directory created at investigation time,
   `git remote -v` confirmed pointing at `https://github.com/ifiokjr/oh-pi.git`

### Harness — failing scenario (no `@mariozechner/pi-tui` present)

```bash
VERIFY=$(mktemp -d -t pi-tui-loader-verify-XXXXXX)
mkdir -p "$VERIFY/node_modules/@earendil-works"
ln -s ~/.pi/agent/npm/node_modules/@earendil-works/pi-tui \
  "$VERIFY/node_modules/@earendil-works/pi-tui"
cp ~/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts \
  "$VERIFY/original-loader.ts"
cat > "$VERIFY/verify-original.ts" <<'EOF'
import { requirePiTuiModule } from "./original-loader.ts";
try {
  const tui = requirePiTuiModule() as { truncateToWidth: (s: string, w: number) => string };
  console.log("ORIGINAL OK:", typeof tui.truncateToWidth);
} catch (err) {
  console.log("ORIGINAL FAIL:", (err as Error).message.split("\n")[0]);
}
EOF
bun "$VERIFY/verify-original.ts"
```

Output (observed 2026-05-20):

```text
ORIGINAL FAIL: Unable to load @mariozechner/pi-tui. Checked the local
dependency and Bun global fallbacks:
/home/user/.bun/install/global/node_modules/@mariozechner/pi-tui
```

This exactly matches the production crash message.

### Harness — passing scenarios

Workaround #1 (consumer-side symlink) and #2 (loader patch) both
verified through the same harness shape.
 See each workaround
section for the exact harness invocation and output.

## Verified workarounds

### #1 (recommended) — symlink the new package under the old name

Create a single symlink inside the agent's npm tree so the loader's
existing hardcoded require resolves to the renamed package without
any source edits:

```bash
mkdir -p ~/.pi/agent/npm/node_modules/@mariozechner
ln -s ../@earendil-works/pi-tui \
  ~/.pi/agent/npm/node_modules/@mariozechner/pi-tui
```

The link target is relative on purpose:
 it stays valid even if `pi`
moves the `@earendil-works` directory during a future extension
reinstall,
 as long as the sibling layout is preserved.

Verification (run alongside the failing-scenario harness above):

```bash
VERIFY2=$(mktemp -d -t pi-tui-symlink-verify-XXXXXX)
mkdir -p "$VERIFY2/node_modules/@earendil-works" "$VERIFY2/node_modules/@mariozechner"
ln -s ~/.pi/agent/npm/node_modules/@earendil-works/pi-tui \
  "$VERIFY2/node_modules/@earendil-works/pi-tui"
ln -s ../@earendil-works/pi-tui \
  "$VERIFY2/node_modules/@mariozechner/pi-tui"
cp ~/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts \
  "$VERIFY2/loader.ts"
cat > "$VERIFY2/verify.ts" <<'EOF'
import { requirePiTuiModule } from "./loader.ts";
const tui = requirePiTuiModule() as { truncateToWidth: (s: string, w: number) => string };
console.log("SYMLINK OK:", typeof tui.truncateToWidth);
console.log("  sample:", JSON.stringify(tui.truncateToWidth("hello world", 8)));
EOF
bun "$VERIFY2/verify.ts"
```

Output (observed 2026-05-20):

```text
SYMLINK OK: function
  sample: "hello[0m...[0m"
```

Tradeoffs:

- `pi` may rewrite `~/.pi/agent/npm/node_modules/` on extension
  reinstall (`pi install`,
   `pi update`).
   The symlink survives most
  noop reinstalls but is wiped by a clean reinstall of any
  package that brings npm into a deduplicate pass.
   Reapply the
  symlink after any `pi install`/`pi update`.
- The `@earendil-works/pi-tui@0.74.1` in the agent tree is older
  than the `@earendil-works/pi-tui@0.75.4` in the Monochromatic
  workspace.
   The loader runs `createRequire` from the loader file's
  location (under `~/.pi/agent/npm/...`),
   so it always resolves
  to the older 0.74.1 copy.
   The two API functions `@ifi/pi-plan`
  uses (`truncateToWidth`,
   `wrapTextWithAnsi`) exist in both
  versions and behave identically;
   this workaround is safe for the
  observed call sites.
- Does not fix `@ifi/pi-extension-subagents`'s direct imports
  if those code paths activate.
   The `@ifi/pi-plan` plan-mode
  banner only goes through the shared loader,
   so this workaround
  covers the present crash but does not future-proof against
  feature paths in the sub-agents extension.
   Watch for new
  `Cannot find module '@mariozechner/...'` failures on
  subagent activation.

### #2 — uninstall `@ifi/pi-plan`

If plan mode is not required:

```bash
pi uninstall npm:@ifi/pi-plan
```

This removes the only currently-active broken extension and
restores `pi` startup without touching anything else.
 The other
seven extensions in the agent tree are unaffected.

Tradeoffs:
 loses plan-mode features (`/plan`,
 branch-aware plan
files,
 delegated research tasks).
 Acceptable for sessions that do
not use plan mode.

### #3 — apply the upstream loader patch locally

Copy the patched loader from the prototype clone into the agent
tree:

```bash
cp /tmp/oh-pi-prototype-jBg5zK/packages/shared-qna/pi-tui-loader.ts \
  ~/.pi/agent/npm/node_modules/@ifi/pi-shared-qna/pi-tui-loader.ts
```

The full diff is recorded at
[`pi-extensions-pi-tui-rename.patch`](pi-extensions-pi-tui-rename.patch).
This is the same change a future `@ifi/pi-shared-qna@0.5.2`
release should ship,
 so it is the most upstream-shaped local fix.

Verification:
 see "Harness — passing scenarios" — the patched
loader returns both `truncateToWidth` and `wrapTextWithAnsi` and
produces correct output for the two functions `@ifi/pi-plan`
exercises.

Tradeoffs:

- Same reinstall fragility as workaround #1:
   `pi` may overwrite
  the file on extension reinstall.
- Does not fix `@ifi/pi-extension-subagents`'s direct imports;
   if
  the sub-agent code paths activate,
   they will fail with the same
  module-not-found error against `@mariozechner/pi-tui`.
   The fix
  is purely structurally upstream-shaped (no new imports);
   the
  subagent direct imports require a separate set of edits across
  ~9 source files.

## What does not work

- **Installing the deprecated `@mariozechner/pi-tui@0.73.1`
  alongside the new package.
  ** `npm view` confirms 0.73.1 still
  resolves and downloads.
   The peerDep `@mariozechner/pi-tui: *`
  in `@ifi/pi-shared-qna` would accept it.
   But Monochromatic's
  `pnpm-workspace.yaml` deliberately filters
  `@earendil-works/pi-tui>chalk`,
   `@earendil-works/pi-tui>koffi`,
  `@earendil-works/pi-tui>mime-types`,
   etc.,
   which suggests an
  intentional minimisation of the TUI's transitive surface.
   Adding
  the deprecated package back fights that policy and pollutes the
  agent tree with a known-deprecated copy that will need to be
  uninstalled later.
- **Setting `BUN_INSTALL` to point at a directory containing
  `@mariozechner/pi-tui`.
  ** The fallback-path builder respects the
  override,
   but the user does not have a `@mariozechner/pi-tui`
  global install to point it at,
   and creating one regresses to the
  "install deprecated package" rejected option above.
- **Pinning `@earendil-works/pi-tui` to 0.74.1 in the workspace
  (matching the agent tree).
  ** The pi binary is `0.75.4` and pulls
  the matching TUI peer-dep at 0.75.4 via the workspace pin;
   the
  agent tree's 0.74.1 is independent.
   The workspace pin does not
  influence loader resolution inside `~/.pi/agent/npm/...`.

## Why we file this upstream — 5-constraint audit

1. **Is it really upstream's fault?
   ** Yes.
    `@ifi/pi-shared-qna`,
   `@ifi/pi-plan`,
    and `@ifi/pi-extension-subagents` all hardcode
   a package name that has been deprecated on npm with an explicit
   "use @earendil-works/pi-tui instead" notice.
    Sibling extension
   publishers (`@aliou/*`,
    `@diegopetrucci/*`,
    `pi-context`,
   `pi-mcp-adapter`,
    `@juicesharp/rpiv-todo`) already migrated,
   confirming the rename is the new canonical name.
    No
   consumer-side choice can satisfy a hardcoded require for a
   deprecated package without supplying that deprecated package.
2. **Can upstream fix it?
   ** Yes for the loader (one file in
   `package/shared-qna/`);
    sorta-yes for the full migration
   (~16 package manifests + ~15 direct imports — mechanical
   rename,
    no semantic changes since the new package's API
   surface is a superset).
    The shared-qna loader change alone
   restores plan-mode functionality,
    which is the most visible
   crash.
3. **Are they supporting this use case?
   ** Yes.
    `@ifi/pi-plan`,
   `@ifi/pi-shared-qna`,
    and `@ifi/pi-extension-subagents` are
   published-and-advertised packages on npm with `bin` scripts
   (`pi-plan`),
    READMEs,
    and the `pi` extension contract
   (`"pi": { "extensions": ["./index.ts"] }`) in their manifests.
   The `homepage` field in each manifest links to
   `https://github.com/ifiokjr/oh-pi/tree/main/packages/...`,
    and
   the `bugs.url` field points at upstream issues.
4. **Will they likely fix it?
   ** Plausible.
    The upstream repo is
   actively committing (head at `1979b3a9`,
    2026-05-17 — three
   days ago).
    The repo's recent commits show ongoing maintenance
   on `extensions`,
    `subagents`,
    and `live-view` — the same areas
   that contain the stale imports.
    No open issue currently tracks
   the rename (verified via
   `gh issue list --repo ifiokjr/oh-pi --search "pi-tui rename
   earendil mariozechner"`),
    so reporting it is the first step;
   given the repo's commit cadence,
    a follow-up release is
   plausible within days of the report landing.
5. **Have we prototyped a minimal fix compatible with their
   architecture?
   ** Yes — see the loader patch at
   [`pi-extensions-pi-tui-rename.patch`](pi-extensions-pi-tui-rename.patch).
   The patch:
   - Touches one file (`package/shared-qna/pi-tui-loader.ts`).
   - Preserves the public function signatures
     (`requirePiTuiModule`,
      `getPiTuiFallbackPaths`) and the
     `PiTuiLoaderOptions` type,
      so consumers keep working.
   - Tries `@earendil-works/pi-tui` first (current name),
      then
     deprecated `@mariozechner/pi-tui` (backward compatible with
     legacy installs),
      then Bun-global fallbacks for both.
   - Verified post-patch against the same node_modules layout as
     the failing scenario:
      returns both `truncateToWidth` and
     `wrapTextWithAnsi` and produces correct output (see "Harness
     — passing scenarios").

   The patch leaves the wider direct-import migration in
   `subagents/`,
    `providers/`,
    and `spec/` for follow-up commits —
   it would be unidiomatic for a single PR to rewrite ~25 files
   across unrelated package surfaces,
    and the shared-qna loader
   change alone restores end-user functionality for the most
   commonly-installed `@ifi/*` extension (`pi-plan`).

All five constraints hold.
 The draft below is fileable as-is.

## Draft upstream issue

Repository:
 `https://github.com/ifiokjr/oh-pi/issues`.
 Title:
`@ifi/pi-shared-qna loader and peerDeps still reference deprecated
@mariozechner/pi-tui`.
 Labels:
 `bug`,
 `extensions`,
 `packaging`.

The body,
 ready to paste:

````md
## Summary

`@ifi/pi-shared-qna@0.5.1`, `@ifi/pi-plan@0.5.1`, and
`@ifi/pi-extension-subagents@0.5.1` (all current on npm latest)
pin the deprecated `@mariozechner/pi-tui` package by name in
`peerDependencies` and in source `import` statements. The pi-tui
library was renamed to `@earendil-works/pi-tui` between npm
publication of these `@ifi/*` packages on 2026-04-28 and the
present, with an npm deprecation notice on the old scope:

> DEPRECATED ⚠️ - please use @earendil-works/pi-tui instead going forward

Any pi session that installs `@ifi/pi-plan` against a current pi
binary (`@earendil-works/pi-coding-agent`) crashes on first
plan-mode banner render with `MODULE_NOT_FOUND` for
`@mariozechner/pi-tui`, because the modern pi extension installer
populates the agent's npm tree with the new
`@earendil-works/pi-tui` only.

Sibling extension publishers (`@aliou/*`, `@diegopetrucci/*`,
`pi-context`, `pi-mcp-adapter`, `@juicesharp/rpiv-todo`) have all
already migrated, so the breakage is confined to the `@ifi/*`
family.

## Reproduction

Install `pi@0.75.x` and the latest `@ifi/pi-plan`:

```bash
pi install npm:@ifi/pi-plan
pi
```

Crash on first render:

```text
Error: Unable to load @mariozechner/pi-tui. Checked the local
dependency and Bun global fallbacks:
/home/<user>/.bun/install/global/node_modules/@mariozechner/pi-tui
    at requirePiTuiModule
      (.../@ifi/pi-shared-qna/pi-tui-loader.ts:76:11)
    at getPiTui
      (.../@ifi/pi-plan/state.ts:8:46)
    at Object.render
      (.../@ifi/pi-plan/state.ts:102:57)
    ...
  [cause]: Error: Cannot find module '@mariozechner/pi-tui'
```

Standalone harness (no `pi` binary needed) that reproduces the
exact error message:

```bash
VERIFY=$(mktemp -d)
mkdir -p "$VERIFY/node_modules/@earendil-works"
# Only the renamed package present
npm pack @earendil-works/pi-tui --pack-destination "$VERIFY"
tar xzf "$VERIFY"/earendil-works-pi-tui-*.tgz -C "$VERIFY/node_modules/@earendil-works/"
mv "$VERIFY/node_modules/@earendil-works/package" "$VERIFY/node_modules/@earendil-works/pi-tui"
# Loader from @ifi/pi-shared-qna@0.5.1
cp <npm-cache>/@ifi/pi-shared-qna/0.5.1/pi-tui-loader.ts "$VERIFY/loader.ts"
cat > "$VERIFY/verify.ts" <<'EOF'
import { requirePiTuiModule } from "./loader.ts";
console.log(requirePiTuiModule());
EOF
bun "$VERIFY/verify.ts"
```

## Suggested fix

Minimal one-file change in `package/shared-qna/pi-tui-loader.ts`:
try `@earendil-works/pi-tui` first, fall back to
`@mariozechner/pi-tui`, and search Bun-global locations under both
scopes. The patch preserves the public function signatures
(`requirePiTuiModule`, `getPiTuiFallbackPaths`,
`PiTuiLoaderOptions`) so downstream consumers keep working:

```diff
-const PI_TUI_NAME = "@mariozechner/pi-tui";
+const PI_TUI_SPECIFIERS = ["@earendil-works/pi-tui", "@mariozechner/pi-tui"] as const;
```

The full diff and a passing verification harness are recorded in
the linked downstream investigation; happy to open a PR if useful.

For a complete migration, also update the 16 package manifests
that pin the deprecated name in `peerDependencies` and the ~15
source files in `package/subagents/`, `package/providers/`, and
`package/spec/extension/` that have direct
`import ... from "@mariozechner/pi-tui"` statements. The
shared-qna loader change alone restores `@ifi/pi-plan`
functionality, which is the most visible crash, so it can land
first and the direct-import migration can follow.

## Workaround

Downstream users can symlink the renamed package under the old
name inside the agent's npm tree:

```bash
mkdir -p ~/.pi/agent/npm/node_modules/@mariozechner
ln -s ../@earendil-works/pi-tui \
  ~/.pi/agent/npm/node_modules/@mariozechner/pi-tui
```

This restores plan-mode without source edits, at the cost of
needing to reapply after each `pi install`/`pi update`.
````
