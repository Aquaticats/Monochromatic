# Oxlint 1.78.0 JS plugin producer guidance cannot attach a cross-file diagnostic location

## Symptom

A JS plugin reports a problem at a consumer callback but proves that the applicable edit belongs in another file.
Oxlint can underline the consumer or a location in the consumer file,
but it cannot attach the producer as a related location.

Passing an extra `relatedLocations` field does not fail.
Oxlint silently emits only the primary consumer diagnostic:

```json
{
  "message": "consumer finding; producer at producer.js:1",
  "filename": "consumer.js",
  "labels": [
    {
      "span": {
        "line": 1,
        "column": 1
      }
    }
  ]
}
```

Passing the producer's line through `loc` does not select another file.
Oxlint interprets that line against the current consumer and can emit:

```text
RangeError: Line number out of range (line 100 requested).
Line numbers should be 1-based, and less than or equal to number of lines in file (2).
```

This surfaced in `prefer-readonly-parameter-types` when an inferred collection element was mutable.
The finding belonged to a later `filter`,
`reduce`,
or `sort` callback,
but the useful edit belonged to the mapping callback that produced the element type.

## Root cause

Source was checked at Oxc tag `oxlint_v1.78.0`,
commit `c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd`.
The installed `@oxlint/plugins` package is version `1.78.0`.

`apps/oxlint/src-js/plugins/report.ts:25` defines a diagnostic as one message plus one `node` or `loc`.
`DiagnosticBase` has no file or related-location field:

```ts
export type Diagnostic = RequireAtLeastOne<
  RequireAtLeastOne<DiagnosticBase, "node" | "loc">,
  "message" | "messageId"
>;

interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  node?: Ranged;
  loc?: LocationWithOptionalEnd | LineColumn;
  data?: DiagnosticData | null | undefined;
  fix?: FixFn;
  suggest?: Suggestion[] | null | undefined;
}
```

`apps/oxlint/src-js/plugins/report.ts:122` converts `loc` directly to offsets,
and the emitted internal report stores only numeric `start` and `end` offsets:

```ts
if (Object.hasOwn(diagnostic, "loc") && (loc = diagnostic.loc) != null) {
  // ...
  start = getOffsetFromLineColumn(startLineCol);
  // ...
}

diagnostics.push({
  message,
  messageId,
  start,
  end,
  ruleIndex: ruleDetails.ruleIndex,
  fixes: getFixes(diagnostic, ruleDetails),
  suggestions: getSuggestions(diagnostic, ruleDetails),
});
```

`apps/oxlint/src-js/plugins/report.ts:336` resolves those line and column values against module-global
`sourceText`,
which is the current linted file:

```ts
function getOffsetFromLineColumn(lineCol: LineColumn): number {
  // ...
  if (lines.length === 0) initLines();
  debugAssertIsNonNull(sourceText);
  // ...
  const lineOffset = lineStartIndices[line - 1];
  const offset = lineOffset + column;
  // ...
  return offset;
}
```

The restriction is architectural,
not a hidden field omitted from the published TypeScript declarations.
The public report object is converted to one current-file offset pair before it crosses into Rust.

Oxlint's [JS plugin documentation](https://oxc.rs/docs/guide/usage/linter/js-plugins.html) calls the API alpha
and says it aims at ESLint compatibility.
ESLint's ordinary rule-report descriptor also lacks cross-file related locations,
so this is not an Oxlint parity bug.

## Verification

### Version

- `oxlint`:
  `1.78.0`
- `@oxlint/plugins`:
  `1.78.0`
- source tag:
  `oxlint_v1.78.0`
- source commit:
  `c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd`

### Harness

`plugin.mjs`:

```js
export default {
  meta: { name: 'related-location-probe' },
  rules: {
    related: {
      meta: { messages: { finding: 'consumer finding; producer at producer.js:1' } },
      create(context) {
        return {
          Program(node) {
            context.report({
              node,
              messageId: 'finding',
              relatedLocations: [{ file: 'producer.js', line: 1, column: 1 }],
            });
          },
        };
      },
    },
  },
};
```

`oxlint.json`:

```json
{
  "jsPlugins": ["./plugin.mjs"],
  "rules": {
    "related-location-probe/related": "error"
  }
}
```

`consumer.js`:

```js
export const consumer = true;
```

Run:

```sh
oxlint --format json -c oxlint.json consumer.js
```

### Working catalog

- `node` reports the current consumer span.
- `loc` reports a valid line and column in the current consumer.
- A repository-relative producer `path:line` embedded in `message` survives JSON and terminal output.
- A one-line local binding subject remains countable with line-oriented tools.

### Failing catalog

- `relatedLocations` is ignored because it is not read by `DiagnosticBase` or `report`.
- A producer line passed as `loc` is resolved against `consumer.js`.
- An out-of-range producer line throws `RangeError` from `getOffsetFromLineColumn`.
- A valid producer line number can silently underline the same-numbered line in the consumer,
  which is worse than rejecting the location.

## Verified workarounds

### Put an eager producer identity in the message

Resolve the producer while the semantic snapshot is active,
then store immutable repository-relative path,
line,
and owner name in rule evidence.
Keep the consumer as the primary range:

```text
Parameter "entry" can be deeply readonly: property verdict is writable.
Its inferred parameter type originates in callable "toJudged" at
package/translation-repair/lane-contest/src/probe-telemetry.ts:398.
Likely edit: give that callable an explicit deeply readonly return type.
No exact type syntax was proved for that producer; run type checking after the edit.
```

The production implementation keeps this on one physical line.
The wrapped form is only for readable documentation.

Tradeoffs:

- the producer is not clickable as a secondary diagnostic range;
- path stability depends on repository layout;
- zero,
  partial,
  and multi-origin evidence must not be rendered as one producer;
- the plugin must resolve semantic handles eagerly before snapshot replacement.

The workaround was verified against the parent of `942258f04`.
Six consumer findings in `probe-telemetry.ts` all named `toJudged` at line 398,
and two comparator findings in `critic-attribution.ts` named `canonical` at line 280.
Applying `942258f04` removed all readonly findings from those files.

## What does not work

### Passing a nonstandard related-location property

JavaScript permits the extra field,
but `report` never reads it.
No secondary label appears in JSON output.

### Reusing `loc` with producer coordinates

`loc` carries no filename.
`getOffsetFromLineColumn` always uses current `sourceText`,
so a foreign coordinate either points to unrelated consumer text or throws.

### Moving the primary location to a cross-file producer

A rule context belongs to the current file.
The public API has no report descriptor that pairs a foreign filename with a range.
Pretending otherwise can produce a plausible but wrong consumer underline.

### Caching unresolved semantic handles

TypeScript project objects change with snapshot replacement.
Keeping a handle for later resolution risks stale or project-ambiguous provenance.
The workaround stores only eager strings and source identity.

## Upstream filing decision

No matching exemption exists in `.out-of-scope/`.
Searches across open and closed Oxc issues and pull requests for `JS plugin related location diagnostic`,
`context.report secondary location`,
and `related location diagnostic JS plugin` found no duplicate.

1. **Is it really upstream's fault?**
   No.
   Oxlint implements the ordinary ESLint-compatible report shape it documents.
   Cross-file related locations are an extension request,
   not a behavior bug.
2. **Can upstream fix it?**
   Yes.
   Oxc could extend the JS-to-Rust diagnostic report with foreign-file related spans.
3. **Are they supporting this use case?**
   Partly.
   Oxc supports custom JS rules and editor diagnostics,
   but does not claim cross-file related-location support.
4. **Would the repository welcome our contribution?**
   Yes with review and disclosure.
   `CONTRIBUTING.md` welcomes contributions and permits AI assistance when disclosed,
   understood,
   tested,
   and reviewed.
5. **Will they likely fix it?**
   Unknown but not declined.
   No matching issue,
   pull request,
   or documented non-goal was found.
6. **Have we prototyped a minimal upstream fix?**
   No.
   Constraint 1 fails,
   so the automatic upstream-prototype gate does not apply.
   The verified consumer-side workaround resolves this repository's user-facing failure.

### Upstream filing artifact

Do not file as-is.
This is a possible feature request only if a human decides clickable cross-file locations justify an Oxc extension.

~~~md
Title: Add related cross-file locations to JS plugin diagnostics

Oxlint 1.78.0 JS plugin diagnostics accept one current-file `node` or `loc`.
The report path converts that location to one `start`/`end` pair against current `sourceText`,
so a custom semantic rule cannot attach the foreign producer location that explains its consumer finding.

Reproduction:

- an extra `relatedLocations` field is silently ignored;
- passing a foreign line through `loc` resolves it against the consumer and can throw `RangeError`;
- embedding `producer/path.ts:line` in the message works but is not a clickable related span.

Relevant source:

- `apps/oxlint/src-js/plugins/report.ts:25` defines `DiagnosticBase` without related locations;
- `apps/oxlint/src-js/plugins/report.ts:122` converts `loc` to offsets;
- `apps/oxlint/src-js/plugins/report.ts:336` uses current-file `sourceText`.

A complete extension would need a JS descriptor for related locations,
serialization carrying filename plus span,
Rust diagnostic support,
and JSON/editor rendering tests.

AI assistance disclosure:
AI tools helped trace and draft this report.
A human is responsible for reviewing the Oxc 1.78.0 source trace,
running the reproduction,
and validating any proposed upstream change before filing.
~~~

[js-plugin-docs]: https://oxc.rs/docs/guide/usage/linter/js-plugins.html
