# toml-eslint-parser 1.0.3: omitted version accepts TOML 1.1 and invalidates a TOML 1.0-only test assumption

## Symptom

A `parseTomlEdit` test expected the default parser to reject a newline inside an inline table.
Instead its helper threw `Error: Expected a TOML parse failure` because parsing succeeded.
The same source rejects when the caller explicitly passes `tomlVersion: '1.0'`.
Our `parse-toml-edit.ts` TSDoc had incorrectly called the parser default TOML 1.0.

## Root cause

`package/toml-edit/core/src/parse-toml-edit.ts:116` forwards `undefined` without selecting a version:

```ts
// package/toml-edit/core/src/parse-toml-edit.ts
return parseTOML(
  source,
  tomlVersion === undefined ? undefined : { tomlVersion, },
);
```

In `ota-meshi/toml-eslint-parser` tag `v1.0.3` (commit `6008e35499342f1116262162d16dbcab0c84760c`),
 `src/toml-parser/index.ts:65-68` forwards the omitted option into `getTOMLVer`:

```ts
// upstream src/toml-parser/index.ts
this.parserOptions = parserOptions || {};
this.tomlVersion = getTOMLVer(this.parserOptions.tomlVersion);
```

Upstream `src/parser-options.ts:35,52-53` selects TOML 1.1 for an omitted version:

```ts
// upstream src/parser-options.ts
const DEFAULT_TOML_VERSION: TOMLVer = TOML_VERSION_1_1;
export function getTOMLVer(v: TOMLVersionOption | undefined | null): TOMLVer {
  return (v && SUPPORTED_TOML_VERSIONS[v]) || DEFAULT_TOML_VERSION;
}
```

Upstream `src/toml-parser/index.ts:510-514` permits inline-table line breaks only at TOML 1.1:

```ts
// upstream src/toml-parser/index.ts
const needSameLine = this.tomlVersion.gte(1, 1)
  ? undefined
  : ("invalid-inline-table-newline" as const);
```

Upstream `README.md:51-53` also explicitly says the default is `"1.1.0"`.
The earlier TOML 1.0 reading was our stale comment,
 not an upstream parser defect.

## Verification

Tested with installed `toml-eslint-parser@1.0.3` and the source at tag `v1.0.3`.
Run this from `package/toml-edit/core`:

```sh
node --input-type=module -e "
import { parseTOML } from 'toml-eslint-parser';
const source = 'key = {foo=\"value\"\\n  , bar=\"value\"}\\n';
for (const version of [undefined, '1.0', '1.1']) {
  try {
    parseTOML(source, version === undefined ? undefined : { tomlVersion: version });
    console.log(version ?? 'default', 'accept');
  } catch (error) {
    console.log(version ?? 'default', error.constructor.name, error.message);
  }
}
"
```

Working catalog:

- Omitted version: `default accept`.
- Explicit TOML 1.1: `1.1 accept`.

Failing catalog:

- Explicit TOML 1.0:
  `1.0 ParseError No newlines are allowed between the curly braces unless they are valid within a value`.

The package-level regression in `package/toml-edit/core/src/parse-toml-edit.unit.test.ts`
 now tests all of these selections through the built package.

## Verified workarounds

- Request `parseTomlEdit({ source, tomlVersion: '1.0', })` when TOML 1.0 grammar is required.
  Tradeoff: this correctly rejects TOML 1.1-only inputs.
- Leave the version unset only when tracking the parser's default is intended.
  Our TSDoc now names the observed 1.1 default and a unit test pins the current result.
  Tradeoff: a future parser release may change its default,
  so callers needing a stable grammar should select a version explicitly.

## What does not work

Calling `parseTomlEdit({ source, })` and assuming TOML 1.0 rejection does not work:
 the test failed on the inline-table newline before changing its assertion to use `tomlVersion: '1.0'`.
The parser does not interpret an omitted option as the older grammar.

## Upstream filing decision

`.out-of-scope/` has no exemption for this parser or default-version class.
Searches of open and closed issues and pull requests in `ota-meshi/toml-eslint-parser`
 for `default tomlVersion 1.1` found no matching report.
No upstream filing is warranted:

- **Upstream fault:** No.
  The release's README and source agree on TOML 1.1;
  our local comment was wrong.
- **Upstream fixability:** A default can be changed in source,
  but no correction is needed for this behavior.
- **Supported use case:** Yes.
  `README.md:51-53` lists explicit 1.0 and 1.1 options.
- **Contribution welcome:** The tag contains a README but no CONTRIBUTING or issue/PR template was found.
  No ban was found,
  but no contribution is needed.
- **Likelihood of upstream fix:** No defect is established for them to fix.
- **Minimal compatible prototype:** No upstream patch was made because the local TSDoc and test were mistaken.
  The local test passed after explicitly selecting TOML 1.0 and pinning the default behavior.

## Upstream filing artifact

**Do not file as-is.**
This draft records why a report would misattribute our documentation error to upstream:

~~~md
Title: Omitted tomlVersion accepts inline-table newlines in toml-eslint-parser 1.0.3
Labels: none
Description: The parser defaults to TOML 1.1 by design (README.md:51-53,
src/parser-options.ts:35,52-53). An inline-table newline succeeds with no option
and fails with tomlVersion: '1.0'. This is documented behavior; our consuming
package incorrectly described the default as TOML 1.0.
Reproduction: Run the Verification command in this document.
Suggested fix: Correct the consuming package's TSDoc and tests, not upstream.
~~~
