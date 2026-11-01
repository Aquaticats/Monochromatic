# morph-compact 0.2.8: SessionStart hook re-runs `npm install` on every Claude Code startup, blocking startup for ~4 minutes on OSTree-symlink systems

The [`morph-compact`][morph-compact] plugin's `SessionStart` hook
unconditionally runs `npm install` on a 73-MB `package-lock.json`
before invoking its actual handler.
 Even with a warm cache,
 this
takes ~3 m 55 s on this machine.
 The hook also fires on every
`SessionStart` event in Claude Code,
 so the cost is paid on every
startup or resume.

A second,
 independent bug compounds the cost:
 on Fedora Silverblue and
similar OSTree-based distros,
 `/home` is a relative symlink to
`var/home`,
 and `npm install --prefix /home/user/...` writes
`node_modules/<pkg>` paths into `package-lock.json` with one extra
`../var/` segment per run.
 The lockfile grows linearly across startups
(observed:
 1237 -> 2642 -> 4039 lines after three runs).
 The user's
copy reached 663 907 lines,
 71 MB,
 48 737 package entries,
 which is
what makes `npm install` walk so much.
 The marketplace clone ships a
clean 43-KB lockfile.

We do not file the npm half upstream (see "Why we do not file the npm
half upstream").
 The morph-compact half is filed and the consumer-side
workaround is to disable the plugin until upstream lands the fix.

---

## Symptom

A Claude Code session running with `morph-compact@morph` enabled on a
distro where `/home` is a symlink to `var/home` (Fedora Silverblue,
Bluefin,
 Bazzite,
 Aurora,
 uBlue family,
 openSUSE MicroOS,
 NixOS with
`home-manager.users.<u>.home.symlink`,
 anything OSTree-based) spends
~3-5 minutes between `cli_after_main_complete` and
`before_processUserInput` on every startup.

Built-in profiler output (enable with `CLAUDE_CODE_PROFILE_STARTUP=1`,
report at `~/.claude/startup-perf/<session>.txt`):

```text
[+ 301.098ms] cli_after_main_complete
[+240465.606ms] before_processUserInput   <-- 240-second gap
[+240501.412ms] after_processUserInput
[+245721.005ms] headless_turn_start
Total startup time: 245721.005ms
```

The 240-second gap has no named checkpoint because Claude Code does
not instrument hook execution time in the startup-perf report.
 The
gap is the morph-compact `SessionStart` hook awaiting `npm install`.

## Root cause

Two interacting bugs.
 The first is the trigger;
 the second is the
amplifier.
 Either alone would be slow but tolerable.
 Together they
multiply.

### Bug 1: morph-compact runs `npm install` on every `SessionStart`, even though `session-start.js` needs no dependencies

`morph-compact` ships `hooks/hooks.json` wiring four events.
 Two of
them (`PreCompact`,
 `SessionStart`) prefix their handler invocation
with `npm install`:

`hooks/hooks.json:3-9` (PreCompact entry,
 upstream HEAD `06f96f0`):

```json
"PreCompact": [{ "hooks": [{
  "type": "command",
  "command": "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/pre-compact.js"
}] }],
```

`hooks/hooks.json:11-17` (SessionStart entry):

```json
"SessionStart": [{ "hooks": [{
  "type": "command",
  "command": "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/session-start.js"
}] }],
```

The `session-start.js` handler imports only Node.
js built-ins:

`hooks/session-start.js:1-3` (upstream HEAD `06f96f0`):

```js
import {
  readFile,
  unlink,
} from 'node:fs/promises';
import { text, } from 'node:stream/consumers';
import {
  emitContext,
  fileExists,
  log,
  stateFile,
} from './lib/state.js';
```

`hooks/lib/state.js:1-3`:

```js
import { access, } from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
```

Neither file imports from `node_modules`.
 `@morphllm/morphsdk` (the
only declared dependency in `package.json`) is imported solely by
`hooks/lib/morph.js`,
 which is imported solely by `hooks/pre-compact.js`.
So the `npm install` step in front of `session-start.js` does no
useful work;
 the handler runs identically without `node_modules`.

Verification:

```text
$ cd $TMPDIR/morph-upstream && ls node_modules
ls: cannot access 'node_modules': No such file or directory
$ echo '{"session_id":"x","hook_event_name":"SessionStart","source":"startup"}' \
  | node hooks/session-start.js
[morph-compact] SessionStart: source=startup session=x
[morph-compact] SessionStart: no state file, nothing to inject
Exit: 0
```

`pre-compact.js` does need `@morphllm/morphsdk`,
 so the
`PreCompact` install step is not pure cruft,
 only mis-placed.
 A
one-time install during plugin install would suffice;
 an unconditional
`npm install` on every fire of two distinct hook events is the bug.

### Bug 2: `npm install --prefix /home/user/...` corrupts `package-lock.json` when `/home` is a relative symlink to `var/home`

On OSTree-based distros,
 `/home` is a relative symlink:

```text
$ ls -la /home
lrwxrwxrwx. 1 root root 8 Dec 31  1969 /home -> var/home
```

When `npm install --prefix /home/user/.claude/plugins/cache/morph/morph-compact/0.2.8`
runs,
 npm writes `node_modules/<pkg>` paths into `package-lock.json`
relative to its own resolved working directory.
 The relative-symlink
target (`var/home`) is interpreted relative to the parent of the
symlink (`/`),
 which yields `/var/home`.
 npm produces a path of the
form `../../../var/home/user/...` on the first run,
 and then on each
subsequent run prepends one more `var/` segment because the previous
run's already-written paths get re-resolved against the new run's
resolved-prefix interpretation.
 The cumulative effect is observable as
a linear lockfile growth across runs.

Reproduction (independent of morph-compact):

```text
$ TEST=/var/home/user/tmp-npm-test
$ rm -rf "$TEST" && mkdir -p "$TEST"
$ cp morph-claude-code-plugin/package.json $TEST/
$ cp morph-claude-code-plugin/package-lock.json $TEST/
$ wc -l $TEST/package-lock.json    # 1237 lines, 43K

$ npm install --silent --prefix /home/user/tmp-npm-test \
      --omit=dev --no-audit --no-fund  # via /home symlink
$ wc -l $TEST/package-lock.json    # 1237 lines (no change first run)

$ npm install --silent --prefix /home/user/tmp-npm-test \
      --omit=dev --no-audit --no-fund
$ wc -l $TEST/package-lock.json    # 2642 lines, 99K

$ npm install --silent --prefix /home/user/tmp-npm-test \
      --omit=dev --no-audit --no-fund
$ wc -l $TEST/package-lock.json    # 4039 lines, 151K
```

Sample of newly-added keys after run 3:

```text
../../../var/var/var/home/user/tmp-npm-test/node_modules/wrappy
../../../var/var/var/home/user/tmp-npm-test/node_modules/yauzl
../../../var/var/var/home/user/tmp-npm-test/node_modules/zod
```

The same test with `--prefix /tmp/...` (no symlink in the path) leaves
the lockfile untouched at 1237 lines after two runs and `npm install`
returns in under 200 ms warm.

The user's cache reached 71 MB / 663 907 lines / 48 737 packages over
many sessions.
 Walking that many lockfile entries is what makes
`npm install` take ~4 minutes on warm cache;
 the install of the
~95 actual packages is fast.

### Why the two bugs compound

Bug 1 ensures `npm install --prefix /home/user/...` runs on every
`SessionStart`.
 Bug 2 ensures each such run grows the lockfile by
~1400 lines and adds ~5-7 seconds to the next run's lockfile-walking
time.
 The cumulative effect is the user's observed 5-minute startup.

If only Bug 1 existed (clean lockfile,
 no symlink),
 each `SessionStart`
would cost ~0.6-2 s (still wrong,
 but tolerable).
 If only Bug 2 existed
(plugin only installed once at install time),
 the lockfile would never
bloat because no subsequent `npm install` would fire.
 The cluster is
worth treating as one document because the trigger for the npm bug is
the morph hook design.

## Verification

Versions under test:

- `morph-compact` 0.2.8 (`gitCommitSha: 06f96f06e31f51838305b7498fe976f6909635c6`)
- npm 10.
  x (the npm shipped with Node.
  js 22)
- Fedora Silverblue / Bluefin 41+ (Linux 6.19.14-ogc5.1.
  fc44.
  x86_64)
- Claude Code 2.1.143

Harness 1 - measure full startup with profiler:

```text
$ rm -rf ~/.claude/startup-perf
$ CLAUDE_CODE_PROFILE_STARTUP=1 claude -p "." > /dev/null 2>&1
$ cat ~/.claude/startup-perf/*.txt | tail -10
[+ 301.098ms] cli_after_main_complete
[+240465.606ms] before_processUserInput
[+240501.412ms] after_processUserInput
[+245721.005ms] headless_turn_start
Total startup time: 245721.005ms
```

Harness 2 - measure the npm install in isolation:

```text
$ { time npm install --silent \
      --prefix ~/.claude/plugins/cache/morph/morph-compact/0.2.8 \
      --omit=dev --no-audit --no-fund ; }
real    3m51.115s    # run 1 (warm cache, no actual deps to fetch)
real    3m55.413s    # run 2 (identical lockfile)
```

Harness 3 - reproduce the lockfile bloat with a clean copy:

```text
$ TEST=/var/home/user/tmp-npm-test
$ rm -rf "$TEST" && mkdir -p "$TEST"
$ cp /tmp/morph-upstream/{package.json,package-lock.json} "$TEST/"
$ for i in 1 2 3; do
    npm install --silent --prefix /home/user/tmp-npm-test \
        --omit=dev --no-audit --no-fund >/dev/null 2>&1
    echo "run $i: $(wc -l < $TEST/package-lock.json) lines"
  done
run 1: 1237 lines
run 2: 2642 lines
run 3: 4039 lines
```

Harness 4 - `session-start.js` runs without `node_modules`:

```text
$ cd /tmp/morph-upstream && ls node_modules || echo absent
absent
$ echo '{"session_id":"x","hook_event_name":"SessionStart","source":"startup"}' \
    | node hooks/session-start.js
[morph-compact] SessionStart: source=startup session=x
[morph-compact] SessionStart: no state file, nothing to inject
$ echo $?
0
```

### Patterns that compile (i.e., work as intended)

- `morph-compact` disabled (`enabledPlugins["morph-compact@morph"] = false`):
  startup completes in seconds,
   npm install never fires.
- `morph-compact` enabled on a system where `/home` is a real directory
  (most distros):
   each `SessionStart` runs `npm install`,
   but the
  lockfile stays small (a few KB) and the install completes in ~1 s
  warm.
   Still wrong,
   still wasteful,
   but tolerable.
- `morph-compact` enabled on Silverblue with `--prefix /var/home/user/...`
  passed directly (bypassing the `/home` symlink):
   no bloat.
   Not
  achievable through configuration;
   would require patching
  `${CLAUDE_PLUGIN_ROOT}` upstream.

### Patterns that fail

- `morph-compact` enabled on any distro with `/home -> var/home`
  symlink,
   default Claude Code plugin install path:
   lockfile grows
  unbounded,
   startup time grows linearly with session count.

## Verified workarounds

### Disable `morph-compact@morph`

In `~/.claude/settings.json`:

```jsonc
"enabledPlugins": {
  "morph-compact@morph": false,
}
```

Effect:
 hook never fires;
 `npm install` never runs.
 Startup returns to
sub-second.

Tradeoff:
 lose Morph-backed `/compact`.
 If `autoCompactEnabled` is
already `false`,
 the loss is the `/compact` slash command only (manual
compaction).
 No data loss;
 the plugin can be re-enabled later.

### Delete the corrupted cache before re-enabling

If you want to re-enable the plugin after upstream lands a fix:

```bash
rm -rf ~/.claude/plugins/cache/morph/morph-compact/0.2.8
# Claude Code re-clones from the marketplace on next start.
# Marketplace ships the clean 43 KB lockfile.
```

Tradeoff:
 re-enables the bug;
 only useful in combination with the
upstream-fix workaround below.

### Local hook override (transitional, brittle)

If you must keep the plugin enabled before upstream lands the fix,
edit the on-disk `hooks.json` and apply the patch in
"Draft upstream issue" below directly:

```bash
$EDITOR ~/.claude/plugins/cache/morph/morph-compact/0.2.8/hooks/hooks.json
```

Tradeoff:
 the next Claude Code marketplace auto-update will overwrite
your edit (the marketplace is git-cloned,
 plugin cache is re-derived
from it).
 You'd need to re-apply on every update.
 Not recommended;
disable the plugin instead.

## What does not work

- **Setting `--ignore-scripts` or other npm flags**:
   the cost is in
  walking the lockfile,
   not in running install scripts.
   Lockfile walk
  happens before script invocation.
- **`npm ci` instead of `npm install`**:
   would be faster for a small
  lockfile,
   but with the bloated 71 MB lockfile it still has to walk
  all 48 737 entries;
   not measurably faster,
   and would fail outright if
  the lockfile drifted.
- **Deleting `node_modules` only**:
   lockfile stays bloated,
   next install
  is just as slow.
- **Pinning npm version**:
   the bug is in npm's relative-symlink
  resolution,
   not in a specific version.
   Reproduced on npm 10.
  x.
- **`CLAUDE_CODE_DISABLE_*` env vars**:
   there is no env var to skip
  SessionStart hooks selectively.
- **Reordering plugins in `enabledPlugins`**:
   hooks fire in parallel;
  reordering doesn't help.

## Draft upstream issue (DO NOT FILE without revalidating the five constraints)

### Why we file this upstream

This repo's policy is to report an issue upstream only when ALL of the
following hold:
 we are absolutely sure it is the upstream's fault,
they can fix it,
 they are supporting the use case,
 they are likely to
fix it,
 and we have already prototyped a minimal fix compatible with
their architecture.

Walking the five constraints against the morph-compact hook design:

1. **Is it really upstream's fault?
   ** Yes.
    The hook unconditionally
   runs `npm install` on every `SessionStart` and `PreCompact` event.
   `session-start.js` imports only Node built-ins and `./lib/state.js`
   (which also only uses built-ins).
    The install on `SessionStart`
   does no useful work.
    The install on `PreCompact` is needed only
   once per plugin install,
    not per event.

2. **Can upstream fix it?
   ** Yes;
    a 2-line edit to `hooks/hooks.json`.
   See the patch in "Suggested fix" below.
    No code-level changes
   required.

3. **Are they supporting this use case?
   ** Yes.
    The plugin's stated
   value proposition is "Intercepts Claude Code's context compaction
   and replaces it with Morph's compaction service"
   (`.claude-plugin/plugin.json` description).
    Claude Code SessionStart
   hooks are explicitly an extension point Claude Code documents.
    The
   plugin is actively maintained:
    latest commit `06f96f0`
   ("chore:
    bump version to 0.2.8") landed this month;
    PRs merged
   include refactors from TypeScript to plain JS (`a62ce8d`) and from
   bun to node (`f4e9daf`),
    both during 2026-Q2.

4. **Will they likely fix it?
   ** Plausibly.
    Project velocity is healthy
   (eight version bumps from 0.2.0 to 0.2.8 in two months,
    multiple PRs
   merged) and the fix is a 2-line gate.
    Risk:
    maintainers may push
   back on the gate semantics ("what if the lockfile changes mid-
   session?
   "),
    but the cache-per-version directory layout means a
   version bump already invalidates the gate via fresh
   `${CLAUDE_PLUGIN_ROOT}`.

5. **Have we prototyped a minimal fix?
   ** Yes.
    Cloned upstream HEAD
   `06f96f0` into a fresh `mktemp -d`,
    applied the patch below,
   verified `session-start.js` runs without `node_modules` (cold:
   38 ms),
    verified `PreCompact` still installs on first run with no
   `node_modules` (1.36 s) and skips the install on subsequent runs
   (0.16 s).
    The full diff is reproduced below.

All five constraints hold.
 The draft below is fileable;
 revalidate
constraints 3-4 if the project has changed when you actually file
(check that the repo is still maintained and that no equivalent fix
has already merged).

### Draft (revalidate before filing)

Title:
 `npm install` re-runs on every `SessionStart` and `PreCompact`
event,
 blocking startup for minutes on OSTree-based distros

Labels:
 `bug`,
 `performance`,
 `hooks`

````md
## Description

`hooks/hooks.json` wires `SessionStart` and `PreCompact` events to
shell commands of the form:

```sh
npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 \
  && node ${CLAUDE_PLUGIN_ROOT}/hooks/<handler>.js
```

This runs `npm install` on every fire of those events, blocking the
handler. Two issues:

1. **`session-start.js` does not need `node_modules`.** It imports
   only `node:stream/consumers`, `node:fs/promises`, and
   `./lib/state.js`. `state.js` imports only `node:path`, `node:os`,
   `node:fs/promises`. `@morphllm/morphsdk` is used solely by
   `hooks/lib/morph.js`, which is imported solely by `pre-compact.js`.
   The install in front of `session-start.js` is pure cost with no
   benefit.

2. **`pre-compact.js` does need `morphsdk`, but only once.** Plugin
   cache lives at `${CLAUDE_PLUGIN_ROOT}` which is keyed by plugin
   version. A version bump creates a fresh directory with no
   `node_modules`; subsequent fires within the same version do not
   need to re-install.

The cost of an unnecessary `npm install` is non-trivial because npm
walks the full `package-lock.json` on every invocation even when no
work is needed. On OSTree-based distros (Fedora Silverblue, Bluefin,
Bazzite, Aurora, uBlue, openSUSE MicroOS), `/home` is a symlink to
`var/home`, and `npm install --prefix /home/user/...` corrupts the
lockfile by appending an extra `../var/` segment to every
`node_modules/<pkg>` path on each run. Lockfile grows linearly across
sessions; on the affected user's machine it reached 663 907 lines /
71 MB / 48 737 packages after many sessions. `npm install` on that
lockfile takes ~4 minutes warm. Startup blocks for the full duration.

The lockfile corruption is npm's bug (reproducible with any npm
project on the same FS layout), but the trigger here is morph-compact
running `npm install` on every event.

## Reproduction

System: Fedora Silverblue or Bluefin (any distro where `/home` is a
symlink to `var/home`).

```sh
TEST=/var/home/user/tmp-morph-test
rm -rf "$TEST" && mkdir -p "$TEST"
git clone https://github.com/morphllm/morph-claude-code-plugin /tmp/morph
cp /tmp/morph/package.json /tmp/morph/package-lock.json "$TEST/"

for i in 1 2 3; do
  npm install --silent --prefix /home/user/tmp-morph-test \
      --omit=dev --no-audit --no-fund >/dev/null 2>&1
  echo "run $i: $(wc -l < $TEST/package-lock.json) lines"
done
# run 1: 1237 lines
# run 2: 2642 lines
# run 3: 4039 lines
```

After a few weeks of Claude Code sessions:

```sh
$ wc -l ~/.claude/plugins/cache/morph/morph-compact/0.2.8/package-lock.json
663907
$ time npm install --silent --prefix ~/.claude/plugins/cache/morph/morph-compact/0.2.8 \
      --omit=dev --no-audit --no-fund
real    3m51.115s
```

Claude Code startup profiler (`CLAUDE_CODE_PROFILE_STARTUP=1`) shows
a 240-second gap between `cli_after_main_complete` and
`before_processUserInput`, matching the npm install duration.

## Suggested fix

Two changes to `hooks/hooks.json`:

1. Remove `npm install` from `SessionStart` entirely.
   `session-start.js` does not import anything from `node_modules`.
2. Gate the `PreCompact` install on `node_modules` directory
   existence. Version-keyed cache means a version bump invalidates
   the gate naturally.

```diff
--- a/hooks/hooks.json
+++ b/hooks/hooks.json
@@ -5,7 +5,7 @@
         "hooks": [
           {
             "type": "command",
-            "command":
  "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/pre-compact.js"
+            "command":
  "([ -d \"${CLAUDE_PLUGIN_ROOT}/node_modules\" ] || npm install --silent --prefix \"${CLAUDE_PLUGIN_ROOT}\" --omit=dev --no-audit --no-fund >/dev/null 2>&1) && node \"${CLAUDE_PLUGIN_ROOT}/hooks/pre-compact.js\""
           }
         ]
       }
@@ -15,7 +15,7 @@
         "hooks": [
           {
             "type": "command",
-            "command":
  "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/session-start.js"
+            "command": "node \"${CLAUDE_PLUGIN_ROOT}/hooks/session-start.js\""
           }
         ]
       }
```

Verified against upstream HEAD `06f96f0`:

- SessionStart cold (no `node_modules`): 38 ms (handler completes
  cleanly; no install attempted).
- PreCompact cold (no `node_modules`): 1.36 s (npm install runs once,
  populates 95 packages, handler runs).
- PreCompact warm (`node_modules` present): 0.16 s (install skipped,
  handler runs immediately).

The quoted-path additions (`"${CLAUDE_PLUGIN_ROOT}"`) are a defensive
fix for plugin-root paths containing spaces; safe to drop if you
prefer to keep the diff smaller.

## Why this matters

Even users on non-symlink distros pay an unnecessary ~1 s per
SessionStart for the no-op install. On OSTree distros the cost
compounds catastrophically.

## Workaround for users on affected distros

Disable the plugin in `~/.claude/settings.json` `enabledPlugins` until
this lands.
````

The full patch as applied during prototyping:

```diff
diff --git a/hooks/hooks.json b/hooks/hooks.json
index d2fcc24..352e61b 100644
--- a/hooks/hooks.json
+++ b/hooks/hooks.json
@@ -5,7 +5,7 @@
         "hooks": [
           {
             "type": "command",
-            "command": "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/pre-compact.js"
+            "command": "([ -d \"${CLAUDE_PLUGIN_ROOT}/node_modules\" ] || npm install --silent --prefix \"${CLAUDE_PLUGIN_ROOT}\" --omit=dev --no-audit --no-fund >/dev/null 2>&1) && node \"${CLAUDE_PLUGIN_ROOT}/hooks/pre-compact.js\""
           }
         ]
       }
@@ -15,7 +15,7 @@
         "hooks": [
           {
             "type": "command",
-            "command": "npm install --silent --prefix ${CLAUDE_PLUGIN_ROOT} --omit=dev --no-audit --no-fund >/dev/null 2>&1 && node ${CLAUDE_PLUGIN_ROOT}/hooks/session-start.js"
+            "command": "node \"${CLAUDE_PLUGIN_ROOT}/hooks/session-start.js\""
           }
         ]
       }
```

## Why we do not file the npm half upstream

1. **Is it really upstream's fault?
   ** Yes,
    npm's relative-symlink
   resolution under `--prefix` is the proximate cause.
2. **Can upstream fix it?
   ** Probably.
    The fix is to call `realpath` on
   the prefix before computing relative `node_modules/<pkg>` paths.
3. **Are they supporting this use case?
   ** No documented signal that
   `--prefix` is expected to work cleanly through relative symlinks.
   OSTree's relative `/home -> var/home` is unusual;
    most distros use
   a real directory.
4. **Will they likely fix it?
   ** Unknown.
    npm has a large surface and a
   complex backlog.
    Not investigated here.
5. **Have we prototyped a minimal fix?
   ** No. Patching npm's
   path-resolution involves the package-arborist core;
    out of scope
   for a Claude Code troubleshooting doc.

We fail constraints 3,
 4,
 and 5.
 Decision:
 do not file.
 The
consumer-side fix (disable morph-compact,
 or land the morph upstream
patch) eliminates the trigger;
 the npm bug becomes latent until
something else passes a `--prefix` through `/home`.

[morph-compact]: https://github.com/morphllm/morph-claude-code-plugin
