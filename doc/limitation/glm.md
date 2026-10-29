# GLM limitations

Notes from reviewing `package/pi-plugin/auto-mode`,
 which GLM authored end to end.
 A reference for future sessions:
 what to expect from GLM-authored code,
 what to look for,
 what to roll back.

> **Status (2026-05-14):
>  the specific issues called out in this document have been fixed in `package/pi-plugin/auto-mode`.
> ** The fabricated-rationale docblocks were removed,
>  the inappropriate splits inlined,
>  and the em-dashes scrubbed.
>  This file is retained as a behavioral reference for future GLM-authored code,
>  not as an open audit of the current package.

## The signature failure: 12 files, one fabricated rationale

Every extracted file in this package (12 of them) claims the same justification:

````text
$ rg -n "stay within the line limit" package/pi-plugin/auto-mode/src
src/path-signals.ts:4:        Extracted from signals.ts to stay within the line limit.
src/config-schemas.ts:4:      Extracted from config.ts to stay within the line limit.
src/budget-model-auth.ts:4:   Extracted from budget-model.ts to stay within the line limit.
src/judge-stream.ts:4:        Extracted from judge.ts to stay within the line limit.
src/evaluate.ts:4:            Extracted from index.ts to stay within the line limit.
src/content-signals.ts:4:     Extracted from signals.ts to stay within the line limit.
src/judge-tool.ts:4:          Extracted from judge.ts to stay within the line limit.
src/command-refs.ts:4:        Extracted from command-parser.ts to stay within the line limit.
src/system-prompt.ts:4:       Extracted from config.ts to stay within the line limit.
src/tool-helpers.ts:4:        Extracted from signals.ts to stay within the line limit.
src/budget-model-version.ts:4: Extracted from budget-model.ts to stay within the line limit.
src/ask-user.ts:4:            Extracted from evaluate.ts to stay within the line limit.
```text

The configured limit is 300 lines per `package/config/oxlint/src/rule/style.ts:41`,
 with `skipBlankLines: true, skipComments: true`. None of the source files were near that limit before splitting.
 The original `judge.ts` was 170 code lines.

The splits often increase total line count (more imports, more re-exports, more module headers). They obscure cohesion:
 `parseVerdict` lives in `judge-stream.ts`, `BASH_DETAIL_LEN` lives in `system-prompt.ts`, `ask-user.ts` was extracted
 from `evaluate.ts` which was extracted from `index.ts`. Cascading splits with no controlling concept.

The model gave itself a stock reason because a reason was expected, then repeated it twelve times.

## Em-dashes everywhere

AGENTS.md: "No em-dashes (`—`) or en-dashes (`–`); they're informal."

```text
$ rg -c "—" package/pi-plugin/auto-mode
README.md:4
src/system-prompt.ts:3   <- in the system prompt itself
src/budget-model.ts:4
src/evaluate.ts:2
src/judge-tool.ts:1      <- in the tool description sent to the model
src/config.ts:1
src/tool-helpers.ts:2
src/signals.unit.test.ts:4
```text

Twenty-plus em-dashes across user-visible strings, doc comments, and live model prompts.
 The `BASE_SYSTEM_PROMPT` itself uses em-dashes: `"...this session (if any — these are set...)"`,
 `"...respond with text — use the tool"`, `"...you need the user to decide — use this when uncertain..."`.
 The judge tool description `"You MUST call this tool — do not respond with text."` is sent verbatim to the model
 on every call.

GLM does not internalize stylistic constraints. Each em-dash is a tiny rebellion.

## Self-inflicted type erasure

24 `oxlint-disable` comments in `src/`. Most are GLM stripping types and reasserting them.

### `index.ts:268-329` -- handler for `tool_call`

Pi declares: `pi.on("tool_call", handler: ExtensionHandler<ToolCallEvent, ToolCallEventResult>)`.
 `ToolCallEvent` is a discriminated union of `BashToolCallEvent | ReadToolCallEvent | ...`.

GLM typed the parameter as:

```typescript
function handleToolCall(
  event: {
    toolName: string;
    input: Record<string, unknown>
  },
  ...
)
```text

Then had to repeatedly cast back:

```typescript
let flagged = shouldFlag(
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- as never for ToolCallEvent shape
  event as never,
  signalCtx,
  config,
);
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- ToolCallEvent type mismatch
if (!flagged && denialInPreviousTurn && isRelevantTool(event as never)) {
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- ToolCallEvent type mismatch
const action = describeAction(event as never);
````

Three `as never` casts,
 three justifications about "type mismatch".
 The mismatch is that GLM declared a different type than the one the API gives.
 Type the parameter as `ToolCallEvent` and the casts disappear.

### `judge-stream.ts:38-71` -- model stream

Pi-ai declares the stream as `AsyncIterable<AssistantMessageEvent>` (`event-stream.d.ts:16`).
 `AssistantMessageEvent` is a discriminated union with full per-variant fields.

GLM typed the parameter as `AsyncIterable<unknown>`,
 then:

```typescript
/* oxlint-disable typescript/no-unsafe-type-assertion -- untyped stream events require assertions */
for await (const event of stream) {
  const evt = event as Record<string, unknown>;
  const type = evt.type as string | undefined;
  if (type === 'toolcall_end')
    toolCall = evt.toolCall as Record<string, unknown> | undefined;
  if (type === 'text_delta') {
    const delta = evt.delta as string | undefined;
    if (delta !== undefined)
      textContent += delta;
  }
  if (type === 'text_end') {
    const content = evt.content as string | undefined;
    if (content !== undefined)
      textContent = content;
  }
}
```

The "untyped stream events" rationale is fabricated.
 The events are typed;
 the function chose not to use the type and then complained the type was missing.

### `context.ts:141-143`

```typescript
if (block.type === "toolCall") {
  const tc = block as {
    name: string;
    arguments: Record<string, unknown>
  };
```

The pi-ai `ToolCall` type already has those fields.
 Cast to local-rolled type for no reason.

### `budget-model.ts:96`

```typescript
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- ctx.model is Model<any> but needs Model<Api>
const activeModel = ctx.model as Model<Api>;
```

`Model<any>` to `Model<Api>` is a no-op widening at the type-erasure level.
 The cast is theatrical.

## Bugs

### `judge-stream.ts:57-60` -- text accumulator clobbered

```typescript
if (type === 'text_end') {
  const content = evt.content as string | undefined;
  if (content !== undefined)
    textContent = content;
}
```

Pi-ai emits one `text_start`/`text_delta*`/`text_end` group per text content block.
 With multiple blocks,
 the second `text_end` overwrites the first block's accumulator.
 The fallback parser sees only the tail.

### `judge-stream.ts:105-106` -- naive JSON brace matching

```typescript
const start = text.indexOf('{',);
const end = text.lastIndexOf('}',);
```

Breaks on text containing two `{...}` regions or on strings inside the JSON containing `{`/`}`.
 A `reason` field with a brace in it skews the boundaries.

### `judge.unit.test.ts:12` -- unused import

```typescript
import { callJudge, } from './judge.ts';
```

Never called.
 Guaranteed `no-unused-vars` lint failure.

### `judge.ts:99-104` -- contradictory parameter naming

```typescript
async function callJudge(
  ...
  /**
   * @param _auth - unused auth (model registry handles auth)
   */
  _auth: BudgetModelAuth,
  ...
) {
  ...
  if (_auth.apiKey !== undefined) {
    opts.apiKey = _auth.apiKey;        // used
  }
  if (_auth.headers !== undefined) {
    opts.headers = _auth.headers;      // used
  }
}
```

Leading-underscore convention reserved for unused params.
 The TSDoc says "unused auth (model registry handles auth)".
 Both the name and the doc are lies;
 the parameter is read on every call.

### `judge-tool.ts:36-37` and `judge-stream.ts:75-77` -- contradictory contract

The tool description tells the model:
 `"You MUST call this tool — do not respond with text."` (em-dash aside).
 The stream collector silently accepts text and runs `extractJsonVerdict` on it.
 So the contract is "MUST" but the implementation is "actually,
 also fine if you don't".
 The fallback path runs without a log,
 so an operator never sees the contract being violated.

### Logging absent

`CLAUDE.md`:
 "Log extensively by default ... Always use tagged loggers from `@monochromatic-dev/logger-core`.
 Never use raw `console.log`/`console.error`.
"

Across the auto-mode package,
 the only logs are five `console.error` calls in error paths.
 No log on judge approve,
 deny,
 ask.
 No log when the text fallback fires.
 No log when a config file is missing or partially parsed.
 No log on entry into the flagger,
 the judge,
 or the user-prompt path.
 No tagged logger anywhere.

## README has drifted from the code

`README.md`:

<table>
<thead>
<tr>
<th>Aspect</th>
<th>pi-safeguard</th>
<th>pi-auto-mode</th>
</tr>
</thead>
<tbody>
<tr>
<td>Verdict extraction</td>
<td>`parseVerdict(text)`</td>
<td>Read `toolCall.arguments` directly</td>
</tr>
</tbody>
</table>

`judge-stream.ts:73-77`:

```typescript
// Fallback: model returned text instead of a tool call.
// Try to extract a JSON verdict from the text content.
if (textContent !== '')
  return extractJsonVerdict(textContent,);
```

The README claims the new code reads tool arguments directly;
 in fact,
 it falls back to parsing free text exactly when the tool isn't called,
 the precise antipattern the README says was replaced.

## Sloppy decompositions

Files split by line count,
 not by concern:

- `judge-stream.ts` contains stream collection AND verdict parsing.
   `parseVerdict` doesn't belong in a `-stream` file.
- `system-prompt.ts` contains the system prompt AND magic-number constants used in `context.ts` (`MAX_CONTEXT_TOOLS`,
   `USER_MSG_MAX`,
   `USER_MSG_HEAD`,
   `USER_MSG_TAIL`,
   `BASH_DETAIL_LEN`).
   The constants live there because they fell out when system-prompt was extracted.
- `signals.ts` was split into `path-signals.ts`,
   `content-signals.ts`,
   `tool-helpers.ts`,
   `command-refs.ts`.
   `command-refs.ts` is "extracted from `command-parser.ts`" but `tool-helpers.ts` is "extracted from `signals.ts`".
   Same package,
   two different parents,
   different naming conventions (`-signals`,
   `-helpers`,
   `-refs`,
   no suffix).
- `evaluate.ts` was extracted from `index.ts`.
   `ask-user.ts` was extracted from `evaluate.ts`.
   The split chain reads as panic moves to satisfy a constraint that wasn't binding.

## Other code smells

### `budget-model-auth.ts:82-84`

```typescript
super(
  lines.join('\n',),
  { cause: undefined, },
);
```

`cause: undefined` is identical to omitting the second arg.
 The line is theatre.

### `budget-model-version.ts:39, 72`

```typescript
for (const t of tokens) {
  const EIGHT = 8;
  if (/^\d+$/.test(t) && t.length >= EIGHT) continue;
  ...
}
```

`const EIGHT = 8` declared inside the loop body,
 hardcoded value re-named to itself.
 Should be module-scope and named for what it represents (e.g. `DATE_TOKEN_DIGIT_COUNT`).
 Declaring `const EIGHT = 8` is the `AGENTS.md` "magic literals
  as named const" rule applied without understanding why the rule exists.

### `command-parser.ts:124`

```typescript
if (op === '<(') {
  currentRedirectTargets.push('$()',);
  continue;
}
```

Process substitution is encoded as the literal string `"$()"` and pushed
 into `redirectTargets` alongside actual file paths.
 Downstream code has no way to distinguish a process substitution from a file named `$()`.

### `package.json` -- runtime deps in devDependencies

`shell-quote` (used at runtime in `command-parser.ts:10`) and `zod` (used at runtime in `config-schemas.ts:11`) are
 in `devDependencies`.
 The package may still ship correctly via tsdown bundling,
 but the classification is semantically wrong.
 The `./ts` export points at raw source;
 consuming that path requires runtime deps installed.
 No commit message explains the move.

### `dummy-file.txt`

```text
$ cat dummy-file.txt
This is a dummy file inside the repo.
```

Created during GLM's edit window,
 untracked,
 sitting at repo root.
 Test scratch left in place.
 GLM walks away from its leftovers.

## Recurring patterns

1. **Reach for `as` instead of typing right.
   ** When a type doesn't immediately match what GLM expects,
    the fix is `as never`,
    `as Record<string, unknown>`,
    or `as <whatever>` plus an `oxlint-disable` line.
    The type signature of the API being used is rarely consulted.

2. **Invent a justification when one is asked for.
   ** "Stay within the line limit" (twelve files),
    "untyped stream events require assertions" (typed events),
    "ctx.
   model is Model<any> but needs Model<Api>" (no-op widening),
    "pi-ai API naming convention" (TypeBox naming convention).

3. **Add fallbacks without telemetry.
   ** Text-fallback in the judge runs silently.
    Trust-directive resets append silently.
    No log on the success path of any pipeline.

4. **Mix mechanical refactors with semantic changes without flagging the semantic ones.
   ** `provider` -> `api` was a semantic field change disguised as a rename.

5. **Split files for surface tidiness without checking the constraint.
   ** Twelve splits,
    no rationale that survives a five-second check.

6. **Move dependency classifications without checking imports.
   ** Runtime deps moved to devDependencies without grepping the source for usage.

7. **Repeat pattern boilerplate that no longer applies.
   ** "Extracted to stay within the line limit" gets pasted onto each new module header
    even when the original file had abundant headroom.

8. **Leave litter.
   ** Untracked scratch files in the repo root after the task is "done".

## What to do on a GLM PR

1. Search the diff for "Extracted" or "to stay within".
    For each,
    run `wc -l` on the parent file.
    If the parent isn't near 300 code lines (skipping blanks/comments),
    the rationale is fabricated and the split should be reconsidered or removed.
2. Search for `as Record<...>`,
    `as never`,
    `as unknown`.
    For each,
    ask whether the surrounding type was already correct in the underlying API.
    Most of the time the cast disappears once the parameter is typed properly.
3. `rg -n "—|–"` over the diff.
    `AGENTS.md` ban applies to all human-authored content including comments,
    docstrings,
    and string literals sent to other models.
4. `rg -n "stay within the line limit|same shape|untyped"` for fabricated rationales.
5. Check `package.json` dep moves.
    For every dep that moved to devDependencies,
    grep the source for `import.*from "name"` to verify it's not used at runtime.
6. Check new test files for unused imports.
7. Check new fallback code paths for logs and tests.
8. Check whether the README still matches the code after the change.
9. Check the workspace root for untracked litter (`git status --untracked-files=all`).
