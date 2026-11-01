# CloudFront origin hostname function

## Purpose

`amazon.aquati.cat` must reach the primary origin as `aquati.cat`
for both HTTP Host and TLS SNI.
The existing cache and origin-request policies forward the viewer hostname.
The live Free flat-rate plan rejects the otherwise-equivalent custom cache policy that removes `host`.

The CloudFront runtime 2.0 function uses the native origin-override helper instead.
It leaves the viewer request and both policies unchanged.
It does not change DNS,
Caddy,
TLS protocols,
the WAF association,
or pricing-plan subscription.

## Canonical uploaded source

This fence contains the exact standalone JavaScript artifact uploaded to CloudFront,
including its final newline.
CloudFront supplies the `cloudfront` module and invokes the global `handler`;
this is not a Node module or a site-build input.

```js
import cf from 'cloudfront';

/**
 * Keep the viewer request unchanged while selecting the primary origin's TLS identity.
 * CloudFront JavaScript runtime 2.0 provides the global handler entry point and cf module.
 * @param event - CloudFront viewer-request event.
 * @returns Original request, preserving its cache-key inputs.
 * @example handler({ request: request });
 */
function handler(event) {
  cf.updateRequestOrigin({ hostHeader: 'aquati.cat', sni: 'aquati.cat' });
  return event.request;
}
```

Source SHA-256:
`3e437fbfe726d449e57b7ecb70a016b3848d88929d38f01196de5fc1482d4f02`.

Function:
`AquatiCat-OriginHost`.
ARN:
`arn:aws:cloudfront::<account-id>:function/AquatiCat-OriginHost`.
Redacted on 2026-10-09 because this repository is public.
Recover `<account-id>` with `aws sts get-caller-identity --query Account`.
Runtime:
`cloudfront-js-2.0`.
Event:
`viewer-request`.

## Verification and limitations

AWS `TestFunction` ran GET,
HEAD,
OPTIONS,
POST,
PUT,
PATCH,
and DELETE as virtual inputs,
without issuing mutating HTTP methods to the origin.
The returned request preserved method,
URI,
query values,
headers,
and cookies.
AWS wraps the request in `FunctionOutput` as `{"request": ...}`;
the first local assertion was corrected to inspect that observed output envelope.
No function-source change was required.

The Test API does not verify origin overrides.
Actual requests through a disposable distribution with the original policies demonstrated
HTTP 502 without the function and matching origin content with it.
The same published bytes were then deployed to `<distribution-id>` at `aws.aquati.cat`.
That hostname moved to `amazon.aquati.cat` on 2026-10-09;
 this sentence keeps the name
the deployment actually used.
Redacted on 2026-10-09;
 recover it with `aws cloudfront list-distributions`.
Live verification passed at `2026-09-07T21:42:12Z`;
deployed distribution ETag:
`<etag>`.
The full config comparison showed only the viewer-request function association changed.
Both policies and the active Free subscription remain unchanged.
The disposable distribution and unused custom cache policy are deleted.

Validation covered route/asset/status/body/header parity,
HEAD,
actual gzip/Brotli decoding,
browser navigation,
resource loading,
and theme toggling.
The primary still rejects TLS 1.2 over IPv4 and IPv6.
Search interaction was not established in either origin or fixture browser controls.
Repeated CSS probes were cache misses with no origin `Cache-Control`,
so cache-hit behavior is not claimed.

Deployment results and rollback state are tracked in
[`doc/handover/aws-cloudfront-mirror-correction.md`](../../../../doc/handover/aws-cloudfront-mirror-correction.md).

This function runs on cache hits as well as misses.
A function error can therefore prevent all requests from succeeding.
Its literal origin identity is appropriate only while this behavior serves `aquati.cat`;
revisit it before adding origins or changing the primary hostname.
No viewer-controlled value is interpolated into an origin setting.

## Rollback

Remove this viewer-request association using a fresh distribution ETag,
while retaining all other current fields.
Both original policy IDs stay unchanged during this function-based correction:

- Cache:
  `4cc15a8a-d715-48a4-82b8-cc0b614638fe`.
- Origin request:
  `216adef6-5c7f-47e4-b989-5492eafa07d3`.

The saved pre-correction configuration remains the comparison reference,
not an instruction to overwrite future unrelated changes.

## Sources

- [Native origin override helper][helper],
  including the Test API limitation,
  inheritance of omitted origin settings,
  and `hostHeader`/`sni` semantics.
- [Flat-rate plans][plans],
  including Free-plan CloudFront Functions and custom-cache-policy restrictions.

[helper]: https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/helper-functions-origin-modification.md
[plans]: https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/flat-rate-pricing-plan.md
