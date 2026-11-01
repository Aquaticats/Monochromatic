# GitHub API 2026-03-10 requires mutation pacing and gh 2.102.0 comments can remain throttled after backoff

## Symptom

A bounded queue can make concurrent GitHub Issue creation look provider-safe.
The proposed adapter initially considered `p-limit` 7.3.1 with concurrency five.
That schedule conflicts with GitHub's current REST API guidance before any request fails:
GitHub recommends serial requests and a pause of at least one second between mutative requests.

This is a design-time incompatibility,
not a reproduced GitHub outage or an upstream `p-limit` defect.

## Root cause

### GitHub treats concurrency and mutation pacing as separate secondary-rate-limit risks

[GitHub's REST API best-practices document][github-best-practices]
says under “Avoid concurrent requests”:

> To avoid exceeding secondary rate limits,
> you should make requests serially instead of concurrently.
> To achieve this,
> you can implement a queue system for requests.

The adjacent “Pause between mutative requests” section adds:

> If you are making a large number of `POST`,
> `PATCH`,
> `PUT`,
> or `DELETE` requests,
> wait at least one second between each request.

Issue creation is a `POST` operation.
A concurrency cap controls how many requests run together,
but it does not make those requests serial or impose time between their starts.

GitHub's “Handle rate limit errors appropriately” section also requires clients
to honor `retry-after`,
wait until `x-ratelimit-reset` when the remaining count is zero,
or wait at least one minute and increase later delays for a secondary limit without either header.
Continuing while rate-limited can result in integration banning.

### `p-limit` starts work immediately whenever capacity exists

The source audit used `p-limit` tag `v7.3.1`,
commit `df476048d023ff868cd45b35ee47f5fb0ca2b25a`.
[`index.js:18-24`][p-limit-resume] starts another queued function whenever active work is below the cap:

```js
const resumeNext = () => {
	// Process the next queued function if we're under the concurrency limit
	if (activeCount < concurrency && queue.size > 0) {
		activeCount++;
		queue.dequeue().run();
	}
};
```

[`index.js:59-63`][p-limit-immediate] invokes that scheduler immediately after enqueueing:

```js
// Start processing immediately if we haven't reached the concurrency limit
if (activeCount < concurrency) {
	resumeNext();
}
```

There is no time-based pacing in this path.
`pLimit(5)` can therefore start five Issue requests without the provider-documented pause.

### Queue clearing cannot revoke active Issue requests

[`index.d.ts:17-27`][p-limit-clear-type] documents the cancellation boundary:

```ts
/**
Discard pending promises that are waiting to run.

Note: This does not cancel promises that are already running.

When `rejectOnClear` is enabled, pending promises are rejected with an `AbortError`.
*/
clearQueue: () => void;
```

[`index.js:77-89`][p-limit-clear-source] confirms that `clearQueue()` touches queued entries only.
A failure observed under concurrency five could leave other Issue creations already in flight.
Clearing pending work would not provide strict stop-at-first-failure behavior.

### Issue creation has no documented idempotency mechanism

[GitHub's best-practices document][github-best-practices]
says conditional requests for unsafe methods such as `POST` are unsupported
unless an endpoint documents otherwise.
The [Create an issue endpoint][github-create-issue] documents its request fields and responses
but no idempotency key or conditional-create contract.

A network failure,
timeout,
or `5xx` response can therefore leave the client unable to prove whether the Issue was created.
Retrying that request can create a duplicate.
The adapter design retries ambiguous transient failures only after repository-number reconciliation.
Before each initial request,
it captures the greatest Issue or pull request number.
After an ambiguous failure,
it queries newer numbers and compares exact generated title and body.
A match suppresses the retry;
a failed reconciliation query stops processing.

GitHub's current Issue documentation does not state a read-after-write consistency guarantee.
A successful query that finds no match therefore reduces duplicate risk.
It cannot prove that a retry is duplicate-safe.
Help,
preview,
and result diagnostics must disclose that residual risk.

### `gh api` leaves explicit retry orchestration to its caller

The selected adapter delegates authentication and HTTP to GitHub CLI 2.97.0,
but retains its own retry policy.
[`pkg/cmd/api/api.go:369-381`][gh-api-input]
opens the named `--input` file and supplies it as the request body:

```go
if opts.RequestInputFile != "" {
	file, size, err := openUserFile(opts.RequestInputFile, opts.IO.In)
	if err != nil {
		return err
	}
	defer file.Close()
	requestPath = addQuery(requestPath, params)
	requestBody = file
```

The adapter can therefore avoid sending issue bodies through its standard input or process arguments.
[`pkg/cmd/api/api.go:474-478`][gh-api-headers]
prints response status and headers before processing the body when `--include` is set:

```go
if opts.ShowResponseHeaders {
	fmt.Fprintln(headersWriter, resp.Proto, resp.Status)
	printHeaders(headersWriter, resp.Header, opts.IO.ColorEnabled())
	fmt.Fprint(headersWriter, "\r\n")
}
```

For each non-paginated command invocation,
[`pkg/cmd/api/http.go:83`][gh-api-do]
performs the request through one client call:

```go
return client.Do(req)
```

The `cli/go-gh` 2.13.0 client begins with `http.DefaultTransport`
and then wraps it for sanitization,
optional caching,
logging,
and headers.
[`pkg/api/http_client.go:59-69`][go-gh-transport]
shows no retry transport in that chain:

```go
transport := http.DefaultTransport

if opts.UnixDomainSocket != "" {
	transport = newUnixDomainSocketRoundTripper(opts.UnixDomainSocket)
}

if opts.Transport != nil {
	transport = opts.Transport
}

transport = newSanitizerRoundTripper(transport)
```

This establishes that `gh api` has no application-level retry loop on this path.
The adapter can classify the returned status and headers,
reconcile an ambiguous creation result,
and decide whether to start another command.
It must not claim control over undocumented lower-level transport behavior.

GitHub CLI's command options also supply no timeout to that client.
[`pkg/cmd/api/api.go:390-405`][gh-api-client]
constructs `HTTPClientOptions` without one,
while [`cli/go-gh/pkg/api/client_options.go:59-61`][go-gh-timeout]
documents the resulting default:

```go
// Timeout specifies a time limit for each API request.
// Default is no timeout.
Timeout time.Duration
```

GitHub CLI 2.97.0 exposes no `gh api` timeout flag.
The adapter must therefore bound the subprocess lifetime itself
if it promises a request timeout or a finite graceful-interrupt wait.

## Verification

### Versions and sources

- GitHub REST API documentation version:
  `2026-03-10 (latest)`,
  accessed 2026-08-16.
- `p-limit` package:
  tag `v7.3.1`,
  commit `df476048d023ff868cd45b35ee47f5fb0ca2b25a`,
  MIT license.
- Upstream clone:
  `~/temp/agent/p-limit-7.3.1-20260816`.
- Workspace catalog:
  `pnpm-workspace.yaml` already contains `p-limit: '>=7.3.1'`.
- GitHub CLI:
  tag `v2.97.0`,
  commit `55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55`.
- `cli/go-gh` used by GitHub CLI:
  tag `v2.13.0`,
  commit `a0a6e8947ae2ceedb496654757886ef41ef5ac72`.

No live GitHub Issue was created for this investigation.
The deciding GitHub behavior is an explicit provider instruction,
and exercising the rejected schedule would mutate external state without proving that GitHub permits it.
The `p-limit` scheduler and cancellation boundaries were verified from source and type declarations.

A read-only probe of `GET /repos/Aquaticat/Monochromatic/issues`
with `state=all`,
`sort=created`,
`direction=desc`,
and `per_page=5` returned descending integer numbers and included a pull request entry.
This verifies the high-water query surface against the destination repository.
It does not establish an undocumented read-after-write guarantee.

A local source check is reproducible with:

```bash
rg --line-number \
  'resumeNext|Start processing immediately|clearQueue|does not cancel' \
  "$HOME/temp/agent/p-limit-7.3.1-20260816/index.js" \
  "$HOME/temp/agent/p-limit-7.3.1-20260816/index.d.ts"
```

### Provider-aligned behavior catalog

- Dispatch one mutative GitHub request at a time.
- Leave at least one second between mutative requests.
- Honor `retry-after` before another request.
- Wait for `x-ratelimit-reset` when `x-ratelimit-remaining` is zero.
- Stop after a bounded number of exponentially delayed secondary-rate-limit retries.
- Treat ambiguous create retries as carrying duplicate-Issue risk.

### Conflicting behavior catalog

- Start five Issue-creation requests together.
- Start another mutation immediately when one concurrency slot opens.
- Treat a concurrency cap as request pacing.
- Assume `clearQueue()` cancels Issue requests that already started.

## Verified workarounds

### Serialize Issue creation and pace request starts

The selected adapter design follows both provider instructions:

1. Create one Issue at a time.
2. Wait at least one second before starting the next Issue-creation request.
3. Do not add `p-limit` to the adapter package for Issue creation.
4. Permit at most three retries after an initial retryable failure.
5. Honor GitHub rate-limit headers;
   otherwise use exponential delays starting at sixty seconds for rate limits
   and one second for network,
   timeout,
   or `5xx` failures.
6. Reconcile ambiguous failures against Issue or pull request numbers above a pre-request high-water mark.
7. Stop instead of retrying if the reconciliation query fails.
8. Stop scheduling new Issues after a retry-exhausted failure.
9. Invoke each REST operation through non-paginated `gh api --include`.
10. Pass each JSON request body through a private named file with `--input`,
    never through standard input or process arguments.
11. Parse returned status,
    headers,
    and JSON before deciding whether another `gh api` invocation is allowed.
12. Apply a fixed one-minute adapter-owned subprocess deadline
    because `gh api` 2.97.0 has no request timeout by default.

This is verified against GitHub's published concurrency and mutation-pacing contract
and the GitHub CLI 2.97.0 request path.
Runtime verification remains part of adapter implementation because no production adapter exists yet.

Tradeoffs:

- Publication takes longer than a concurrent queue.
- Strict serialization makes stop-at-first-terminal-failure exact for requests not yet started.
- Provider-compliant pacing reduces secondary-rate-limit exposure but cannot eliminate network or service failures.
- Retrying ambiguous Issue-creation failures can create duplicates.

## What does not work

### `pLimit(5)` without pacing

It bounds simultaneous work but immediately fills available slots.
It satisfies neither serial dispatch nor the one-second mutation pause.

### `pLimit(5)` with one-second start spacing

Spacing request starts follows the mutation-pause guidance,
but slow requests can still overlap.
It therefore continues to conflict with GitHub's separate serial-request recommendation.

### Clearing the queue after a failure

`clearQueue()` discards pending functions only.
It cannot cancel requests already running,
so concurrency five cannot promise that no later Issue is created after the first terminal failure.

### Assuming a retry is duplicate-safe

The Issue-creation endpoint has no documented idempotency key.
An ambiguous failed response is not proof that GitHub created nothing.
A no-match reconciliation query is also not documented as a consistency guarantee.
The adapter may retry by explicit design choice,
but it must not claim that the retry is duplicate-safe.

## Upstream filing decision

No `.out-of-scope/` entry covers GitHub REST mutation concurrency or `p-limit` queue clearing.
GitHub documentation issues and community discussions were searched for:

- `Pause between mutative requests concurrent requests REST`;
- `secondary rate limit one second concurrent Issue creation`;
- `p-limit clearQueue does not cancel running promises`.

No upstream defect matching this design conflict was found.
The `p-limit` tracker contains historical queue-clearing discussions,
but current 7.3.1 types already state the active-task boundary.

The filing constraints resolve as follows:

1. **Is it really upstream's fault?**
   No.
   GitHub documents the accepted request schedule,
   and `p-limit` documents that it limits concurrency rather than pacing or canceling active tasks.
2. **Can upstream fix it?**
   Not applicable as a defect.
   The consumer must choose a schedule compatible with the destination API.
3. **Are they supporting this use case?**
   GitHub supports Issue creation and publishes the relevant rate-limit practices.
   `p-limit` supports bounded concurrency,
   not provider-specific pacing.
4. **Would the repo welcome our contribution?**
   Not evaluated because there is no missing or incorrect upstream contract to change.
5. **Will they likely fix it?**
   Not applicable;
   the observed behavior is documented and intentional.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Not applicable because constraints one and two fail.
   The selected fix is consumer-side serialization.

There is nothing additive to file or comment upstream.

## Observed GitHub CLI 2.102.0 comment rejection on 2026-10-17

### Symptom and boundary

The authorized Depot trial reconciliation attempted to add a reviewed comment to repository issue `#1160`.
GitHub CLI `2.102.0` returned:

```text
GraphQL: was submitted too quickly (addComment)
```

The initial request and two later retries received the same diagnostic.
The retries followed the initial backoff and inserted a pause before each planned mutation.
Processing stopped at the rejected comment each time.
No successful comment receipt or issue closure was recorded.
Those rejected attempts left the reviewed comments pending.
A later paced retry successfully published the comments for `#1160`,
`#1156`,
`#1052` and `#1131` with exact body readback.
`#1160` was then closed.

Read-only issue queries,
Depot metadata queries and primary-document fetches continued to work.
The observation does not establish an authentication failure,
a particular quota,
the scope or reset time of a restriction,
or another session as its cause.

### Current client source trace

The source clone uses GitHub CLI tag `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`.
Its push remote is disabled and its source was not modified.
The paths in this subsection refer to that upstream repository.

`pkg/cmd/pr/shared/commentable.go:262` constructs the comment request and returns a mutation error:

```go
// pkg/cmd/pr/shared/commentable.go
params := api.CommentCreateInput{Body: body, SubjectId: commentable.Identifier()}
url, err := api.CommentCreate(apiClient, opts.Host, params)
if err != nil {
    // Upload first: an uploaded asset outlives the failed write, and the
    // user needs to know it exists before hearing the comment did not save.
    return errors.Join(uploadErr, err)
}
```

`api/queries_comments.go:74` sends `CommentCreate` through the GraphQL mutation client:

```go
// api/queries_comments.go
err := client.Mutate(repoHost, "CommentCreate", &mutation, variables)
if err != nil {
    return "", err
}
```

`api/client.go:123` creates that client and passes its result through the response handler:

```go
// api/client.go
func (c Client) Mutate(hostname, name string, mutation any, variables map[string]any) error {
    opts := clientOptions(hostname, c.http.Transport)
    opts.Headers[graphqlFeatures] = features
    gqlClient, err := ghAPI.NewGraphQLClient(opts)
    if err != nil {
        return err
    }
    return handleResponse(gqlClient.Mutate(name, mutation, variables))
}
```

This traces the failed operation to `addComment`.
It does not expose the server's deciding throttle or prove why that throttle applied.

### Verification and workaround state

`gh --version` reported `2.102.0`.
The actual posting command was:

```bash
# Authorized trial action, not a disposable verification probe.
gh issue comment 1160 --repo Aquaticats/Monochromatic \
  --body-file coherent-reconciliation-comment-1160.md
```

The private controller reconciles the full live issue thread against a unique comment marker before posting.
It requires an exact body readback before closing an issue.
The paced variant waits 1100 milliseconds before a comment or closure mutation.
Both posting variants stopped on the diagnostic.

The working catalog is read-only issue inspection,
repository-scoped Depot metrics collection and MCP document fetching.
The failing catalog is the original `addComment` request and both paced retries.
Local serialization did not recover this incident during those observations.
No bypass,
credential rotation or automatic repost through a different API was attempted.

The [current REST best-practices document][github-best-practices] was fetched again on 2026-10-17
through the configured `search_fetch` MCP.
It still prescribes honoring rate-limit headers,
otherwise waiting at least a minute,
then increasing delays and stopping after a bounded retry count.
That REST guidance informs conservative retry scheduling;
it does not identify this GraphQL rejection's specific limit.
The retained primary text is in the
[repository accounting evidence](../audit/depot-repository-accounting-2026-10-17.evidence.json).

The later retry verified recovery of the requested operation,
not the cause or timing of a server reset.
The controller reused the reviewed bodies and reconciled each live marker before posting.
Exact body readback preceded the `#1160` closure.
The [trial handover](../handover/depot-trial-grilling.md) records the recovered publication state.
This recovery preserves the earlier rejections without claiming that local pacing alone cleared them.

A tracker search found [CLI issue #3292](https://github.com/cli/cli/issues/3292)
about rate-limit handling and [CLI issue #4801](https://github.com/cli/cli/issues/4801)
about the same error wording during issue creation.
Both full threads were retained with their comment counts checked.
The historical maintainer explanation in `#4801` describes a dynamic platform content-creation restriction.
It supports waiting rather than treating primary API quota as a complete explanation.
It does not establish the exact condition or limit for this `addComment` incident.

### Upstream filing decision for this incident

1.  An upstream defect is not established.
    A server rejection alone does not establish an incorrect client or provider behavior.
2.  The deciding server condition is unknown;
    no architectural impossibility is claimed.
3.  Commenting on repository issues is a supported CLI surface.
4.  Contribution acceptance was not evaluated because no upstream defect is established.
5.  `#4801` historically declined making individual CLI commands block for long retry periods.
    The broader rate-limit interface discussion in `#3292` remains open.
6.  No upstream fix is justified by the current evidence.
    The consumer pacing variant first failed and later completed the requested publication.

There is no fileable upstream defect report from this observation.
The authorized current-repository comments are published;
no new upstream issue or comment was sent.

[github-best-practices]: https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
[github-create-issue]: https://docs.github.com/en/rest/issues/issues#create-an-issue
[gh-api-client]: https://github.com/cli/cli/blob/v2.97.0/pkg/cmd/api/api.go#L390-L405
[gh-api-do]: https://github.com/cli/cli/blob/55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55/pkg/cmd/api/http.go#L83
[gh-api-headers]: https://github.com/cli/cli/blob/55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55/pkg/cmd/api/api.go#L474-L478
[gh-api-input]: https://github.com/cli/cli/blob/55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55/pkg/cmd/api/api.go#L369-L381
[go-gh-timeout]: https://github.com/cli/go-gh/blob/v2.13.0/pkg/api/client_options.go#L59-L61
[go-gh-transport]: https://github.com/cli/go-gh/blob/v2.13.0/pkg/api/http_client.go#L59-L69
[p-limit-clear-source]: https://github.com/sindresorhus/p-limit/blob/v7.3.1/index.js#L77-L89
[p-limit-clear-type]: https://github.com/sindresorhus/p-limit/blob/v7.3.1/index.d.ts#L17-L27
[p-limit-immediate]: https://github.com/sindresorhus/p-limit/blob/v7.3.1/index.js#L59-L63
[p-limit-resume]: https://github.com/sindresorhus/p-limit/blob/v7.3.1/index.js#L18-L24
