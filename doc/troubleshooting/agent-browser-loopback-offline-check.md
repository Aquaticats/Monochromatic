# agent-browser 0.41.2: an offline render check cannot block egress while serving the page from loopback

## Symptom

A guard that serves a directory on `127.0.0.1` and wants the page rendered with no request leaving that server has
two natural probes,
and both fail on the served page itself rather than on the dependency:

```text
agent-browser set offline on <session>
agent-browser open http://127.0.0.1:8796/candidates/cover-a.dc.html <session>
✗ Navigation failed: net::ERR_INTERNET_DISCONNECTED
```

The tab then sits on `chrome-error://chromewebdata/`,
so a probe that reads the rendered tree reports the error page
rather than the design page:

```json
{"url":"chrome-error://chromewebdata/","root":-1}
```

Aborting every URL with `network route "**" --abort` and then registering the loopback origin fails the same way,
because the wildcard route aborts the document before the narrower route is consulted:

```text
agent-browser network route "**" --abort <session>
agent-browser network route "http://127.0.0.1:8796/**" <session>
agent-browser open http://127.0.0.1:8796/candidates/cover-a.dc.html <session>
✗ Navigation failed: net::ERR_FAILED
```

A third symptom belongs to the caller rather than the tool,
and cost longer than the first two:
a sweep whose page
server runs in the same Node process never finishes.
The batch process does not return,
its own deadline fires,
and
the run reports a timeout with no output:

```text
elapsed ms 298118
status null error Error: spawnSync agent-browser ETIMEDOUT
```

## Root cause

Inspected release:
`v0.41.2`,
commit `d52a5e90c372ec4710c69adc3837ac14bbd852aa`,
read from a private clone at `~/temp/agent/agent-browser-source-2026-11-10`
checked out at that commit.
The installed binary reports `agent-browser 0.41.2`.

`set offline on` reaches Chrome as CDP network emulation for the whole session,
not as a per-host rule (see
[`cli/src/native/network.rs:28-44`][network-offline]):

```rust
// cli/src/native/network.rs:28-44
pub async fn set_offline(
    client: &CdpClient,
    session_id: &str,
    offline: bool,
) -> Result<(), String> {
    client
        .send_command(
            "Network.emulateNetworkConditions",
            Some(json!({
                "offline": offline,
                "latency": 0,
                "downloadThroughput": -1,
                "uploadThroughput": -1,
            })),
            Some(session_id),
        )
```

`Network.emulateNetworkConditions` with `offline: true` applies to every URL the tab requests,
loopback included,
so
the served document never loads.
That is Chrome's behavior rather than a choice this CLI makes.

`network route` cannot express "everything except this origin".
A paused request is matched by walking the routes in
registration order and taking the first that matches and then aborts or answers (see
[`cli/src/native/actions.rs:12452-12453`][route-matching]):

```rust
// cli/src/native/actions.rs:12452-12453
    // Route matching
    for route in routes {
```

```rust
// cli/src/native/actions.rs:12465-12467
            if route.abort {
                let _ = client
                    .send_command(
                        "Fetch.failRequest",
```

A route registered without `--abort` and without `--body` matches nothing this loop acts on (see
[`cli/src/native/actions.rs:12465-12467`][route-abort]),
so the wildcard registered before it still aborts the loopback
document and returns.
Registration order decides which route answers,
and no pattern or later route exempts a URL an
earlier abort already claims.

Two more measured properties shaped the workaround below.
`network requests --clear` cannot report what it cleared,
so it cannot hand a caller the control fixture's requests (see
[`cli/src/native/actions.rs:12722-12726`][requests-clear]):

```rust
// cli/src/native/actions.rs:12722-12726
async fn handle_requests(cmd: &Value, state: &mut DaemonState) -> Result<Value, String> {
    if cmd.get("clear").and_then(|v| v.as_bool()).unwrap_or(false) {
        state.tracked_requests.clear();
        return Ok(json!({ "cleared": true }));
```

And Node's `child_process.spawnSync` blocks the calling event loop for the child's whole life,
so an `http` server in
that same process cannot answer any request the child's browser makes.
The browser waits on a document that only the
blocked loop could serve.

## Verification

Measured with `agent-browser 0.41.2`,
headless Chrome `157.0.8103.38`,
and Node `26.13.0`,
serving
`package/music-player/design/` with `python3 -m http.server 8796 --bind 127.0.0.1`.

Each cited line was confirmed at that commit by reading the raw file in `agent-browser`:
the route loop at
`actions.rs:12452`,
the abort branch at `:12465`,
and the clearing return at `:12725` all print at those line numbers.
The blob pages linked above are the human-readable form of the same citations;
a headless read of that 12,000-line
file stops before those lines,
so the raw file is what confirmed them.

Working catalog:

- `open http://127.0.0.1:8796/candidates/cover-a.dc.html` with offline mode off loads the page and its dependencies.
- `network requests --json --filter http` lists every request the session made,
  with `resourceType` and `status`.
- `network route "**" --abort` alone blocks the document,
  which is what it says.
- A sweep run from `child_process.spawn` (not `spawnSync`) against a server in the same process completes:
  the design
  tree's own guard renders all 97 pages in about 19 s this way.

Failing catalog:

- `set offline on` then any loopback URL:
  `net::ERR_INTERNET_DISCONNECTED`,
  and the tab lands on the error page.
- `network route "**" --abort` then a loopback passthrough route:
  `net::ERR_FAILED` for the loopback document.
- `network requests --clear` returns `{"cleared": true}` and the cleared entries are gone.
- `spawnSync` around `agent-browser batch` while the same process serves the pages:
  no return inside 240 s,
  killed at
  its deadline.

The runnable harness is the guard this work produced,
which reports a finding rather than a browser error for each of
the four failing cases above:

```bash
mise run //package/music-player/design:test:offline-pages:render
```

Its output on the design tree:

```text
Offline design pages in a browser: 97 pages rendered, 906 requests all answered inside this tree, every shipped face loaded.
```

## Verified workarounds

Serve the tree on loopback and assert on the request record instead of blocking egress.
Every recorded request must
start with the served origin,
no response may be missing for a path a page or its stylesheets name,
and no page error
may be recorded.
One more assertion makes the record self-checking:
some request must name a file only the tree
serves,
so a browser that answered everything from cache fails the check instead of passing it silently.
Tradeoff:
this proves the page asked nothing outside the tree rather than proving it renders with the network cut,
so
a dependency that a page needs and never requests cannot be caught this way.
The render assertions cover content,
and
the static guard covers references.

Run the control fixture in a sweep of its own.
A fixture page that mounts nothing and asks a host for a stylesheet
then produces its own findings,
which are asserted as proof that the guard can fail,
and they stay out of the page
findings.
Tradeoff:
one extra browser launch,
about two seconds.

Use `child_process.spawn` and await the child's close,
with a deadline that kills the child.
The serving loop keeps
running while the browser works.
Tradeoff:
the caller owns the deadline and the kill,
where `spawnSync` carried both;
the guard's `runChild` helper does exactly that.

## What does not work

- `set offline on` with a loopback server:
  the document itself is blocked.
- `network route "**" --abort` plus a narrower passthrough route:
  the wildcard is consulted first and aborts.
- `network requests --clear` to collect the control fixture's requests:
  it returns only `{"cleared": true}`.
- `spawnSync` for the browser batch while the same process serves the tree:
  the batch waits on a server the blocked
  loop cannot run.
- Using `file://` to dodge the server:
  pages that `import()` a sibling module cannot do so from a `file://` document,
  so they render partially.
  Classic scripts load fine from `file://`,
  which is why the runtime's own vendored builds
  work there.

## Upstream filing decision

Nothing was filed,
and nothing is worth filing.

1. Fault:
   not upstream's.
   Blocking every origin under `Network.emulateNetworkConditions` is Chrome's contract,
   and
   first-match route selection is a documented-style matching model rather than a defect.
2. Fixability:
   a negative-route or exemption concept could be added,
   but it would be a feature request against a
   design that already works as described,
   not a bug fix.
3. Supported use:
   serving a directory locally and checking its requests is a supported combination today through
   `network requests`.
4. Contribution policy:
   not relevant to a no-file decision.
5. Likely fix:
   no signal either way;
   no existing issue was searched for,
   because nothing is being filed.
6. Prototype:
   none,
   and none is warranted for a feature request.

`.out-of-scope/` holds no entry for agent-browser,
and this decision does not depend on one.

[network-offline]: https://github.com/vercel-labs/agent-browser/blob/d52a5e90c372ec4710c69adc3837ac14bbd852aa/cli/src/native/network.rs#L28-L44
[route-matching]: https://github.com/vercel-labs/agent-browser/blob/d52a5e90c372ec4710c69adc3837ac14bbd852aa/cli/src/native/actions.rs#L12452-L12453
[route-abort]: https://github.com/vercel-labs/agent-browser/blob/d52a5e90c372ec4710c69adc3837ac14bbd852aa/cli/src/native/actions.rs#L12465-L12467
[requests-clear]: https://github.com/vercel-labs/agent-browser/blob/d52a5e90c372ec4710c69adc3837ac14bbd852aa/cli/src/native/actions.rs#L12722-L12726
