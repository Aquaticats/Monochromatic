# OpenAI Python 3.11.0 includes retry-count request metadata

## Symptom

A proposed Promise-lesson correction says that a real request carries no attempt counter.
That statement is broader than the local fixture evidence supports.
The distinction needed by the lesson is between fixture-controlled outcomes and an external API's actual contract,
not a prohibition on transmitting retry metadata.

## Source trace

Read-only source checkout:
`/var/home/user/temp/agent/promises-review-assessment.qUhUdO/openai-python-2026-09-08`.
Commit:
`2d4b97cc84d5c3ca051cc2ac2d8a6c9928b4d5f0`.
`src/openai/_version.py:2` identifies version `3.11.0`.
Upstream:
[OpenAI Python source][source].

`src/openai/_base_client.py:473` defines `BaseClient._build_headers` with a `retries_taken` input.
At lines 487 to 489,
the SDK supplies the retry-count header unless the caller overrides or omits it:

```python
# src/openai/_base_client.py:487
lower_custom_headers = [header.lower() for header in custom_headers]
if "x-stainless-retry-count" not in lower_custom_headers:
    headers["x-stainless-retry-count"] = str(retries_taken)
```

`src/openai/_base_client.py:540` connects this header builder to request construction:

```python
# src/openai/_base_client.py:540
headers = self._build_headers(options, retries_taken=retries_taken)
```

This is counterexample evidence for the universal claim.
It does not establish that a server uses the header to decide whether an attempt succeeds.

## Verification and limits

The source was retrieved with `gh repo clone openai/openai-python` and `--depth 1`.
The checkout's commit,
version,
header builder,
and existing test cases were inspected.
No SDK dependencies were installed,
no SDK tests were executed,
and no API request was sent.

Source-supported cases:

- Default metadata:
  `tests/test_client.py:1228` asserts that
  `response.http_request.headers.get("x-stainless-retry-count")` equals the retries taken.
- Omission:
  `tests/test_client.py:1257` supplies `Omit()`;
  line 1260 asserts the outgoing header list is empty.
- Override:
  `tests/test_client.py:1289` supplies `"42"`;
  line 1292 asserts that exact outgoing header value.

Rejected interpretation:

- Inferring that every real request excludes an attempt counter.
- Inferring that the server promises success after a client-declared number of attempts.

## Correction at the teaching boundary

Explain that `attempt` selects repeatable outcomes in this local fixture,
while the retry loop's counter belongs to the application's policy.
A real integration follows its own API contract and may transmit retry metadata.
No runtime workaround or upstream change is needed for this source finding.
No lesson wording or implementation was changed during this review.

## Upstream filing decision

Nothing to file:
the problem is an overbroad proposed teaching statement,
not an SDK failure.

1.  Upstream fault:
    no;
    the source intentionally adds metadata and has corresponding tests.
2.  Fixability:
    not applicable;
    no SDK change is requested.
3.  Supported use:
    the named source tests cover default,
    omitted,
    and overridden metadata.
4.  Contribution welcome:
    not investigated because no contribution is proposed.
5.  Likelihood of an upstream fix:
    not applicable because there is no upstream defect to fix.
6.  Prototype:
    not applicable;
    this is a source-backed correction to a local review recommendation.

No issue/comment draft or external communication was produced.
An upstream duplicate search is unnecessary for a filing that is not proposed.

[source]:
  https://github.com/openai/openai-python/blob/2d4b97cc84d5c3ca051cc2ac2d8a6c9928b4d5f0/src/openai/_base_client.py
