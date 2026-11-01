# dsh 0.2.1-alpha.4 coerces `cordis_inspect_query` `json` input to a string, failing every filtered inspect query with `"input" must be an object`

## Symptom

Every `cordis_inspect_query` call that passes an `input` object fails
with one message variant:

`Host Cordis inspect Config.listConfigs rejected input: "input" must be an object`

The same shape appears for any provider and method
whose input schema is a non-empty object:
`Config.listConfigs` `name`/`offset`/`limit` filters,
`Service.listService` `service` selection,
`Event.listEvents` `event` selection.
Calls that omit `input` succeed,
so unfiltered discovery works
while every filtered or paged query is unreachable.

Surface trigger:
the model-facing tool call for `cordis_inspect_query`
with an `input` value in any encoding.

## Root cause

The tool declares `input` as a `json` typed parameter.
`@deepseek-ai/dsh-tool-cordis/lib/types/index.js:44`:

```js
input: { type: 'json', description: 'Optional query input; it must satisfy the method input schema.' },
```

The handler forwards the parsed value untouched.
`@deepseek-ai/dsh-tool-cordis/lib/types/index.js:51`:

```js
const data = await ctx.cordisInspect.query(args.platform, args.provider, args.method, args.input, requireAgent(exec), exec.signal);
```

The host validates it against the method schema
and names the offending path `input`.
`@deepseek-ai/dsh-cordis-host-runner/lib/index.js`:

```js
function validateInput(platform, provider, method, input) {
	const violations = validateJsonSchemaValue(method.inputSchema, input ?? {}, "input");
	if (violations.length > 0) throw new Error(`${platform} Cordis inspect ${provider}.${method.name} rejected input: ${violations.join("; ")}`);
}
```

The failure is between those two files:
the model-facing tool-call marshaling delivers the `json` parameter
as a string,
so `validateJsonSchemaValue` sees a string where the schema wants
an object and reports `"input" must be an object`.
Measured:
both a JSON-string parameter value
and a nested object parameter value
arrive identically as strings,
in the live `web` session and in headless fixture sessions.

An earlier reading was wrong and is recorded here:
"this is a headless-runner serialization quirk;
the web session parses object inputs correctly."
The web session's identical failure,
quoted in the Symptom section,
disproves it.

## Verification

Version under test:
`dsh --version` prints `0.2.1-alpha.4`
from the mise npm install `npm-deepseek-ai-dsh` alpha v11.

Still reproduces unchanged on `dsh` 0.2.1-rc.2
(2026-10-21,
mise npm install `npm-deepseek-ai-dsh` next v11/1483ad):
every call that supplies `input` is rejected with the same message,
including an empty object
(`Tool.listTools` with `input: {}`),
while `input` omitted
(client `Slots.listSubTree`)
returns normally.

Harness:
call `cordis_inspect_query` from a live session
and compare input encodings.

Catalog that works cleanly:

- `cordis_inspect_query` with `platform: host`,
  `provider: Tool`,
  `method: listTools`,
  input omitted:
  full tool list with parameter schemas.
- `cordis_inspect_query` with `platform: host`,
  `provider: Config`,
  `method: listConfigs`,
  input omitted:
  first page of the entry directory.

Catalog that fails:

- `Config.listConfigs` with `input` naming a plugin filter,
  JSON-string encoding:
  `Host Cordis inspect Config.listConfigs rejected input: "input" must be an object`.
- `Config.listConfigs` with the same `input` as a nested object:
  identical message.
- `Service.listService` with `input: {"service": "tools"}`:
  identical message with `Service.listService` in the prefix.

## Verified workarounds

### Omit `input` and use only unfiltered discovery

`Tool.listTools` needs no input and returns the full registry,
which covers tool-presence and parameter-schema checks.

Tradeoffs:
`Config.listConfigs` returns only its first page
(the live tree holds 102 entries),
`Service.listService` and `Event.listEvents`
cannot select their exact contract,
and no check requiring a name filter,
page offset,
or entry id can run.

## What does not work

- Passing `input` as a JSON-string parameter value.
- Passing `input` as a nested object parameter value.
  Both arrive as strings and fail identically.
- Assuming the defect is headless-specific.
  The live web session fails the same way.
- Reaching entries past the first `Config.listConfigs` page
  without `offset`,
  which itself needs `input`.

## Upstream filing decision

`.out-of-scope/` holds no entry covering dsh tool marshaling
(checked `bun-install.md`,
`codex-harness.md`,
and the other listed exemptions).
The upstream repository is `deepseek-ai/deepseek-harness`
(`packages/extensions/tool-cordis`,
from the package `repository` field).

1. Is it really upstream's fault?
   Yes.
   The `json` parameter declaration,
   the unchecked forward,
   and the rejecting validator are all in upstream packages,
   quoted in the Root cause section.
2. Can upstream fix it?
   Yes.
   Either the tool-call marshaling parses `json` parameters,
   or `validateInput` accepts a JSON string and parses it.
3. Are they supporting this use case?
   Yes.
   `@deepseek-ai/dsh-tool-cordis/README.md:28`
   instructs filtered inspect queries,
   and the tool description tells agents
   `input must satisfy that method's schema`.
4. Would the repo welcome our contribution?
   Fails for issue filing:
   `gh repo view deepseek-ai/deepseek-harness --json hasIssuesEnabled`
   returns `false`,
   so the tracker accepts no issues at all.
   A pull-request channel is not investigated here.
5. Will they likely fix it?
   No signal exists to read,
   because the repository keeps no issue tracker.
6. Have we prototyped a minimal fix?
   Not reached,
   since constraint 4 fails and no filing channel exists.
   The fix site is upstream's tool marshaling,
   outside this repository's boundary.

Outcome:
do not file;
there is no channel.
The kept artifact is the draft below,
marked do not file as-is:

~~~md
Title: `cordis_inspect_query` `json` input arrives as a string,
so every filtered inspect query fails with `"input" must be an object`

In 0.2.1-alpha.4,
`cordis_inspect_query` declares
`input: { type: 'json' }`
(`packages/extensions/tool-cordis/lib/types/index.js`),
but the model-facing tool call delivers that parameter as a string.
`validateInput` in
`packages/extensions/cordis-host-runner/lib/index.js`
then rejects it:

`Host Cordis inspect Config.listConfigs rejected input: "input" must be an object`

Reproduction:
call `cordis_inspect_query` with any `input` object
(for example `Config.listConfigs` with `{"name": "..."}`).
Omitting `input` works.

Suggested fix:
parse `type: 'json'` tool parameters in the tool-call marshaling,
or accept and parse JSON strings in `validateInput`.
~~~
