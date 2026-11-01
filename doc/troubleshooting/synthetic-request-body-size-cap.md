# Synthetic request body size cap

Fact base for a support report to [Synthetic],
against `https://api.synthetic.new/openai/v1/chat/completions`.
We are grateful for the service;
this document exists because one failure mode is reported as a parse error
when it appears to be a size limit,
and because the limit is not documented.

Everything attributed below is stated with its source,
because the size observations were reported to this repository rather than measured in it.
What this repository can measure about its own traffic is measured and marked as such.

## Summary

A request whose body exceeds roughly 10 MiB returns `400`
with a message naming a JSON parse failure:

```text
Could not parse request as valid JSON. Unterminated string in JSON at position 1044xxxx
```

The body sent is valid,
complete JSON.
The reporter verified this by capturing the body locally and replaying it with plain `curl`.

- A body of exactly 10485760 bytes succeeds.
- A body of roughly 10.2 MiB fails.
- The same cutoff appears when the upload is throttled to 1 MB/s,
  so the boundary is a byte count rather than a timeout.

The reading that fits those observations is that the gateway truncates the body near 10 MiB
and then parses the truncated text,
which fails inside whatever string the cut landed in.
The reported position,
around 1044xxxx,
sits just under the 10 MiB boundary,
which is consistent with that reading.

## What is asked of the provider

- Return `413` for a body over the limit,
  rather than a `400` naming a parse failure in a body that parses.
  The current message sends a caller to look for malformed JSON that is not there.
- Document the limit.
  The chat completions documentation encourages base64 images
  and names no request size cap.

## Why the misleading error costs more than the limit does

The reporter hit this from an agent workflow that attaches images.
Once the accumulated history passes the cap,
every subsequent request fails,
and the session cannot recover on its own,
because a caller reading the error has no reason to suspect size.
A `413` would let a client drop or downscale attachments and continue.

## What this repository sends today, measured

Measured on 2026-08-21 against the pinned corpus and the shipped reading stage.

- `package/translation-repair/slice-reading/src/image-reading-stage.ts` sends ONE picture per request,
  as a `text` part followed by a single `image_url` part carrying a base64 data URI.
  There is no accumulating history,
  so the growth mode the report describes does not apply here.
- `READING_MAX_BYTES` in that file is 8388608 raw bytes.
  MEASURED 2026-08-22 by serializing the exact body the stage builds,
  with `~/temp/agent/measure-168-body.mjs`:
  at that ceiling the body is 11185335 bytes,
  which is 699575 above the only size known to pass.
  So the refusal threshold this repository configures permits a request the gateway would reject,
  and the rejection would arrive as the parse error rather than as a refusal naming size.
- The overhead around the picture is a constant 501 bytes,
  the 342 byte instruction plus the JSON envelope,
  so a raw ceiling maps onto a body size exactly and monotonically.
  The largest raw picture whose body still fits is 7863927 bytes,
  for a body of 10485759 and one byte of headroom.
  7863930 gives 10485763 and does not fit,
  so that boundary is bisected rather than estimated.
- Nothing in the pinned corpus comes close.
  The largest asset is 1344454 bytes of 291 assets,
  whose body measures 1793131 bytes,
  leaving 8692629 under the passing size.

The exposure is therefore the configured ceiling rather than current traffic.

## The ceiling this repository will configure

DECIDED 2026-08-22 on the body measurement recorded in this document,
made with `~/temp/agent/measure-168-body.mjs`.
It is recorded rather than asked because nothing is traded away by it:
the new ceiling is still more than five times the largest asset in the corpus,
so no picture that is read today would be refused.

`READING_MAX_BYTES` becomes 7340032,
seven mebibytes.
Its body measures 9787235 bytes,
which leaves 698525 under the passing size.

NOT the exact fit of 7863927.
Only the passing size is exact.
The failing size is reported as approximate and the true boundary between them has never been bisected,
so a ceiling with one byte of headroom rests on an assumption rather than on a measurement.
Seven mebibytes also absorbs growth in the instruction text,
which is part of the constant.

LANDED 2026-08-22,
once the source freeze recorded in `doc/handover/translation-repair-run-continuity.md` lifted:

- The constant lowered to 7340032 in `package/translation-repair/slice-reading/src/image-reading-stage.ts`,
  with its documentation rewritten around the gateway rather than the model.
- The refusal reason renamed from `too-large-for-model` to `too-large-for-transport`,
  in both unions that declare it
  (`image-asset.ts` and `image-reading-stage.ts`)
  and in both tests that assert it.
  No settled artifact carries the old spelling,
  checked across `~/temp/agent/*.json` and both run directories,
  and refusals are re-asked rather than cached under `#123`,
  so no persisted state holds it either.
  The two mentions in `doc/audit/reading-a-picture-at-the-user-boundary.md` describe historical runs
  and keep the old spelling deliberately.

- The gateway's parse failure re-raised as a size refusal,
  in `package/translation-repair/provider/src/request-size-refusal.ts`.
  `failureForReply` builds `SyntheticRequestTooLargeError`,
  a subclass of `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
  carrying the measured body size beside the passing size,
  and `synthetic-client.ts` routes its streaming-completions failure through it.

Nothing is owed on this document now.

## How the re-raise decides

FOUR SIGNALS,
ANDed.
Status 400,
a body message beginning `Could not parse request as valid JSON`,
a request body measured over 10485760 bytes,
and a reply from Synthetic.
Any one of them missing leaves the plain `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) untouched.

The provider joined the conjunction on 2026-10-19.
Until then `failureForReply` read every provider's reply this way,
since the Charm Hyper,
OpenRouter and Bedrock clients route their failures through it too,
and a reply from any of them with the other three signals would have claimed Synthetic's cap.
Every measurement here is Synthetic's;
whether the other gateways cap a body at any size,
and in what words they refuse it,
is unmeasured.

The conjunction protects both directions.
A small body drawing that message is a body we genuinely malformed,
and reporting it as too large would send whoever chases it hunting for a limit they never hit.
An oversize body refused for a bad model name is refused for the model name.

BYTES,
NOT CHARACTERS.
This corpus is Chinese,
where one character costs three bytes in UTF-8,
so `bodyJson.length` reads about a third of the wire size
and would never fire on the oversize-prompt case the re-raise exists for.
`Buffer.byteLength` is what the client measures,
and `synthetic-client.unit.test.ts` pins it with a request whose character count is under the cap
while its byte count is over.

AFTER THE FACT,
NEVER BEFORE IT.
Nothing refuses a request client-side at the passing size.
Only that size is exact;
the failing size is approximate and the boundary between them has never been bisected,
so a pre-flight guard at 10485760 would reject bodies in the gray zone above it
that the gateway may well carry.
Reading an answer that already arrived cannot cost a call that would have worked.

NOT CAUGHT ANYWHERE,
deliberately.
The reading stage does not fold this into an `unavailable` reading.
After the ceiling drop the picture path cannot produce an oversize body,
9787235 against 10485760 by measurement,
so a catch there would absorb a state that should be impossible.
A body pushed over by something other than a picture is a defect to see,
not to recover from.

## Each guard shown to fail

Measured 2026-08-22.
Every break below is type-legal,
so only a test can catch it,
and each was applied alone and reverted before the next.
Every one failed exactly the case it targets and nothing else.

- The ceiling restored to 8388608:
  fails `REFUSES AT THE CEILING THE MEASUREMENT DECIDED`,
  which reads `expected 'read' to equal 'unavailable'`.
  The 8000000 byte picture was sent and read.
- `Buffer.byteLength(bodyJson)` replaced with `bodyJson.length`:
  fails `MEASURES THE BODY IN BYTES, NOT CHARACTERS`.
  The Chinese request measures 3600000 characters and over 10800000 bytes,
  so the character reading calls it small and no size failure is raised.
- The size signal always true:
  fails `STAYS A PLAIN FAILURE FOR A SMALL BODY CARRYING THE SAME MESSAGE`
  and `LEAVES A SMALL MALFORMED BODY AS AN ORDINARY FAILURE`,
  one in each test file.
- The message signal always true:
  fails `STAYS A PLAIN FAILURE FOR AN OVERSIZE BODY REFUSED FOR SOMETHING ELSE`.
- The status signal always true:
  fails `STAYS A PLAIN FAILURE ON ANY OTHER STATUS`.

Suite after every break was reverted:
517 groups passing,
none failing,
lint clean.

## What is not ruled out

- Only the passing size,
  10485760 bytes,
  is exact.
  The failing size is reported as approximate.
  The true boundary between them has not been bisected.
- Whether the cap belongs to the gateway or to a proxy in front of it.
- Whether streaming and non-streaming requests share the boundary.
- Whether other endpoints share it.
- Whether the limit counts bytes on the wire or bytes after any transfer encoding.

Repro scripts and captures were offered by the reporter and can be shared if useful.
