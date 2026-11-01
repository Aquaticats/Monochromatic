# GitHub CLI 2.100.0 JSON readbacks rewrite literal control escapes

## Symptom

An exact readback of repository issue #542 failed after successful creation.
The submitted Markdown contained literal `\u0000` text inside a JSON example.
Both `gh issue view --json body` and `gh api` returned `\^@` instead.
The verification driver reported an `AssertionError` comparing those strings.

This was not a failed issue creation,
a changed GitHub-stored body,
or a SHA-256 collision.
An independent unauthenticated HTTP response from the public REST endpoint
matched the submitted body exactly.
No issue content repair or duplicate creation was needed for that mismatch.
A separate digest-label line-wrapping correction is editorial,
not a response-sanitizer workaround.

The legacy pairing-key ambiguity described by #542 is a separate incident.
This document concerns the readback transport only.

## Root cause

The receiving CLI transport rewrites response JSON before the issue command or API command consumes it.
GitHub still stores the original issue body.

The inspected GitHub CLI source is tag `v2.100.0`,
commit `45437bc7eeeb3359bbfddd1742f79de7652fd3e2`.
Its `go.mod:21` selects `github.com/cli/go-gh/v2 v2.16.0`.
`api/http_client.go:72` constructs that library's client:

```go
// cli/cli, api/http_client.go
client, err := ghAPI.NewHTTPClient(clientOpts)
```

The inspected go-gh source is tag `v2.16.0`,
commit `c9808f266122bea7f5d7500d771c8134a0217af7`.
`pkg/api/http_client.go:77` installs the sanitizer transport:

```go
// cli/go-gh, pkg/api/http_client.go
transport = newSanitizerRoundTripper(transport)
```

The same file's `sanitizerRoundTripper.RoundTrip`,
starting at line 257,
wraps JSON response bodies after the underlying request completes:

```go
// cli/go-gh, pkg/api/http_client.go
resp, err := srt.rt.RoundTrip(req)
if err != nil || !jsonTypeRE.MatchString(resp.Header.Get(contentType)) {
    return resp, err
}
```

Its replacement reader at line 266 is:

```go
// cli/go-gh, pkg/api/http_client.go
Reader: transform.NewReader(resp.Body, &asciisanitizer.Sanitizer{JSON: true}),
```

`pkg/asciisanitizer/sanitizer.go:70` examines six-byte JSON escape windows.
It does not use preceding backslashes to exclude literal escape text from replacement.
The `addEscape` state instead adds escaping so the replacement remains valid JSON:

```go
// cli/go-gh, pkg/asciisanitizer/sanitizer.go
if t.JSON && len(src) >= 6 {
    if repl, found := mapJSONControlToCaret(src[:6]); found {
        if t.addEscape {
            repl = append([]byte{'\\'}, repl...)
            t.addEscape = false
        }
```

The mapping at `pkg/asciisanitizer/sanitizer.go:188` includes:

```go
// cli/go-gh, pkg/asciisanitizer/sanitizer.go
`\u0000`: `^@`,
```

Consequently,
JSON containing an escaped backslash followed by `u0000` remains valid JSON after transport,
but the decoded Markdown string changes from literal `\u0000` to literal `\^@`.

This is an explicit current transport behavior,
not evidence that GitHub changed stored text.
GitHub CLI's `api/http_client_test.go:367` expects caret replacements even in escaped examples:

```go
// cli/cli, api/http_client_test.go
assert.Equal(t, "Escaped ^[ \\^[ \\^[ \\\\^[", issue.ActiveLockReason)
```

`pkg/cmd/api/api.go:342` also states:

```go
// cli/cli, pkg/cmd/api/api.go
// JSON is sanitized by the transport and the jq/template/jsoncolor paths emit
// our own formatting, so only a raw non-JSON body needs neutralizing
```

## Verification

Installed binary:

```text
# gh --version
gh version 2.100.0 (2026-09-03)
```

The reproducing public issue is
[Monochromatic #542](https://github.com/Aquaticat/Monochromatic/issues/542).
These read-only commands were exercised:

```bash
# Readbacks pass through the same response sanitizer.
gh issue view 542 --repo Aquaticat/Monochromatic --json body,title,state

gh api repos/Aquaticat/Monochromatic/issues/542
```

### Clean catalog

- The ordinary issue title matches the raw response.
- The Markdown body outside the identified escape replacements matches exactly,
  including its ordinary text,
  quotes and line breaks.
- The unfiltered public HTTP body matches the complete local submitted body.

### Changed catalog

- Literal `\u0000` within `Cat\u0000Dog` becomes `\^@` within `Cat\^@Dog`.
- The corresponding occurrence in `Dog\u0000Owl` changes the same way.
- Both `gh issue view --json` and `gh api` produce that changed value.

The retained verification asserts that these two substitutions account for the complete difference.
That diagnostic comparison does not approve rewritten CLI data as original response bytes.

Durable local evidence lives under
`package/module/translation-repair/node_modules/.monochromatic/preparation-input-reader/gh-json-observation/`.
Its `verification.json` records source-to-copy paths,
source hashes,
raw and CLI responses,
and the exact comparison results.
The sibling `harness/read-key-issue-raw.mjs` performs the verified raw-response check.

## Verified workaround

Use an unfiltered HTTP client for byte-sensitive comparison of this public endpoint.
Continue using `gh` for ordinary issue operations.
Do not render untrusted raw response text directly in a terminal.

The observed workaround used Node 26.8.2 without authentication or supplied credentials:

```js
// Public-issue readback, independent of the gh response transport.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const response = await fetch(
  'https://api.github.com/repos/Aquaticat/Monochromatic/issues/542',
  { headers: {
    'user-agent': 'Monochromatic integrity verification',
    accept: 'application/vnd.github+json',
  } },
);
assert.equal(response.status, 200);
const issue = await response.json();
const expected = await readFile(process.argv[2], 'utf8');
assert.equal(issue.body, expected);
assert.equal(issue.state, 'open');
```

This preserves exact received text for verification rather than applying terminal-display policy to it.
The tradeoff is that the caller owns safe storage and display of untrusted content.
This verification covers the public endpoint;
it does not prescribe authentication handling for private resources.

## What does not work

- Switching from `gh issue view --json` to `gh api` did not recover original field contents.
  Both actual readbacks traversed the sanitizer transport.
- Treating the failed comparison as failed creation would be incorrect.
  Creation returned #542 successfully,
  and the unfiltered response confirmed its exact body.
- Replacing `\^@` with `\u0000` is not the workaround.
  That reversal would conflate genuinely stored caret text with transformed escape text.
  The accepted evidence comes from the independent raw response.

## Upstream filing decision

No upstream issue,
comment or patch is proposed from this consumer-boundary incident.

1.  Upstream fault is not established by a consumer expecting unfiltered bytes from this CLI transport.
    Sanitized JSON and the escaped-example substitutions are explicit in current source and tests.
2.  Changing upstream policy is technically separate from restoring this observer's fidelity.
    The verified consumer-side HTTP path already meets that requirement.
3.  The examined CLI source supports sanitized JSON output.
    No promise of byte-identical response fields was established.
4.  Contribution welcome and AI-filing policies were not evaluated for a proposed upstream change,
    because no upstream change or filing is being proposed.
5.  No maintainer rejection is inferred from silence.
    The related history concerns a different,
    already-fixed invalid-JSON failure.
6.  No upstream prototype was made.
    The upstream-fault condition was not established,
    so the automatic prototype gate does not apply.

The repository's `.out-of-scope/` inventory contained no matching GitHub CLI exemption.
Searches for `sanitize JSON escape` and `sanitize escape` found no matching issue.
Broader `sanitizer` searches across cli/cli and cli/go-gh found related history,
which was read rather than treated as proof of the current failure:

- [cli/cli #8336](https://github.com/cli/cli/issues/8336)
  reported `invalid character '^' in string escape code`.
- [cli/go-gh #145](https://github.com/cli/go-gh/pull/145)
  repaired multiple escaped control sequences producing invalid JSON.
- The installed sanitizer contains the `t.addEscape = false` reset associated with that repair.
  Current responses parse successfully;
  the incident here is changed field contents,
  not that historical invalid-JSON crash.
- [cli/cli #6954](https://github.com/cli/cli/issues/6954)
  concerns sanitizer buffer-length handling,
  not this successful-but-transformed readback.

The upstream filing artifact is therefore an explicit nothing-to-file note,
not a speculative patch or a duplicate report.
