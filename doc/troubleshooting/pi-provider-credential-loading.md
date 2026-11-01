# Pi provider probe credential loading

## Ciphertext mistaken for an API key

On October 14,
2026,
private Liquid and SubQ probes read `AUTO_MODE_LIQUID_API_KEY`
and `AUTO_MODE_SUBQ_API_KEY` directly from `.env.local.json`.
That file contains SOPS-encrypted values.
Liquid's documented Decisions endpoint returned HTTP 401 `invalid_api_key`;
SubQ's Decisions endpoint returned HTTP 403 `permission_error`.
Explicit rechecks reproduced both diagnostics with the same mistaken credential source.
SubQ's models endpoint returned HTTP 200 with the ciphertext,
so that response did not prove valid authentication.

The user questioned whether mise was involved.
`mise.toml` loads `.env.local.json` through `env._.file` with redaction enabled.
Its comments identify SOPS decryption through mise's built-in rops.
The [current mise documentation](https://mise.jdx.dev/environments/secrets/sops.html)
describes that encrypted-file support.
The installed `mise env --help` confirms that `--json` emits resolved values,
including secrets;
`--redacted` filters variables but does not mask their values.

Capture `mise env --json` in process memory from the repository directory.
Pass only the required decrypted key to the isolated probe process.
Never print or persist that JSON,
and avoid exception objects that include captured subprocess output.
Reject missing,
empty,
or `ENC[`-prefixed values before dispatch.
Only a real request with the resolved key verifies account access.

The initial in-memory capture passed presence and non-ciphertext checks for both keys.
Liquid's resolved key has its documented `liquid_` prefix.
No key or account mutation was needed to obtain these resolved values.
Both resolved-key controls returned HTTP 200 with their requested models and complete answer maps.
Liquid answered one question in 591.486 ms;
SubQ answered 16 in 845.179 ms.
The matching ciphertext controls had returned 401 and 403 respectively.
A synthetic `ENC[synthetic]` credential now fails the guard before dispatch and before `started.json` exists.
The private probes retain those checks under
`${HOME}/temp/agent/pi-liquid-access-resolved-20261014`
and `${HOME}/temp/agent/pi-subq-boundary-resolved-20261014`.

## Upstream filing decision

No upstream bug is established:
the caller bypassed configured credential resolution.
There is no upstream defect to fix or prototype.
Mise documents the encrypted-file use case,
and the corrected consumer invocation works.
Contribution-policy and maintainer-interest checks are inapplicable without an upstream issue.
No external report or draft is warranted.

## Proposed guidance tightening

The existing `AGENTS.md` rule `QF1` requires measuring facts and checking configuration before categorical claims.
The proposed replacement makes the configured resolver explicit:

> Measure facts and cite results.
> Resolve configuration through its owning tool;
> reject ciphertext as credentials.
> Verify categorical claims with a search or configuration read.

The proposal reuses the existing rule's scope and identifier.
The user chose to keep `QF1` unchanged on October 14,
2026.
The proposed rule change is rejected;
the verified resolver and ciphertext checks remain in this troubleshooting document.
