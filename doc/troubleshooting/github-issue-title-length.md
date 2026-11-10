# GitHub REST API 2026-03-10 omits the Issue title length limit needed for prevalidation

## Symptom

A client generating GitHub Issue titles needs to know which titles the Create an issue endpoint accepts.
GitHub's current REST documentation and OpenAPI description require `title`,
but publish no minimum length,
maximum length,
or counting unit.
A client therefore cannot derive a provider-guaranteed truncation rule from the published REST contract.

A community report shows the GitHub interface message
`Title can not be longer than 256 characters`.
That report is useful operational evidence,
but it does not define whether the REST API uses the same limit
or whether “characters” means bytes,
Unicode code points,
or grapheme clusters.

## Root cause

### The endpoint documentation gives `title` no constraints

The [Create an issue documentation][github-create-issue]
describes the body parameter only as a required string or integer:

```text
`title` string or integer Required

The title of the issue.
```

Its documented `422` response combines validation failure and spam detection.
It does not enumerate title validation errors.

### The current OpenAPI schema also omits length metadata

The source audit used `github/rest-api-description` commit
`67c14c7efb01cdeeac0ecd8cee9fae8d7a80e2aa`.
[`descriptions/api.github.com/api.github.com.2026-03-10.yaml:46459-46471`][openapi-source]
defines `title` without `minLength` or `maxLength`:

```yaml
requestBody:
  required: true
  content:
    application/json:
      schema:
        type: object
        properties:
          title:
            oneOf:
            - type: string
            - type: integer
            description: The title of the issue.
```

The same operation lists `title` as required,
but supplies no additional reusable schema reference or constraint.
The omission is in the published provider description,
not in a generated TypeScript client.

### Community evidence is not a provider contract

A public Issue titled
[`Title can not be longer than 256 characters`][community-256]
includes a screenshot of that GitHub interface error.
Other projects have independently truncated generated GitHub titles,
but those observations do not establish the REST API's current validation unit or exact inclusive boundary.

The earlier assumption that the current REST schema documented a 256-character limit was wrong.
The current documentation and OpenAPI source contain no such bound.

## Verification

### Versions and sources

- GitHub REST API documentation:
  version `2026-03-10 (latest)`,
  accessed 2026-08-16.
- `github/rest-api-description`:
  commit `67c14c7efb01cdeeac0ecd8cee9fae8d7a80e2aa`.
- Audited operation:
  `issues/create` in `descriptions/api.github.com/api.github.com.2026-03-10.yaml`.

The source check is reproducible after cloning the description repository:

```bash
gh repo clone github/rest-api-description "$HOME/temp/agent/github-rest-api-description"
rg --line-number --context 12 'operationId: issues/create' \
  "$HOME/temp/agent/github-rest-api-description/descriptions/api.github.com/api.github.com.2026-03-10.yaml"
```

No live Issue was created for this investigation.
A boundary probe would mutate a real repository,
and no disposable GitHub repository was authorized for that test.
The investigation therefore verifies the documentation gap,
not the service's undocumented runtime limit.

### Behavior documented cleanly

- `title` is required for Issue creation.
- The request accepts a string or integer title.
- A successful creation returns `201`.
- Validation failures may return `422`.

### Behavior missing from the contract

- Minimum accepted string length.
- Maximum accepted string length.
- Whether a maximum is inclusive.
- Whether length counts bytes,
  Unicode code points,
  UTF-16 code units,
  or grapheme clusters.
- Whether the REST API and web interface share one title validator.

## Verified workarounds

### Define an adapter-owned title contract

A consumer can choose its own deterministic non-empty fallback and conservative cap,
truncate locally,
and retain the complete finding in the Issue body.
This makes previews and create requests identical
and avoids depending on an undocumented server-side truncation behavior.

The OCR adapter selected a 256-byte UTF-8 cap on the complete final title.
It leaves titles within that cap unchanged.
For an overlength title,
it retains the longest valid UTF-8 prefix fitting within 253 bytes,
trims trailing whitespace,
and appends the three-byte `…` character.
The cap includes the `[needs-triage] ` fallback prefix when present.
Boundary tests must cover ASCII,
multibyte code points,
whitespace trimming,
and exact-limit inputs.
The package must not describe this local cap as GitHub's documented limit.

Tradeoffs:

- A local cap can discard title detail,
  though the complete OCR content remains in the body.
- A conservative cap can be lower than GitHub's actual acceptance boundary.
- GitHub can still reject a title for an undocumented rule,
  so `422` remains a terminal validation response rather than a retryable failure.

### Reject titles outside the adapter contract

A consumer can reject overlong generated titles before publication instead of truncating them.
This preserves exact titles and avoids an ambiguous shortening algorithm.

Tradeoffs:

- One long path or summary rejects the complete validated input before GitHub operations.
- Users must alter upstream OCR output or adapter policy before retrying.

## What does not work

### Claiming that the REST documentation guarantees 256 characters

Neither the endpoint page nor its OpenAPI request schema publishes that number.
The community interface report cannot replace a provider contract.

### Waiting for GitHub to truncate the title

The endpoint documents validation failure,
not server-side title truncation.
A create request must not rely on an undocumented transformation.

### Inferring units from JavaScript string length

JavaScript string length counts UTF-16 code units.
Without a provider-defined unit,
that measurement cannot be labeled as GitHub's title-length rule.

## Upstream filing decision

The repository's issue search found no open or closed report matching
`issue title maxLength` or `256 characters`.

1.  **Is it really upstream's fault?**
    Not yet proven.
    The schema omits a limit,
    but no authorized disposable runtime probe established that the REST endpoint enforces one.
2.  **Can upstream fix it?**
    Yes,
    if runtime behavior has a stable bound,
    GitHub can publish matching OpenAPI length metadata and units.
3.  **Are they supporting this use case?**
    Yes.
    The description publishes the `issues/create` request schema and accepts schema-inaccuracy reports.
4.  **Would the repo welcome our contribution?**
    [`CONTRIBUTING.md`][contributing]
    directs description mismatches to Issues rather than pull requests,
    and the repository provides a Schema Inaccuracy template.
    No AI-assistance prohibition was found in the contribution guide,
    issue template,
    or sampled recent Schema Inaccuracy discussions.
5.  **Will they likely fix it?**
    No negative signal was found.
    Recent completed schema reports show follow-up,
    but that does not predict action on an unverified limit.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No.
    The source descriptions are generated artifacts that reject direct pull requests,
    and the required runtime boundary has not been reproduced.

Constraints one and six fail.
There is no upstream filing or comment to make from the current evidence.
A future filing requires an authorized disposable API reproduction
that identifies the accepted boundary and counting unit.

[community-256]: https://github.com/nelsontky/gh-pages-url-shortener/issues/125
[contributing]: https://github.com/github/rest-api-description/commit/67c14c7efb01cdeeac0ecd8cee9fae8d7a80e2aa
[github-create-issue]: https://docs.github.com/en/rest/issues/issues#create-an-issue
[openapi-source]: https://github.com/github/rest-api-description/commit/67c14c7efb01cdeeac0ecd8cee9fae8d7a80e2aa
