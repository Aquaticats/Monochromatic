# wrangler 4.136.1 `secret put` stores an empty secret when stdin delivers no value

This silently opens LFS uploads.

This note records a 2026-10-01 diagnosis against wrangler 4.136.1
(installed by mise as `npm:wrangler`,
 bundle at
`~/.local/share/mise/installs/npm-wrangler/4.136.1/v11/21c81b-18d7c2394ad08a0a-0/node_modules/.pnpm/wrangler@4.136.1/node_modules/wrangler/wrangler-dist/cli.js`),
cross-checked against upstream `cloudflare/workers-sdk` commit
`7f57b1c60002ae3f077dd9c1e8cc482371065ef4`.
The incident happened while migrating this repository's LFS Worker to a new
`workers.dev` subdomain:
the upload token was rotated,
wrangler reported success,
and the live Worker then accepted an **empty** Basic-auth password,
which left object uploads open to any anonymous caller until the token was
rotated again and verified.

## Symptom

Rotating the LFS upload token through the documented pipe:

```sh
printf '%s' "$(openssl rand -hex 32)" \
  | mise run "//package/config/lfs-r2-worker:secret:write-token"
```

prints the success line:

```text
✨ Success! Uploaded secret LFS_WRITE_TOKEN
```

and the rotated token is then rejected by the Worker.
An upload batch request carrying an empty password succeeds instead:

```text
POST /objects/batch, operation upload, Basic password ""   -> HTTP 200
POST /objects/batch, operation upload, Basic password <rotated token> -> HTTP 401
```

`wrangler secret list` still reports the secret as present,
so nothing in the operator's usual feedback says the stored value is empty.

The consequence is specific to this Worker:
`authorized()` refuses only when the secret is `undefined`,
so an empty secret matches an empty password and every upload succeeds.

## Root cause

### Step 1: `secret put` uploads whatever stdin yielded, including nothing

The installed bundle's `secret put` handler reads the value from stdin when
stdin is not a terminal,
trims trailing whitespace,
and submits it with no emptiness check
(`wrangler-dist/cli.js:299167`):

```js
const isInteractive4 = process.stdin.isTTY;
const secretValue = trimTrailingWhitespace(
  isInteractive4 ? await prompt2("Enter a secret value:", { isSecret: true }) : await readFromStdin()
);
logger2.log(`🌀 Creating the secret for the Worker "${scriptName}"`);
async function submitSecret() {
  const url4 = `/accounts/${accountId}/workers/scripts/${scriptName}/secrets`;
  ...
        body: JSON.stringify({
          name: args.key,
          text: secretValue,
          type: "secret_text"
```

The success line follows the accepted `PUT`
(`wrangler-dist/cli.js:299224`):

```js
logger2.log(`✨ Success! Uploaded secret ${args.key}`);
```

Upstream source at the cross-checked commit is identical
(`packages/wrangler/src/secret/index.ts:134`):

```ts
const isInteractive = process.stdin.isTTY;
const secretValue = trimTrailingWhitespace(
  isInteractive
    ? await prompt("Enter a secret value:", { isSecret: true })
    : await readFromStdin()
);
```

`readFromStdin()` resolves with the concatenation of whatever chunks arrived,
so an empty stream resolves to `""` rather than failing
(`packages/wrangler/src/utils/std.ts:15`):

```ts
export function readFromStdin(): Promise<string> {
  return new Promise((resolve, reject) => {
    const stdin = process.stdin;
    const chunks: string[] = [];
    stdin.on("readable", () => { ... });
    stdin.on("end", () => { resolve(chunks.join("")); });
```

The sibling Secrets Store command in the same package **does** guard this,
which shows the omission is local to `secret put`
(`packages/wrangler/src/secrets-store/commands.ts:425`,
installed bundle `wrangler-dist/cli.js:192853`):

```ts
if (!secretValue) {
  throw new UserError("Need to pass in a value when creating a secret.", {
    telemetryMessage: "secrets store secret create missing value",
  });
}
```

### Step 2: an empty secret is a valid credential for this Worker

`package/config/lfs-r2-worker/src/authorize.ts:131` refuses only the absent
secret:

```ts
const token = env.LFS_WRITE_TOKEN;
if (token === undefined) {
  al.warn('LFS_WRITE_TOKEN is unset; refusing the write',);
  return false;
}
```

and the comparison at `package/config/lfs-r2-worker/src/authorize.ts:70` is a
plain equality:

```ts
return password === token;
```

With `token === ""`,
a request whose Basic password is also `""` satisfies the check.
git-lfs never sends an empty password,
so legitimate pushes fail while an attacker who sends
`Authorization: Basic base64("anything:")` succeeds.

That was the behavior at diagnosis time.
`authorized()` now refuses a blank secret as well as an absent one,
deployed 2026-10-01 as Worker version `b0009f5d-9b00-4719-bbb4-1b6e5902867f`,
so the same operator mistake now fails closed instead of opening uploads.

### Step 3: why stdin was empty is unresolved

Two invocations fed the value through a Node `spawnSync({ input })` pipe,
once via `mise run` and once via `mise exec`,
and both stored an empty secret.
mise's forwarding is not the cause:
in the same session,
the same pipe mechanism delivered its payload intact to every generic consumer
tested (see the "Patterns that work" catalog).
The loss is therefore inside wrangler's own read path,
and two candidate mechanisms were measured and excluded:

- late listener attachment:
  a Node consumer attaching `data` listeners 2 s after start still received the
  payload,
  directly and through `mise exec`;
- paused-mode `readable` plus `read()` (wrangler's exact pattern) attached 2 s
  after start also received the payload.

What remains untested is whether something earlier in wrangler's startup
(configuration load,
 `requireAuth`,
 telemetry) consumes or closes `process.stdin` before the handler reads it.
Settling that needs an instrumented run of the installed binary against a
disposable Worker,
which was not worth the account mutation for this note.

## Verification

Version under test:
wrangler 4.136.1 from the mise install path in the title,
on Node v26.10.0,
against the live Worker `monochromatic-lfs` on the
`aquaticat.workers.dev` subdomain.
Upstream cross-check:
`cloudflare/workers-sdk` at `7f57b1c60002ae3f077dd9c1e8cc482371065ef4`
(the `wrangler-v4.136.1` tag was not fetchable at `--depth 1`,
so the source comparison is against `main`).

### Harness

The live-Worker probe posts an upload batch and never sends a `PUT`,
so it stores nothing:

```sh
node --input-type=module -e '
const batch = "https://monochromatic-lfs.aquaticat.workers.dev/objects/batch";
const body = JSON.stringify({ operation: "upload", transfers: ["basic"], objects: [{ oid: "0".repeat(64), size: 1 }] });
for (const password of process.argv.slice(1)) {
  const response = await fetch(batch, {
    method: "POST",
    headers: {
      Accept: "application/vnd.git-lfs+json",
      "Content-Type": "application/vnd.git-lfs+json",
      Authorization: `Basic ${Buffer.from(`lfs:${password}`).toString("base64")}`,
    },
    body,
  });
  console.log(`${password === "" ? "<empty>" : "<value>"}: ${response.status}`);
}
' "" "definitely-wrong-token"
```

Measured while the stored secret was empty:

```text
<empty>: 200
<value>: 401
```

Measured after a verified rotation:

```text
<rotated token>: 200
<empty>: 401
<value>: 401
```

### Patterns that work

Value supplied by shell redirection from a file,
with no trailing newline,
then verified by probe:

```sh
umask 077
openssl rand -hex 32 | tr -d '\n' > "${HOME}/temp/lfs-write-token.txt"
mise run "//package/config/lfs-r2-worker:secret:write-token" < "${HOME}/temp/lfs-write-token.txt"
```

Result:
the probe returned `200` for the file's value,
`401` for an empty password,
`401` for a wrong token.

Generic stdin consumers receive piped input intact under the same invocation
style,
which is the control that exonerates mise.
Each case was run with `spawnSync({ input: "MARK" })`:

- a mise task running `cat`:
  payload delivered;
- a mise task running `sleep 2; cat`:
  payload delivered;
- `mise exec -- node -e` attaching `data` listeners immediately:
  payload delivered;
- `mise exec -- node -e` attaching `data` listeners after 2 s:
  payload delivered;
- `mise exec -- node -e` attaching paused-mode `readable` plus `read()`
  immediately and after 2 s:
  payload delivered both times.

### Patterns that fail

- `spawnSync("mise", ["run", "//package/config/lfs-r2-worker:secret:write-token"], { input: token })`:
  wrangler reported success,
  stored secret empty (measured twice);
- `spawnSync("mise", ["exec", "--", "wrangler", "secret", "put", "LFS_WRITE_TOKEN"], { input: token })`:
  same outcome;
- probing immediately after `secret put`:
  one rotation returned `401` for the correct token,
  for an empty password,
  and for a wrong token at once,
  which is the signature of a secret that is momentarily absent rather than
  wrong.
  A later rotation with the same invocation succeeded after repeated probes,
  so verification must retry instead of concluding from one response.

## Verified workarounds

- Supply the value by shell redirection from a `0600` file and destroy the file
  afterwards.
  Tradeoff:
  one more step than a pipe,
  and the token touches disk briefly.
- Verify every rotation against the live Worker with three credentials
  (correct,
  empty,
  wrong) and retry on `401`,
  because propagation is not instantaneous.
  Tradeoff:
  a few seconds per rotation;
  the probe must not send a `PUT`,
  or it stores an object.
- Treat an empty `LFS_WRITE_TOKEN` as unusable in the Worker itself,
  refusing writes when the secret is blank rather than only when it is absent.
  This is the consumer-side guard at our boundary and it holds regardless of
  upstream movement.
  Tradeoff:
  it needs a Worker redeploy,
  and it does not repair a secret that is non-empty but wrong.
  Deployed 2026-10-01 as Worker version `b0009f5d-9b00-4719-bbb4-1b6e5902867f`:
  `package/config/lfs-r2-worker/src/authorize.ts` now refuses an absent **or**
  blank secret.
  Two unit cases pin it,
  each sending a Basic password that matches the configured secret exactly,
  so only the guard can make them pass.
  Both were shown to fail without it:
  reverting the guard and rebuilding produced
  `[refuses every write while the secret is blank] [FAIL] AssertionError: expected true to equal false`
  and the same for the whitespace case,
  which is the vulnerability itself,
  measured rather than argued.

## What does not work

- Trusting `✨ Success! Uploaded secret <KEY>`.
  It reports that Cloudflare accepted a value,
  not that the value was the intended one.
- `wrangler secret list`.
  It prints names and types,
  and lists an empty-valued secret exactly like a populated one.
- `git lfs status`,
  `git lfs env`,
  and the repository's `check:markdown-urls` task.
  All three exercise the read path or local state;
  none sends an authenticated upload batch,
  so all three stayed green while uploads were open.
- Blaming mise's stdin forwarding.
  The control catalog shows piped payloads reaching every generic consumer
  tested,
  including wrangler's own paused-mode read pattern with a delayed listener.

## Upstream filing decision

Checked `.out-of-scope/` for a wrangler or Cloudflare exemption:
no entry names wrangler,
Cloudflare,
or `workers-sdk`,
so filing is in scope.

1. **Is it really upstream's fault?**
   Yes for the missing guard.
   `secret put` submits `text: secretValue` without checking emptiness,
   while the sibling Secrets Store command in the same package throws
   `Need to pass in a value when creating a secret.` on the same condition.
   The empty stdin itself is not upstream's fault and its cause is unresolved.
2. **Can upstream fix it?**
   Yes.
   The guard is five lines inside the existing handler and mirrors code already
   in the package.
3. **Are they supporting this use case?**
   Yes.
   Non-interactive `secret put` is an explicit code path
   (`process.stdin.isTTY` branch),
   and `packages/wrangler/src/__tests__/secret.test.ts` has a `non-interactive`
   block with piped-input tests,
   including `should trim stdin secret value, from piped input`.
4. **Would the repo welcome our contribution?**
   Yes,
   with no disclosure requirement found.
   `CONTRIBUTING.md` opens with "Wrangler is an open-source project and we
   welcome contributions from you",
   asks for an issue or discussion before non-trivial changes,
   and contains no AI-assistance or AI-authorship clause
   (`rg -i 'ai-assisted|llm|generated by|disclos' CONTRIBUTING.md` returned
   nothing).
   `.github/ISSUE_TEMPLATE/bug-template.yaml` has no AI policy either.
5. **Will they likely fix it?**
   Plausible.
   No documented won't-fix and no maintainer decline found.
   Comparable piping defects were fixed before:
   issue #784 "Piping into wrangler secret put throws an error",
   #1303 "Can't pipe to `wrangler secret put`",
   #1478 "secret put does not accept value",
   #2730 "secrets put is returning `This command cannot be run in a
   non-interactive context`",
   all closed.
   Tracker searches for `secret put empty stdin`,
   `wrangler secret put blank value`,
   `secret put stdin`,
   and `secret stdin empty` found no existing report of a silently stored empty
   secret.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
   in a disposable clone of `cloudflare/workers-sdk` at
   `7f57b1c60002ae3f077dd9c1e8cc482371065ef4`.
   The fix adds the emptiness guard to `packages/wrangler/src/secret/index.ts`
   and one test to `packages/wrangler/src/__tests__/secret.test.ts`;
   the full diff is [wrangler-secret-put-empty-stdin.patch](wrangler-secret-put-empty-stdin.patch).
   Verification commands and output:

   ```sh
   pnpm install --filter wrangler...
   pnpm exec turbo run build --filter=wrangler^...
   pnpm exec vitest run src/__tests__/secret.test.ts
   ```

   Pre-patch,
   with the new test added and the source unmodified:
   exit status 1,
   `Test Files 1 failed (1)`,
   and the failure output names
   `should refuse an empty piped secret value`.
   Post-patch:
   exit status 0,
   `Test Files 1 passed (1)`.
   The dependency build reported `Tasks: 18 successful, 18 total`.

All six constraints hold,
so the draft below is fileable.
It has not been filed.

### Draft issue

Title:
`🐛 BUG: wrangler secret put stores an empty secret when stdin yields no value`

Body:

````markdown
### What version of Wrangler are you using?

4.136.1

### What operating system are you using?

Linux (Fedora), Node v26.10.0

### Describe the Bug

`wrangler secret put <KEY>` reads the value from stdin in non-interactive mode
and submits it without checking that anything arrived. When stdin yields no
value, the command stores an empty secret and still prints
`✨ Success! Uploaded secret <KEY>`.

An empty secret is not inert. Consumers that compare a presented credential
against the secret treat an empty value as "the credential is the empty
string", so a Worker that gates writes on a secret silently starts accepting
requests with an empty password. That is how we found this: our Git LFS Worker
began returning HTTP 200 for upload batch requests carrying
`Authorization: Basic base64("lfs:")`.

`secret put` for Secrets Store already guards this case:

```ts
// packages/wrangler/src/secrets-store/commands.ts:425
if (!secretValue) {
  throw new UserError("Need to pass in a value when creating a secret.", {
    telemetryMessage: "secrets store secret create missing value",
  });
}
```

The Worker `secret put` handler has no equivalent check:

```ts
// packages/wrangler/src/secret/index.ts:134
const isInteractive = process.stdin.isTTY;
const secretValue = trimTrailingWhitespace(
  isInteractive
    ? await prompt("Enter a secret value:", { isSecret: true })
    : await readFromStdin()
);
```

### Steps to reproduce

1. Deploy any Worker that gates a route on a secret.
2. Run `wrangler secret put KEY` in a context where stdin yields no value.
   We measured it with the value piped through a task runner. We did not test
   `</dev/null`; the source path is the same for any stdin that yields nothing,
   but that specific invocation is inference, not measurement.
3. Observe `✨ Success! Uploaded secret KEY`.
4. Call the gated route with an empty credential. It succeeds.

`wrangler secret list` reports the secret as present, so nothing distinguishes
this state from a correct rotation.

### Expected behavior

`secret put` should refuse an empty value with the same `UserError` the Secrets
Store command already raises, rather than storing an empty secret and reporting
success.

### Proposed fix

Add the guard after `secretValue` is read in
`packages/wrangler/src/secret/index.ts`, plus a test in the existing
`non-interactive` block of `packages/wrangler/src/__tests__/secret.test.ts`.
Verified against `7f57b1c60002ae3f077dd9c1e8cc482371065ef4`: the new test fails
before the guard and the whole file passes after it.
````
