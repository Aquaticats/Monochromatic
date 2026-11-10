# JSR npm bridge: Bun's conditional GET cache triggers 502 Bad Gateway from npm.jsr.io

> **Historical context (2026-05-14).
> ** This document is retained for
> source-trace reference.
>  The workspace no longer consumes JSR-hosted packages
> (see `PHILOSOPHY.tool-choices.md`) and uses pnpm as the package manager,
>  not
> `bun install` (migration landed in `e424ffde build(*): migrate package
> manager from vlt to pnpm`).
>  The conditional-GET 502 cannot trigger on the
> current install path.
>  Both the registry-side and installer-side defects are
> tracked as out-of-scope categories in `.out-of-scope/jsr.md` and
> `.out-of-scope/bun-install.md`;
>  no GitHub tracking issue is open against
> either.
>  The "Verified workaround" section below references a `mise run
> fix:jsr` task that was deleted as part of the pnpm migration;
>  treat it as
> historical reproduction guidance,
>  not as a live workspace task.

## Symptom

```bash
error: GET https://npm.jsr.io/@jsr%2fzod__zod - 502
error: zod@catalog: failed to resolve
```

Every JSR-hosted package (`@logtape/*`,
 `@optique/*`,
 `@cspotcode/outdent`,
 `zod`)
fails with `502 Bad Gateway` when Bun resolves them.
Non-JSR packages resolve fine.
`curl https://npm.jsr.io/@jsr/zod__zod` from the same host returns
200,
 which initially suggests a Bun-side bug;
 the real cause sits on the
JSR edge.

Tracked upstream:
 [jsr-io/jsr#1323](https://github.com/jsr-io/jsr/issues/1323).

## Root cause

JSR's npm compatibility layer is a Cloudflare-edge worker that returns
**502 Bad Gateway** when the request includes an `If-None-Match` header,
regardless of whether the etag is valid,
 stale,
 or garbage.

Bun's package manager maintains an HTTP cache at `~/.bun/install/cache/`
and sends `If-None-Match` from that cache on every subsequent fetch.
`curl` does not send conditional headers by default,
 which is why
ad-hoc manual probes succeed while `bun install` fails.

Per RFC 9110 Section 13.1.2,
 the server must return 304 (etag match) or
200 (no match) for a conditional GET.
 502 is never a valid response.
The defect lives on JSR's side;
 Bun's behaviour conforms to the HTTP
specification.

## Verification

Confirm the JSR edge still rejects conditional requests:

```sh
curl -s -o /dev/null -w "%{http_code}\n" \
  -H 'If-None-Match: "any-value"' \
  "https://npm.jsr.io/@jsr/zod__zod"
```

- 502:
   the JSR bug is still present;
   the workaround below is still needed.
- 200 or 304:
   the upstream fix has landed;
   clear the cache once more,
  then the workaround can be retired.

Reproduce the failing install with:

```sh
bun pm cache rm   # ensure cache holds no etag yet
bun install       # populates cache; this run succeeds
bun install       # second run sends If-None-Match, fails with 502
```

## Verified workaround

Clear Bun's HTTP cache before each fresh resolution so no `If-None-Match`
header is sent:

```sh
bun pm cache rm
bun install
```

Or,
 equivalently,
 the workspace task:

```sh
mise run fix:jsr
```

Tradeoff:
 the cache is reset every time,
 so `bun install` does not benefit
from cached etag negotiation.
 On a workspace with hundreds of dependencies
the bandwidth cost is one full re-download of any JSR metadata that
otherwise would have validated with a 304.
 Tolerable;
 non-JSR caching is
unaffected.

## What does not work

- Editing `~/.bun/install/cache/` to remove only JSR etags:
   Bun does
  not expose per-entry deletion through any CLI;
   the cache file format
  is internal and unstable across versions.
- Setting `BUN_INSTALL_CACHE_DIR=/dev/null` or similar:
   forces a full
  re-resolve on every install but does not stop Bun from caching once
  it runs.
- Switching to `npm` or `pnpm` for installs:
   avoids Bun's conditional
  GET (those clients do not send `If-None-Match` from disk caches in
  the same way),
   but loses Bun's catalog-aware install paths used
  elsewhere in this workspace.

## Why we do not file this upstream

JSR's tracker already has [jsr-io/jsr#1323](https://github.com/jsr-io/jsr/issues/1323)
open against this exact behaviour,
 so a fresh report would duplicate
without adding evidence.
 Walking the five constraints anyway:

1.  **Is it really upstream's fault?
    ** Yes.
     The edge worker is the one
    violating RFC 9110.
2.  **Can upstream fix it?
    ** Yes;
     either accept conditional GET and
    return 304/200,
     or drop incoming `If-None-Match` headers before they
    reach the cache.
3.  **Are they supporting this use case?
    ** The npm-compat bridge is the
    advertised on-ramp for npm tooling;
     Bun is npm-compat tooling.
4.  **Will they likely fix it?
    ** The existing issue has been quiet since
    filing;
     no movement in the JSR repo's release notes touches the
    conditional-GET path.
     Not optimistic.
5.  **Have we prototyped a minimal fix?
    ** Not applicable;
     the JSR worker
    is closed-source from this repo's vantage point,
     and a workspace
    patch is impossible.

Decision:
 keep the workaround in `mise run fix:jsr`;
 re-evaluate when
issue #1323 closes or shows movement.
