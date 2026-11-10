# pi-safeguard 2.0.1: `/var/home` false-positive blocks everything; pi-budget-model 1.0.1 cannot find judge model on latest-major-version-only default; pi-budget-model 1.0.1 calls `ModelRegistry.getApiKey` that no longer exists on pi-coding-agent 0.70.6

This file groups three independent pi-safeguard / pi-budget-model
bugs that block the guardrail on this workspace.
 Each gets its
own canonical section.

Upstream:
[mgabor3141/yapp](https://github.com/mgabor3141/yapp).

---

## Bug 1: `pathSignals` flags every file under `/var/home/` as a system path

Pi-safeguard version:
 2.0.1.
 Source:
`package/safeguard/src/signals.ts`.
 Date 2026-04-28.

### Symptom

On systems where the home directory is under `/var/home/`
(Fedora/SELinux,
 NixOS,
 others),
 pi-safeguard flags every
file read/write/edit as a security risk,
 even files inside
the project working directory.
 The judge model denies or
prompts for every `read`,
 `write`,
 `edit` tool call,
rendering the guardrail unusable:

```bash
# On a system with /var/home/user as $HOME:
pi -p "Read README.md"
# Result: blocked by the security guardrail
```

### Root cause

`pathSignals` in `package/safeguard/src/signals.ts:95-101`:

```ts
const SYSTEM_PREFIXES = ['/etc', '/usr', '/var', '/boot', '/sys', '/proc',
  '/dev', '/sbin', '/lib',];

function pathSignals(filePath, ctx,) {
  const resolved = resolvePath(filePath, ctx.cwd,);
  if (!isUnder(resolved, ctx.cwd,))
    return true; // outside cwd; flag
  if (isHomeDotfile(resolved, ctx.home,))
    return true; // dotfile in $HOME; flag
  if (isSystemPath(resolved,))
    return true; // system path; flag
  if (SECRET_PATH_PATTERN.test(filePath,))
    return true; // secret file; flag
  return false;
}
```

The `isSystemPath` check fires **after** the `isUnder` check
confirms the path is inside cwd.
 On systems where `$HOME` is
`/var/home/user`,
 every resolved project path starts with
`/var/...`,
 and `isSystemPath` returns `true` for all of
them.

The `isSystemPath` check is redundant once `isUnder` has
returned:
 the `!isUnder` check on the line above already
flags any path outside the project directory,
 including
actual system paths like `/etc/passwd`.
 The `isSystemPath`
check adds no value for paths confirmed under cwd,
 but it
causes a false positive when the project itself lives under
a `SYSTEM_PREFIXES` entry.

Source citations in the yapp monorepo:

- `package/safeguard/src/signals.ts:95-101`:
   `pathSignals`
  function.
- `package/safeguard/src/signals.ts:122-125`:
  `SYSTEM_PREFIXES` array.
- `package/safeguard/src/signals.ts:127-129`:
  `isSystemPath` function.

### Verification

Version under test:
 pi-safeguard 2.0.1.
 Reproduce on any
system with `$HOME` under `/var/home/`.

### Verified workaround: monkey-patch the installed dist

In the compiled dist at
`~/.local/share/mise/installs/node/<node-version>/lib/node_modules/pi-safeguard/dist/index.js`,
delete the line:

```js
if (isSystemPath(resolved,))
  return true;
```

The corrected `pathSignals`:

```js
function pathSignals(filePath, ctx,) {
  const resolved = resolvePath(filePath, ctx.cwd,);
  if (!isUnder(resolved, ctx.cwd,))
    return true;
  if (isHomeDotfile(resolved, ctx.home,))
    return true;
  if (SECRET_PATH_PATTERN.test(filePath,))
    return true;
  return false;
}
```

Tradeoff:
 the patch is lost when `pi update` reinstalls
pi-safeguard.
 Re-apply after updating,
 or install from a
forked version.

### What does not work

- Setting `judgeModel.instructions` to explain that
  `/var/home/user` is a home directory:
   the flagger sends
  the action to the judge regardless,
   and the judge only
  sees the raw action description,
   not why it was flagged.
  Judge instructions cannot override the flagger.
- Adding `/var/home` to allowed paths:
   no such config option
  exists in pi-safeguard.
   The `allowedPaths` concept exists
  in pi-guardrails but not in pi-safeguard.
- Using `strategy: "any-provider"` in `judgeModel`:
   changes
  which model the judge uses,
   not which signals fire.
   The
  flagger is model-agnostic.

### Why we would file this upstream

All 5 constraints hold:

1.  **Is it really upstream's fault?
    ** Yes;
     the redundant
    check is the source.
2.  **Can upstream fix it?
    ** Yes;
     one-line removal in
    `pathSignals`.
3.  **Are they supporting this use case?
    ** Implicit yes;
    pi-safeguard's promise is to block destructive actions,
    not to break on Fedora.
4.  **Will they likely fix it?
    ** Plausible;
     a small,
     targeted
    bug.
5.  **Have we prototyped a minimal fix?
    ** Yes;
     the patched
    dist runs cleanly on this workspace.

Decision:
 worth filing.

### Draft upstream issue (kept as reference; revise before filing)

```md
**Title**: `pathSignals` false positive on systems with home under `/var/home/`

**Labels**: bug

**Description**:

On distributions where the home directory is `/var/home/<user>` (e.g. Fedora with SELinux), the `isSystemPath` check in `pathSignals` flags every project file as a system path because `/var` is in `SYSTEM_PREFIXES`. This makes pi-safeguard unusable on these systems; every `read`, `write`, and `edit` tool call is blocked.

The `isSystemPath` check is redundant for paths already confirmed to be under cwd (the `!isUnder` check on the prior line already flags paths outside the project). Removing it fixes the false positive without weakening protection for actual system paths outside cwd.

**Reproduction**:

1. Set up a system with `HOME=/var/home/user`.
2. `pi install npm:pi-safeguard`.
3. `pi -p "Read README.md"`.
4. Expected: file is read normally (under cwd, no secret keywords).
5. Actual: blocked by the security guardrail.

**Suggested fix**:

Remove `if (isSystemPath(resolved)) return true;` from `pathSignals` in `package/safeguard/src/signals.ts:95-101`. The `!isUnder` check already handles paths outside the project directory, which covers the actual system-path threat model.
```

---

## Bug 2: pi-budget-model 1.0.1 default `majorVersions: 1` excludes cheaper models from older major versions, surfacing as `NoBudgetModelError`

Pi-safeguard version:
 2.0.1.
 Pi-budget-model version:
 1.0.1.
Date 2026-04-28.

### Symptom

When the active model is the latest major version in its
provider and is already relatively cheap,
pi-budget-model's auto-selection fails with
`NoBudgetModelError`.
 Every flagged action then falls back to
user confirmation,
 making the "auto" behaviour equivalent to a
manual permission gate.

### Root cause

`pi-budget-model` defaults to `majorVersions: 1`,
 meaning it
only considers models in the latest major version group.
 If
the active model is in that group and no cheaper model exists
within it,
 the cost-ratio check (`activeCost * costRatio`)
rejects all candidates.

For the synthetic provider,
 the active model
`synthetic/hf:zai-org/GLM-5.1` costs $1/M input.
 With
`costRatio: 0.5`,
 the budget selector looks for models under
$0.50/M input in major version 5.
 Both GLM-5 models cost
$1/M,
 so none qualify.
 Cheaper models (GLM-4.7-Flash at
$0.10/M,
 Nemotron at $0.30/M) are in major versions 4 and 3
respectively,
 and are excluded by the default `majorVersions:
1`.

### Verification

Version under test:
 pi-safeguard 2.0.1 + pi-budget-model
1.0.1.
 Reproduce by configuring pi-safeguard with an
expensive active model and observing `NoBudgetModelError` on
the first flagged action.

### Verified workaround

Set `majorVersions: 0` in the safeguard config to search all
major versions:

```json
// ~/.pi/agent/extensions/pi-safeguard.json
{
  "judgeModel": {
    "majorVersions": 0
  }
}
```

With this setting,
 pi-budget-model finds GLM-4.7-Flash at
$0.10/M input and uses it as the judge.

Tradeoff:
 cross-major-version candidates may use different
prompt formats or token economies;
 the budget selector trusts
the cost numbers but does not validate format compatibility.
For the synthetic provider,
 the cross-major candidates remain
suitable.

### What does not work

- Setting `costRatio: 1`:
   allows the budget selector to pick
  a model that costs the same as the active model,
  defeating the point of using a cheaper model.
- Setting `strategy: "any-provider"`:
   unless other providers
  have API keys configured,
   there are no candidates to find.

### Why we would file this upstream

1.  **Is it really upstream's fault?
    ** Borderline.
     The
    default is documented;
     users can override.
     But
    "latest-only" is an unfriendly default when the active
    model is already latest.
2.  **Can upstream fix it?
    ** Yes;
     default could be 0
    (consider all majors),
     or 2 (latest plus one back),
     or
    bias toward "latest with cheaper fallback".
3.  **Are they supporting this use case?
    ** Yes;
     the option
    exists.
4.  **Will they likely fix it?
    ** Possibly;
     depends on default
    philosophy.
5.  **Have we prototyped a minimal fix?
    ** No;
     just the
    user-side override.

Decision:
 worth raising as a defaults-discussion issue.

---

## Bug 3: pi-budget-model 1.0.1 calls `ModelRegistry.getApiKey(model)` which was removed in pi-coding-agent 0.70.6, crashing silently into "No judge model available"

Pi-safeguard version:
 2.0.1.
 Pi-budget-model version:
 1.0.1.
Pi-coding-agent version:
 0.70.6.
 Source:
`package/budget-model/src/index.ts`.
 Date 2026-04-28.

### Symptom

Every flagged action shows "No judge model available;
manual approval required" regardless of judge-model
configuration.
 Setting `majorVersions: 0` or
`strategy: "any-provider"` has no effect.
 The judge is never
reached.

```bash
# Any action that triggers the safeguard flagger (e.g. a bash command
# containing "sudo"):
pi -p "run sudo apt update"
# Result: "No judge model available; manual approval required."
```

### Root cause

`pi-budget-model` 1.0.1 calls
`ctx.modelRegistry.getApiKey(model)` at four locations in its
compiled dist:

```js
// pi-budget-model/dist/index.js, lines 81, 113, 128, 156
const apiKey = await ctx.modelRegistry.getApiKey(candidate,);
```

The `ModelRegistry` class in
`@earendil-works/pi-coding-agent` 0.70.6 does **not** have a
`getApiKey(model)` method.
 It was replaced by
`getApiKeyAndHeaders(model)`:

```ts
// ModelRegistry methods in pi-coding-agent 0.70.6
getApiKeyAndHeaders(model: Model<Api>): Promise<ResolvedRequestAuth>;
getApiKeyForProvider(provider: string): Promise<string | undefined>;
```

`ResolvedRequestAuth` is a discriminated union:

```ts
{ ok: true; apiKey?: string; headers?: Record<string, string> }
| { ok: false; error: string }
```

When `findBudgetModel` reaches the first `getApiKey` call,
 it
throws:

```text
TypeError: ctx.modelRegistry.getApiKey is not a function
```

Pi-safeguard's `evaluate` function catches **all** errors
with a bare `catch` block:

```js
// pi-safeguard/dist/index.js
async function evaluate(pi, ctx, config, systemPrompt, action, batchContext,
  flowVerdicts,)
{
  let judge;
  try {
    judge = await resolveJudgeModel(ctx, config,);
  }
  catch {
    return askUser(pi, ctx, action,
      'No judge model available; manual approval required.',);
  }
  // ...
}
```

The TypeError is silently swallowed.
 The user sees the
generic "No judge model available" message with no
indication that the real problem is a missing method,
 not a
missing model.

Source locations:

- `package/budget-model/src/index.ts`:
   four calls to
  `ctx.modelRegistry.getApiKey(model)`.
- `@earendil-works/pi-coding-agent`
  `core/model-registry.ts`:
   `ModelRegistry` class has
  `getApiKeyAndHeaders` and `getApiKeyForProvider` but no
  `getApiKey`.
- `package/safeguard/src/index.ts`:
   bare `catch` block in
  `evaluate` that swallows the TypeError.

### Verification

Version under test:
 pi-safeguard 2.0.1 + pi-budget-model
1.0.1 + pi-coding-agent 0.70.6.
 Reproduce any flagged action;
the "No judge model available" message surfaces every time.

### Verified workarounds

#### Option A: monkey-patch `ModelRegistry.prototype`

Add to the pi-safeguard dist so it runs before
`findBudgetModel` is called:

```js
import { ModelRegistry, } from '@earendil-works/pi-coding-agent';
if (!ModelRegistry.prototype.getApiKey) {
  ModelRegistry.prototype.getApiKey = async function(model,) {
    const result = await this.getApiKeyAndHeaders(model,);
    if (!result.ok)
      return undefined;
    return result.apiKey;
  };
}
```

Tradeoff:
 monkey-patching prototypes is fragile across
upgrades.
 The shim is small and limited;
 safer than nothing.

#### Option B: edit the pi-budget-model dist to call `getApiKeyAndHeaders`

Replace all four occurrences of:

```js
const apiKey = await ctx.modelRegistry.getApiKey(candidate,);
```

with:

```js
const apiKey = (await ctx.modelRegistry.getApiKeyAndHeaders(candidate,)).apiKey;
```

(and similarly for the call sites that use `model` instead
of `candidate`).

Tradeoff:
 patches the dist directly;
 lost on update.
 Cleaner
than the prototype shim but loses portability.

Caveat for both options:
 lost when `pi update` reinstalls
pi-safeguard.
 Re-apply after updating,
 or install from a
forked version.

### What does not work

- Setting `majorVersions: 0`:
   the code crashes before
  reaching the version filtering logic.
   The method does not
  exist regardless of config.
- Setting `strategy: "any-provider"`:
   same crash,
   different
  code path (the `getApiKey` calls in `findAnyProvider` and
  `resolveModelOverride` are also broken).
- Setting `modelOverride` to pin a specific judge model:
  calls `getApiKey` in `resolveModelOverride`,
   which also
  crashes.
- Configuring `judgeModel.instructions`:
   the judge is never
  called;
   the crash happens during model resolution.

### Why the `majorVersions: 0` workaround from Bug 2 does not help

The `majorVersions: 0` workaround targets a different
(earlier) bug where the budget selector excluded older major
versions.
 That fix is correct for its specific scenario,
 but
it cannot help here because the `getApiKey` TypeError
crashes the function **before** the version filtering logic
runs.
 The `majorVersions` setting is never evaluated.

### Why we would file this upstream

All 5 constraints hold:

1.  **Is it really upstream's fault?
    ** Yes;
     pi-budget-model
    1.0.1 was published against an older pi-coding-agent
    ModelRegistry shape and was not updated when the method
    was renamed.
2.  **Can upstream fix it?
    ** Yes;
     replace four call sites
    with `getApiKeyAndHeaders`,
     or add a `getApiKey`
    compatibility method to `ModelRegistry`.
3.  **Are they supporting this use case?
    ** Yes;
     pi-budget-model
    is a documented pi-safeguard companion.
4.  **Will they likely fix it?
    ** Plausible;
     the fix is small
    and the failure mode is total (every flagged action
    broken).
5.  **Have we prototyped a minimal fix?
    ** Yes;
     both Option A
    and Option B run cleanly on this workspace.

Decision:
 worth filing.

### Draft upstream issue (kept as reference; revise before filing)

```md
**Title**: `pi-budget-model` calls `ModelRegistry.getApiKey()` which no longer exists in pi-coding-agent 0.70.6

**Labels**: bug

**Description**:

`pi-budget-model` 1.0.1 calls `ctx.modelRegistry.getApiKey(model)` at four locations, but `ModelRegistry` in `@earendil-works/pi-coding-agent` 0.70.6 only has `getApiKeyAndHeaders(model)` and `getApiKeyForProvider(provider)`. The `getApiKey(model)` method was removed (or renamed) without a compatibility shim.

This causes a `TypeError: ctx.modelRegistry.getApiKey is not a function` that is silently caught by pi-safeguard's bare `catch` block in `evaluate`, surfacing as the generic "No judge model available" message with no diagnostic detail.

Every flagged action requires manual approval, regardless of configuration. The `majorVersions`, `costRatio`, `strategy`, and `modelOverride` settings are all unreachable.

**Reproduction**:

1. Install pi-safeguard 2.0.1 (which depends on pi-budget-model 1.0.1).
2. Use any model that triggers the safeguard flagger.
3. Expected: judge model is selected and evaluates the action.
4. Actual: "No judge model available; manual approval required."

**Suggested fix**:

Replace `ctx.modelRegistry.getApiKey(model)` calls with `ctx.modelRegistry.getApiKeyAndHeaders(model)` and extract the API key from the structured result. Alternatively, add a `getApiKey(model)` compatibility method to `ModelRegistry` that wraps `getApiKeyAndHeaders`.
```
