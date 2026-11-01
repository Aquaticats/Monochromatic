# GitHub CLI 2.102.0 changes escaped controls in raw JSON manifest downloads

## Symptom

An exact-byte audit manifest downloaded with `gh api` and
`Accept: application/vnd.github.raw+json` had a different digest from its committed bytes.
Adding `--allow-escape-sequences` produced the same transformed bytes.

The fixture at commit `8c70741076b02238bc9dab9fc10443dd9aea77e2` is
`doc/audit/depot-final-recovery-2026-10-17.evidence.json`:

- Committed bytes:
  1,043,537.
  SHA-256:
  `1a6f6aaee60910a6fe44c653308648ba8a19abf2dc3713ebe05e227ff1390369`.
- Raw JSON CLI bytes:
  1,030,081.
  SHA-256:
  `c33dfcb8b87ba5e45a5d275f82d38ce7d6542d4531eb1b7dbfb47df122a5d0cf`.
- The first difference is byte offset 58,924:
  the JSON escape `\u001b` becomes the inert text `^[`.

The JSON remains parseable,
but its bytes and string values have changed.
There is no error diagnostic for this transformation.

## Root cause

The installed `gh` version is `2.102.0`,
source tag `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`.
Its `go.mod:21` selects `github.com/cli/go-gh/v2 v2.16.1`.
The dependency's tag resolves to `37aa5bbaf1a591aa134913efeb3775f0235dc5ab`.
Both source repositories were shallow-cloned at those exact tags and left unmodified.

In `go-gh`,
`pkg/api/http_client.go:77` installs `newSanitizerRoundTripper`.
The response path at line 257 wraps bodies whose content type matches JSON:

```go
// cli/go-gh: pkg/api/http_client.go:257
func (srt sanitizerRoundTripper) RoundTrip(req *http.Request) (*http.Response, error) {
    resp, err := srt.rt.RoundTrip(req)
    if err != nil || !jsonTypeRE.MatchString(resp.Header.Get(contentType)) {
        return resp, err
    }
    sanitizedReadCloser := struct {
        io.Reader
        io.Closer
    }{
        Reader: transform.NewReader(resp.Body, &asciisanitizer.Sanitizer{JSON: true}),
        Closer: resp.Body,
    }
    resp.Body = sanitizedReadCloser
    return resp, err
}
```

In `pkg/asciisanitizer/sanitizer.go:69`,
the JSON branch recognizes six-byte escaped control sequences.
The mapping at line 211 explicitly replaces `\u001b` with `^[`:

```go
// cli/go-gh: pkg/asciisanitizer/sanitizer.go:211
`\u001b`: `^[`,
```

GitHub CLI's `pkg/cmd/api/api.go:342` states that JSON is sanitized by the transport.
Its opt-out flag gates only the separate non-JSON copy guard at line 538:

```go
// cli/cli: pkg/cmd/api/api.go:538
if !isJSON && !opts.AllowEscapeSequences && bodyWriter != io.Discard {
    err = iostreams.CopyGuardedContent(bodyWriter, responseBody, opts.IO.IsStdoutTTY())
```

The transport has already transformed JSON before this branch.
GitHub CLI's `api/http_client_test.go:333` also verifies JSON control sanitization,
including conversion to caret notation.
This is intentional terminal-content handling,
not evidence that GitHub's stored blob changed.

The stale-branch hypothesis was rejected:
both `main` and the exact fixture commit returned the same transformed bytes,
while local bytes exactly matched `git show` at that commit.

## Verification

Working cases:

- Authenticated Contents API retrieval with `application/vnd.github.object+json`,
  followed by base64 decoding,
  preserved all 1,043,537 fixture bytes and the committed SHA-256.
- The same byte-preserving retrieval succeeded at the exact fixture commit.
- A deliberately wrong expected digest was rejected.

Failing cases:

- Raw JSON retrieval changed escaped controls at both tested refs.
- `--allow-escape-sequences` did not bypass the JSON transport transformation.

The actual receipts are `commonManifestTransportControls` and
`desktopSessionContinuationPreparation` in the current audit evidence file.
The raw-response comparison is retained privately as `transport-inspection.json`;
it records both digests and escaped excerpts around the first difference.

## Verified workaround

Use the Contents API object response and decode `content` before hashing.
The [official repository-contents API][contents-api] documents base64 content for this file-size range,
the object media type,
and selection by commit ref.
The following task uses the exact exercised request and fixture digest:

```toml
# mise.toml: an exact-byte manifest verification task
[tasks."verify:manifest-bytes"]
shell = "node --input-type=module-typescript -e"
run = '''
const { execFileSync } = await import('node:child_process');
const { createHash } = await import('node:crypto');
const { strict: assert } = await import('node:assert');
const path = 'doc/audit/depot-final-recovery-2026-10-17.evidence.json';
const ref = '8c70741076b02238bc9dab9fc10443dd9aea77e2';
const response = JSON.parse(execFileSync('gh', [
  'api', '--method', 'GET',
  '--header', 'Accept: application/vnd.github.object+json',
  `repos/Aquaticats/Monochromatic/contents/${path}?ref=${ref}`,
], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 }));
assert.equal(response.type, 'file');
assert.equal(response.path, path);
assert.equal(response.encoding, 'base64');
const bytes = Buffer.from(response.content, 'base64');
const sha256 = createHash('sha256').update(bytes).digest('hex');
assert.equal(sha256, '1a6f6aaee60910a6fe44c653308648ba8a19abf2dc3713ebe05e227ff1390369');
JSON.parse(bytes.toString('utf8'));
console.log(JSON.stringify({ bytes: bytes.length, sha256 }));
'''
```

This adds an explicit decoding step and relies on the API supplying base64 content.
It is not a verified general download recipe for larger files whose object response omits content.
The final common cohort manifest is constrained to less than 1 MiB;
its expected digest comes from the separately frozen experiment receipt.
Never treat successful decoding alone as a digest match.

## What does not work

- Redirecting raw JSON output does not remove a transformation performed by the HTTP transport.
- `--allow-escape-sequences` selects the non-JSON copy behavior,
  so it did not repair these JSON bytes.
- Comparing parsed object structure misses altered string values and cannot prove byte identity.
- Advancing the branch ref does not explain the observed mismatch:
  the exact immutable commit reproduced it.

Raw non-JSON job logs have a different output boundary,
documented in [completed-job log retrieval](gh-completed-job-logs.md).

## Upstream filing decision

The repository's upstream-scope inventory contains no GitHub CLI exemption.
No new upstream defect is established:
the deciding transport source and tests intentionally sanitize JSON controls,
and the repository's exact-byte consumer has an exercised API-envelope workaround.

1.   Fault attribution:
     the observed digest mismatch belongs to choosing a display-sanitizing JSON transport for exact-byte evidence.
2.   Fixability:
     object-response decoding resolves the repository consumer boundary without changing GitHub CLI.
3.   Supported scope:
     the Contents API documents and returned the base64 file content used by the control.
4.   Contribution policy:
     no upstream patch is proposed,
     so submission eligibility was not established.
5.   Maintainer direction:
     the explicit transport implementation and tests establish the current behavior;
     they do not establish willingness to change it.
6.   Prototype:
     actual authenticated retrieval and digest rejection controls exercise the consumer workaround.

There is nothing to add upstream from this investigation.
The repository tracker records adoption of byte-preserving retrieval as its own harness work.

[contents-api]: https://docs.github.com/en/rest/repos/contents
