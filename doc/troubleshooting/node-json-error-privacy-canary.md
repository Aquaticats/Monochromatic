# Node 26.8.2 JSON.parse diagnostics can defeat whole-input privacy canaries

## Symptom

A translation-repair namespace guard-removal test passed after raw `SyntaxError` causes were restored.
The test checked that the complete input `private-fixture-invalid-plan` was absent from the rendered error.
Node `26.8.2` instead emitted only part of that input:

```text
Unexpected token 'p', "private-fi"... is not valid JSON
```

Absence of the complete string did not prove absence of private input bytes.
The production sanitizer was not removed from the development worktree.
The failing evidence was the test's ability to detect its removal in an isolated worktree.

A short input produces a different diagnostic:

```text
Unexpected token 'q', "q7z9k2" is not valid JSON
```

Keeping that native error as an `Error.cause` exposes the complete short input through `util.inspect`.

## Root cause

Source was inspected from the read-only Node clone at tag `v26.8.2`,
commit `f2f2c2f246c36bd74f082cb43ecfe830657d81c9`:
`/var/home/user/temp/agent/node-json-diagnostic-20260912`.
The extracted files and source identity are recorded by
`/var/home/user/temp/agent/node-json-source-20260912.out`.

In `deps/v8/src/json/json-parser.cc:525`,
`ReportUnexpectedToken` chooses the diagnostic through `LookUpErrorMessageForJsonToken`:

```cpp
// deps/v8/src/json/json-parser.cc
MessageTemplate message =
    errorMessage ? errorMessage.value()
                 : LookUpErrorMessageForJsonToken(token, arg, arg2, pos);
```

The default token path calls `GetErrorMessageWithEllipses` at
`deps/v8/src/json/json-parser.cc:472`:

```cpp
// deps/v8/src/json/json-parser.cc
message = GetErrorMessageWithEllipses(arg, arg2, pos);
```

That helper selects a substring for context-bearing diagnostics.
Its beginning-of-input branch at `deps/v8/src/json/json-parser.cc:418` says:

```cpp
// deps/v8/src/json/json-parser.cc
if (origin_source_length >= kMinOriginalSourceLengthForContext) {
  int substring_start = 0;
  int substring_end = origin_source_length;
  if (pos < kMaxContextCharacters) {
    message =
        MessageTemplate::kJsonParseUnexpectedTokenStartStringWithContext;
    // Output the string followed by ellipses.
    substring_end = pos + kMaxContextCharacters;
```

The short-string branch at `deps/v8/src/json/json-parser.cc:441` retains the original source:

```cpp
// deps/v8/src/json/json-parser.cc
} else {
  arg2 = original_source_;
  // Output the entire string without ellipses but provide the token which
  // was unexpected.
  message = MessageTemplate::kJsonParseUnexpectedTokenShortString;
}
```

The templates at `deps/v8/src/common/message-template.h:591` and `:597` make the distinction visible:

```cpp
// deps/v8/src/common/message-template.h
T(JsonParseUnexpectedTokenShortString,
  "Unexpected token '%', \"%\" is not valid JSON")
T(JsonParseUnexpectedTokenStartStringWithContext,
  "Unexpected token '%', \"%\"... is not valid JSON")
```

The final throw retains those arguments in a native syntax error,
`deps/v8/src/json/json-parser.cc:554`:

```cpp
// deps/v8/src/json/json-parser.cc
isolate()->ThrowAt(factory->NewSyntaxError(message, arg, arg2, arg3),
                   &location);
```

The incorrect assumption was ours:
searching for the complete long input was treated as a privacy witness without first proving the native error contained it.

## Verification

Executed on Node `26.8.2`:

```bash
# Repository working directory
node /var/home/user/temp/agent/probe-node-json-canary-20260912.mts
```

The provider-free probe is retained with its output at
`/var/home/user/temp/agent/node-json-canary-probe-r2-20260912.json`.
It parses these control catalogs:

- Valid JSON:
  `{}` and `{"fixture":"q7z9k2"}` both parse.
- Short invalid input:
  `q7z9k2` appears completely in the native diagnostic and in a raw cause.
- Long invalid input:
  `private-fixture-invalid-plan` is not present completely,
  even though the native diagnostic exposes its `private-fi` prefix.
- Sanitized cause:
  retaining only `{name: error.name}` does not expose the short input in the rendered wrapper.

The production test in
`package/module/translation-repair/src/preparation-attempt.unit.test.ts` (removed before 2026-10-18)
now first proves native exposure of its short `canary` input,
then checks that `createPreparationAttempt` omits it and allocates no directory.
R8 build,
types and focused tests pass;
its remaining formatter finding was the new `try` statement layout,
corrected in `b0824a634`.
R9 passes all source verification,
including the full suite's `unit exit 0` at line 9886.
The revised isolated run detects removal of the parser-cause sanitizer with the designated ordinary assertion.
The complete R5 namespace proof subsequently passes all 26 removals,
including the parser-cause case,
on independently recorded Node `26.8.2` test processes.
Its restored build and tests pass;
records are retained under `/var/home/user/temp/agent/namespace-guard-proof-20260912`.

## Verified workarounds

The existing consumer boundary in
`package/module/translation-repair/src/create-preparation-attempt.ts` (removed before 2026-10-18)
retains only the parser error's class name:

```ts
// package/module/translation-repair/src/create-preparation-attempt.ts (removed before 2026-10-18)
cause: { name: error.name, },
```

This discards the native message,
input excerpts and parser stack from the retained cause.
The tradeoff is reduced parser detail;
the wrapper still identifies the namespace operation and directory.
It does not sanitize unrelated filesystem errors or make arbitrary logging private.

For tests,
use a sentinel whose presence in the native diagnostic is asserted first.
Choose a sentinel absent from filenames and stack labels.
This tests input disclosure rather than incidental stack text.

## What does not work

- Checking only the top-level wrapper message misses a raw nested cause.
- Checking absence of an entire long input misses truncated excerpts.
  The first namespace guard run demonstrated this by surviving the raw-cause mutation.
- The initial standalone probe used `canary` while its filename also contained `canary`.
  Its sanitized wrapper still matched because of the stack's filename.
  Replacing the probe sentinel with `q7z9k2` separated input disclosure from stack metadata.
- Passing the ordinary suite was insufficient evidence that the privacy guard was tested.
  The isolated mutation exposed the missing positive control.

## Upstream filing decision

- Fault:
  no upstream defect established.
  V8's source explicitly chooses contextual or complete diagnostic text;
  our privacy test incorrectly assumed complete-input exposure.
- Fixability:
  no upstream change is required for the consumer sanitizer or corrected witness.
- Supported use:
  invalid JSON produces a syntax error in the measured runtime.
  No guarantee of complete-input diagnostic rendering was established.
- Contribution policy:
  not evaluated for a patch because no upstream defect or contribution is proposed.
- Maintainer direction:
  not inferred.
  Node issue searches for `JSON.parse truncate error` and `JSON.parse "error message"`,
  plus a pull-request search for `JSON.parse truncate`,
  returned no rows in this investigation.
  That is not evidence that no related discussion exists.
- Prototype:
  the consumer-side positive control and sanitized-cause probe were executed.
  No upstream prototype is justified when the diagnosed defect is in our test.

The `.out-of-scope/` inventory was checked;
no exemption is being used to justify a filing.
Nothing to add upstream and no external issue,
comment or patch was sent.
