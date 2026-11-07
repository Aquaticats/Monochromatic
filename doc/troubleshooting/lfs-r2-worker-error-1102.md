# LFS R2 Worker answers git-lfs batches with Cloudflare Error 1102: one R2 `head` per object exhausts the 10 ms CPU limit

Worker `monochromatic-lfs` (`package/config/lfs-r2-worker`),
 git-lfs 3.8.1,
 Cloudflare Workers runtime on a compatibility date of 2026-06-01,
 observed and fixed on 2026-11-01.

## Symptom

Every `git push` in this repository runs `git-lfs push`,
 and the installed wrapper (0.5.1) adds `--all`,
 so each push sends the history's 2174 objects to the Worker in upload batches.
git-lfs sends 100 objects per batch unless `lfs.transfer.batchSize` says otherwise.
Large batches were answered with HTTP 503 and this JSON body:

```json
{"title":"Error 1102: Worker exceeded resource limits","status":503,"detail":"A Worker script configured by the website owner exceeded its resource limits (CPU time or memory) and was terminated."}
```

Read-only probes (an `upload` batch naming objects the server already holds) showed:

- 10,
  25 and 50 objects:
  HTTP 200 twice each;
- 75 and 100 objects:
  HTTP 503,
  Error 1102;
- earlier the same day 100-object batches sometimes passed,
  and a later push at `lfs.transfer.batchSize=25` failed 56 of 87 batches,
  so the failure is not a clean batch-size threshold.

git-lfs treats the 503 as fatal and does not shrink the batch
(see "Root cause",
client behavior),
 so a push died part way (`35% (769/2169)`).

## Root cause

The Worker spent its CPU on R2 `head` calls,
 one per object,
 and the Cloudflare Workers Free plan allows 10 ms of CPU per request.

### What the platform documents

[Workers limits][limits-cpu] list CPU time per HTTP request as 10 ms on Workers Free
and 5 min (default 30 s) on Workers Paid,
 and state that exceeding it returns Error 1102 with the message `Worker exceeded resource limits`.
Memory is 128 MB per isolate and also ends in Error 1102.
Subrequests,
 which include R2 calls,
 are capped at 50 external and 1,000 to internal services on Free.
The 100-object batches stay under the internal cap,
 which is why no subrequest error appeared.

The account plan was not read from the dashboard or API.
It is inferred from the tail evidence below:
 failing invocations were killed at `cpuTime` 10 ms,
 which is the Free figure.

### What `wrangler tail` measured

`wrangler tail --format json` reports `cpuTime`,
 `wallTime` and `outcome` for every invocation.
Failing invocations on the original version `b0009f5d` read `outcome: exceededCpu`,
 `cpuTime: 10`,
 `wallTime` 32 to 35 ms,
 with an `exceptions` entry and no log lines,
 so they were killed before the handler's first log.
Memory is therefore not the cause,
 and neither is a leak that grows across requests:
 the tail never showed `exceededMemory`,
 and a local workerd run kept the V8 heap flat at 6 to 8 MB over 1000 consecutive 100-object batches.

Per-object cost on the live account (successful invocations only):

- original version `b0009f5d`:
  25 objects 6 to 7 ms,
  75 objects 23 ms,
  100 objects 28 ms,
  so roughly 0.25 to 0.3 ms per object;
- a throwaway Worker bound to the same bucket made `head` calls on the same objects:
  25 heads 14 ms,
  100 heads 25 to 43 ms,
  100 sequential heads 64 ms,
  100 heads at concurrency 6 57 ms,
  and an empty handler 0 ms.

Concurrency shape therefore does not help.
 The cost is in each R2 binding call,
 not in how they overlap.
The Worker's own JavaScript is not the cost:
 under local workerd with a fake in-memory bucket,
 100 objects cost 3.0 ms of workerd CPU and 400 objects 3.9 ms,
 while the real local R2 binding cost 22 ms for 100 objects and 98 ms for 400.

Invocations that stay under the limit are not all killed,
 and some reported 21 to 56 ms of `cpuTime` with `outcome: ok`.
That means the limit is not a plain cut on the reported figure,
 and why one 100-object batch passes while the next dies was not determined.
It explains the observed pattern (passes and failures interleaved at one size),
 and the fix does not depend on it,
 because the fix removes the per-object cost.

### Why small batches also failed

Cold isolates pay more.
In a fresh isolate the first request cost 9 ms for a single object on version `1a334d2c`,
 against 1 to 2 ms for the following ones.
In a probe Worker the default logger's first call cost 2 to 4 ms and the first batch code path 3 to 5 ms.
A 25-object batch warm already costs 6 to 7 ms on the original version,
 so a cold start plus 25 heads exceeds 10 ms,
 and a killed isolate is replaced by another cold one.
That chain matches 56 consecutive 25-object failures after 87 passes.

### What git-lfs does with the answer

Source read at tag v3.8.1
([`lfshttp/errors.go` line 75][errors-go]):

```go
if res.StatusCode > 499 && res.StatusCode != 501 && res.StatusCode != 507 && res.StatusCode != 509 {
    return errors.NewFatalError(err)
}
```

A 503 is fatal,
 so a failing batch fails its objects without retry
 ([`tq/transfer_queue.go` lines 586 to 610][transfer-queue]).
No special case exists for 413,
 so a 413 would also fail the push and the client would not shrink its batch.
A response that omits objects leaves them unanswered
 and the transfer queue waits for them,
 so partial answers are not an option either.

`lfs.transfer.batchSize` is read from git config
 ([`config/config.go` line 198][config-go]),
 but `.lfsconfig` may set only a short list of keys
 ([`config/git_fetcher.go` lines 182 to 191][fetcher]):
 `lfs.allowincompletepush`,
 `lfs.fetchexclude`,
 `lfs.fetchinclude`,
 `lfs.gitprotocol`,
 `lfs.locksverify`,
 `lfs.pushurl`,
 `lfs.skipdownloaderrors` and `lfs.url`.
The committed `.lfsconfig` therefore cannot shrink batches for other clones.

[limits-cpu]: https://developers.cloudflare.com/workers/platform/limits/#cpu-time
[errors-go]: https://github.com/git-lfs/git-lfs/blob/v3.8.1/lfshttp/errors.go#L75
[transfer-queue]: https://github.com/git-lfs/git-lfs/blob/v3.8.1/tq/transfer_queue.go#L586-L610
[config-go]: https://github.com/git-lfs/git-lfs/blob/v3.8.1/config/config.go#L198
[fetcher]: https://github.com/git-lfs/git-lfs/blob/v3.8.1/config/git_fetcher.go#L182-L191

The four git-lfs links were loaded in `agent-browser` on 2026-11-01,
 and the first and last lines of each range read as quoted
 (`#LC75`,
`#LC586`,
`#LC610`,
`#LC198`,
`#LC182`,
`#LC191`).
The Cloudflare limits link is a heading anchor,
 not a text fragment,
 and was not loaded in `agent-browser`;
 its wording was read from the page's Markdown rendering the same day.

## What was wrong first

- Per-object `rl.debug` log lines looked like the cost.
  Removing them (commit "log one summary per batch") changed nothing measurable:
  100 objects read 28 ms before and 24 to 25 ms after.
  They stay removed because a record per object is wasted work,
  and a test pins it,
  but they were not the cause.
- A memory leak in the logger was suspected because small batches failed after bursts.
  The tail never reported `exceededMemory`,
  and the local heap stayed flat.
- A subrequest limit of 50 would have failed at 50 objects,
  and 50-object batches passed.
- Bounded concurrency does not reduce the CPU of 100 `head` calls (57 ms at concurrency 6 against 25 to 43 ms unbounded).
- Reading every key with `list` was measured too:
  three pages for the 2174 objects cost 32 ms and grow with the bucket.

## Fix

The Worker keeps a presence index in Cloudflare D1
 (database `monochromatic-lfs-index`,
 binding `INDEX`,
 schema in `package/config/lfs-r2-worker/schema.sql`),
 one row per oid stored in R2.
One query for a whole batch replaces one `head` per object.
Measured on the live account:
 100 oids cost 0 to 2 ms and 500 oids cost 1 ms of CPU,
 with a wall time of 22 to 34 ms,
 against 25 to 64 ms of CPU and about 0.9 s of wall time for 100 `head` calls.

R2 stays the source of truth.
 A row proves the object exists,
 a missing row proves nothing:

- upload batch:
  an index hit gets no action,
  a miss gets an upload action without asking R2
  (git-lfs `PUT`s it,
  which is idempotent,
  and the `PUT` records the oid);
- download batch:
  an index hit gets a download action,
  and the first 10 misses are confirmed with `head` and recorded when R2 has them,
  so a stale index heals toward R2,
  and misses beyond 10 are reported absent
  (the one wrong answer the design allows:
  a stored object beyond the cap gets a per-object 404,
  and git-lfs fails that checkout until later batches record the object;
  the cap exists because each `head` costs 0.25 to 0.45 ms and a fresh isolate already uses 5 to 9 ms,
  so 10 misses add 2.5 to 4.5 ms and stay under 10 ms where 20 would reach about 14 ms);
- `PUT /<oid>` records the oid after R2 accepted the bytes,
  and a failing index write is logged and ignored,
  so the object is stored either way;
- no `INDEX` binding or a failing index query falls back to one `head` per object,
  which is the old behavior;
- one summary log line per batch replaces one record per object;
- the root logger builds only the console sink,
  because the default logger probes IndexedDB,
  sessionStorage and localStorage,
  which workerd lacks.

Free-tier quotas for D1 (from the [D1 pricing page][d1-pricing]):
 5 million rows read per day and 100,000 rows written per day on Workers Free.
A push of 2174 objects reads about 2174 rows and writes none,
 so this is not a billing change.

[d1-pricing]: https://developers.cloudflare.com/d1/platform/pricing/

The index was filled once from R2's own listing (2174 keys,
 equal to the 2174 objects of the bucket) with a throwaway Worker bound to both resources,
 which was deleted afterwards.
The runbook records how to repeat that.

## Verification

Versions deployed this session:
 `b0009f5d-9b00-4719-bbb4-1b6e5902867f` (original),
 `79c499e4-2853-4d96-8a6f-5cfbcb89eb5e` (summary logging only),
 `1a334d2c-bf3b-40d1-bd5b-c008576b2dd3` (D1 index),
 `8028faf7-33bf-4a28-95d6-e4648d54db4e` (D1 index and console-only logger).

Failing catalog on `b0009f5d`:
 six consecutive 100-object `upload` batches answered 503 Error 1102,
 and the tail showed `exceededCpu`,
`cpuTime` 10 ms,
`wallTime` about 32 ms for each.

Working catalog on `8028faf7`,
 probes of already-stored objects,
 every response HTTP 200 with the full object list:

- 100-object upload batches:
  5 sequential,
  plus 6 started in parallel;
- 200-object upload batches:
  5 sequential;
- 25-object upload batches:
  20 back to back with no pause;
- anonymous download batches of 100 objects:
  3;
- one batch with a stored and a fabricated oid:
  upload gave "no action" and an `upload` action,
  download gave a `download` action and a per-object 404.

The tail for the whole version (59 invocations,
 including other sessions' traffic) held no non-`ok` outcome;
 100-object invocations read 1 to 8 ms except one at 13 ms (25 invocations),
 200-object invocations read 2 to 4 ms (5 invocations),
 and the median was 2 ms.
Version `1a334d2c` had 10 invocations,
 none failing,
 maximum 9 ms.
A fresh isolate's first request read 9 ms on `1a334d2c`,
 5 ms on `8028faf7` and 9 ms on the final version `00c0cac8` (one sample each),
 so the live data does not show that the console-only logger shortens a cold start.
It is kept because the probe Worker measured the default logger's first call at 2 to 4 ms,
 and no unit test guards it,
 because the sink choice is not observable from the Node test harness.

Final version `00c0cac8-0932-41c2-a778-d0e37cf041d1` (cap of 10 download misses),
 same probes:
100-object upload x5,
 200-object upload x3,
 25-object upload x20 back to back,
 anonymous 100-object download x3,
 and a real `git-lfs push --all` at batch size 100 (22 batches,
 exit 0,
 `100% (2169/2169)`).
Every response was HTTP 200 and the tail held 28 invocations,
 all `ok`:
 100-object invocations read 1 to 4 ms and one 10 ms,
 200-object invocations 3,
5 and 7 ms,
 25-object invocations 1 to 2 ms,
 and the first request of the fresh isolate (1 object) 9 ms.

Real `git-lfs push --all origin refs/heads/main` from the main checkout,
 which uploads nothing because the server holds everything:

- at the clone's `lfs.transfer.batchSize=25`:
  exit 0,
  87 batches (86 of 25 and one of 19),
  `Uploading LFS objects: 100% (2169/2169)`;
- with `lfs.transfer.batchSize=100` forced for the run:
  exit 0,
  22 batches (21 of 100 and one of 69),
  `100% (2169/2169)`.

Unit tests (`src/presence-index.unit.test.ts`) pin the cost per object:
 zero `head` calls when the index covers a batch of 100 and one of 501 (two queries),
 a bounded number of `head` calls on a miss,
 the same number of log records for 1 and 100 objects,
 recording on `PUT`,
 and the fallbacks.
Each guard was shown to fail without its fix:
 disabling the index lookup failed five tests that count `head` calls,
 removing the `PUT` recording failed `records the stored oid in the index after the bucket accepted it`,
 and a per-object `debug` call failed `writes the same number of log records for one object as for a default-size batch`.

## Verified workarounds

- Setting `git config --local lfs.transfer.batchSize=25` on the pushing clone shortened batches,
  but it did not stop the failures (56 of 87 batches failed at 25 on the original Worker),
  so it is not a workaround for this incident.
  The index fix makes it unnecessary.

## What does not work

- A 413 guard for oversized batches:
  git-lfs does not shrink the batch on a 413.
- Bounded concurrency and `list`-based membership:
  measured above.
- Setting the batch size in the committed `.lfsconfig`:
  the key is ignored there.
- Workers Paid:
  it would lift the CPU limit,
  but it is a billing change and was not needed.

## Open items

- The cold-start cost (5 to 9 ms in a fresh isolate for one object) leaves 1 to 5 ms for the batch itself,
  and a 100-object batch costs 1 to 8 ms warm.
  A 100-object request read 13 ms once and was not killed.
  If failures return,
  `wrangler tail` will show `exceededCpu` with a small object count,
  and Workers Paid is the remaining remedy.
- The plan was inferred,
  not read from the account.

## Upstream filing decision

Nothing to file.
The behavior is a documented Cloudflare limit,
 the defect was in this repository's Worker,
 and git-lfs behaved as its source says.
