# `node:util` `parseArgs` (node v26.13.0) silently ignores an unknown config key, so `includeTokens` yields no tokens and `--` words stay indistinguishable in `positionals`

## Symptom

Two surface patterns fail silently,
with no throw and no warning:

- `parseArgs({ args, allowPositionals: true, includeTokens: true })` returns a result
  with no `tokens` key at all.
- `positionals` cannot tell `search -- a b` from `search a b`;
  both produce the same array,
  so a grammar that treats everything after `--` as data cannot be built
  from `positionals` alone.

Verbatim probe output on node v26.13.0:

```text
with --: ["search","a","b"]
without: ["search","a","b"]
tokens: [{"kind":"positional","index":0,"value":"search"},{"kind":"option-terminator","index":1},{"kind":"positional","index":2,"value":"a"},{"kind":"positional","index":3,"value":"b"}]
```

The third line shows what `tokens: true` exposes;
the first two are what `positionals` returns with and without `--`.

## Root cause

`lib/internal/util/parse_args/parse_args.js` reads exactly three config keys
and validates only their values
([commit-pinned source][src-config]):

```js
  const strict = objectGetOwn(config, 'strict') ?? true;
  const allowPositionals = objectGetOwn(config, 'allowPositionals') ?? !strict;
  const returnTokens = objectGetOwn(config, 'tokens') ?? false;
...
  validateBoolean(strict, 'strict');
  validateBoolean(allowPositionals, 'allowPositionals');
  validateBoolean(returnTokens, 'tokens');
```

The token return flag is named `tokens`.
 `includeTokens` is never read anywhere in that block,
and nothing there rejects unknown keys,
so the wrong spelling is accepted and ignored.
 Where the `includeTokens` spelling came from was not established;
this doc claims only that v26.13.0 does not read it.

The `--` separator is dropped before results are assembled
([commit-pinned source][src-terminator]):

```js
      ArrayPrototypePush(tokens, { kind: 'option-terminator', index });
```

It exists only as a token of kind `option-terminator`,
while positional tokens carry no marker of which side of `--` they came from
([commit-pinned source][src-positional]):

```js
    ArrayPrototypePush(tokens, { kind: 'positional', index, value: arg });
```

Related trap in the same block:
 `allowPositionals` defaults to `!strict`,
so strict parsing also needs `allowPositionals: true` to accept positionals.

[src-config]: https://github.com/nodejs/node/blob/18b32a3b1552e199a01f1a8d9fd64aa756d3b75b/lib/internal/util/parse_args/parse_args.js#L305-L317
[src-terminator]: https://github.com/nodejs/node/blob/18b32a3b1552e199a01f1a8d9fd64aa756d3b75b/lib/internal/util/parse_args/parse_args.js#L204-L206
[src-positional]: https://github.com/nodejs/node/blob/18b32a3b1552e199a01f1a8d9fd64aa756d3b75b/lib/internal/util/parse_args/parse_args.js#L298-L299

The link targets were confirmed against the same commit through the GitHub
contents API (`gh api -H "Accept: application/vnd.github.raw" ...`).
This harness has no `agent-browser`,
so the selection-permalink check ran against the API response rather than a
browser render.

## Verification

Version under test:
node v26.13.0
 (pinned in `toolchain.lock.json`),
 upstream tag `v26.13.0` at commit `18b32a3b1552e199a01f1a8d9fd64aa756d3b75b`.

Runnable harness:

```bash
# Failing catalog: includeTokens is ignored, -- is invisible in positionals
node -e "const {parseArgs}=require('node:util'); const a=parseArgs({args:['search','--','a','b'],allowPositionals:true,includeTokens:true}); const b=parseArgs({args:['search','a','b'],allowPositionals:true,includeTokens:true}); console.log('with --:',JSON.stringify(a.positionals)); console.log('without:',JSON.stringify(b.positionals)); console.log('tokens key:', 'tokens' in a);"

# Working catalog: tokens:true exposes the option-terminator
node -e "const {parseArgs}=require('node:util'); console.log(JSON.stringify(parseArgs({args:['search','--','a','b'],allowPositionals:true,tokens:true}).tokens.filter(t=>t.kind==='option-terminator'||t.kind==='positional')))"
```

Failing catalog:
 `includeTokens: true` produces no `tokens` key,
 and `positionals` is identical with and without `--`.
Working catalog:
 `tokens: true` returns the `option-terminator` token between the two sides,
 and normal option parsing is unchanged.

## Verified workarounds

- Use `tokens: true` and locate the `option-terminator` token.
  Tradeoffs:
   the tokens array must be walked to find the boundary,
   and callers that do not need the boundary pay for the larger result.
- Pre-split `process.argv` at the first bare `--` before calling `parseArgs`,
   then treat the right side as data.
  Tradeoffs:
   the split rule becomes the caller's documented grammar,
   and `parseArgs` never sees `--`;
   this is what `doc/planning/search-fetch-cli.md` adopts for the
  `search-fetch` CLI.

## What does not work

- `includeTokens: true`:
   silently ignored,
  no error,
  no tokens.
- Reading `positionals` to detect `--` words:
   the arrays are identical.
- Relying on strict mode to reject the wrong key:
   strict validates option usage,
  not config keys.

## Upstream filing artifact

`.out-of-scope/` checked 2026-10-31:
 11 exemption files present,
 none covers node core or argument-parsing bug classes,
 so no exemption applies.

Duplicate search 2026-10-31:
 `gh search issues --repo nodejs/node "parseArgs includeTokens"` returned
 nothing,
 with the positive control `gh search issues --repo nodejs/node "parseArgs"`
 returning five issues
 (including #67085 on strict validation and the closed #56184 feature
 request for parseArgs help output).
 No duplicate and no in-flight fix for this behavior was found;
 the work-in-progress search was not run because no filing is planned.

### Upstream filing decision

- Is it upstream's fault?
   Partially.
   Unknown config keys being ignored is this parser's behavior;
   the wrong spelling is the caller's.
   The missing validation is a gap upstream could close.
- Can upstream fix it?
   Yes:
  the config block could reject unknown keys or accept the alias.
- Are they supporting this use case?
   No evidence.
   The documented `tokens: true` path works,
   and no document was found promising typo tolerance for config keys.
- Would the repo welcome our contribution?
   Not checked (their contribution and AI-assistance policy unread);
   recorded as not run.
- Will they likely fix it?
   Not checked beyond the duplicate search;
   recorded as not run.
- Prototype?
   Not reached,
  since constraints 3 through 5 do not hold as checked.

Outcome:
**do not file as-is**,
and no recommendation is made,
 because the policy checks were not run.
 Filing is the poster's decision either way,
 and any filing puts a real person's account in front of strangers;
 trust and blocklist systems exist,
 a maintainer can denounce a poster,
 and that can follow the person across projects.
 The consumer-side workaround above fully solves this repo's problem,
 so nothing here is blocked on upstream.

Draft,
kept for auditability and marked do-not-file:

~~~md
Title: util: parseArgs silently ignores unknown config keys (includeTokens), and positionals cannot distinguish `--` words

Body: On v26.13.0, `parseArgs({ includeTokens: true })` returns no tokens and
throws nothing, because `lib/internal/util/parse_args/parse_args.js` reads
only `strict`, `allowPositionals`, and `tokens` (lines 305-317) and never
validates that config keys are known. Separately, `--` appears only as an
`option-terminator` token (line 204), so `positionals` cannot distinguish
`search -- a b` from `search a b`.

Suggested fix: in the config block (lines 305-317), reject unknown keys with
a TypeError naming the accepted keys, which also catches the `includeTokens`
misspelling.
~~~
