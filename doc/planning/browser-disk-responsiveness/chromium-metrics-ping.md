# BrowserMetrics live mapped writes can block browser main on Btrfs: owner coordination

This is an initial metrics-owner coordination request based on a downstream capture,
not a claim of a reproducible bug in latest upstream Chromium.
Is live `BrowserMetrics` mapped-write blocking already tracked,
and what crash-retention guarantees should a regression test preserve?

## Observed boundary

Helium `0.20.2.1`,
based on Chromium `156.0.8078.73` (`2f9a11097447f226133bc1a31c787291a5eeab98`),
showed whole-window unresponsiveness on Linux with Btrfs.
A subsequent main-thread capture at `2026-11-03T07:05:04.385Z` correlated:

- A user-mode write-fault event with error code `0x7`.
- Its instruction pointer matching the blocked main thread's sampled current instruction pointer.
- Its target address at offset `0x1b61cc` inside a `4194304`-byte,
  `rw-s` mapping of `<profile>/BrowserMetrics/<active-file>.pma (deleted)`.
- This kernel stack:

```text
# Sanitized browser-main procfs stack
folio_wait_bit_common
folio_wait_writeback
btrfs_page_mkwrite
do_page_mkwrite
do_wp_page
__handle_mm_fault
handle_mm_fault
do_user_addr_fault
exc_page_fault
asm_exc_page_fault
```

A matching observation at `2026-11-03T07:13:58.080Z`
was followed by a recovery record reporting `6622 ms` in the sampled blocked interval.
The observer waited `100 ms` between samples and logged changes of kernel stack.
This is sampled blocked time,
not an exact fault duration or an input-to-presentation measurement.
The correlated boundary is established at the recorded sample;
the interval may include additional waits.
The mapping was marked deleted in procfs,
and remained mapped.

The observer recorded write faults only.
Read-fault samples with stale preceding write events were excluded from this attribution.
Ordinary Chromium separately showed `btrfs_page_mkwrite` on browser main,
but its faulting file was not identified.
That separate observation does not confirm `BrowserMetrics` attribution in upstream Chromium.

## Relevant upstream code and nearby work

At inspected upstream revision `e436fd72f52814c1228fb16a8c8f7cae8253fb99`,
[normal browser startup explicitly selects mapped persistent histograms before field trials][startup].
[The mapping setup and local-memory fallback][allocation] explain why moving explicit flushes
off the recording thread would not by itself remove a later filesystem-backed write fault.
This is a source-based inference about the mechanism,
not a runtime reproduction on that revision.

[CL 8707393][segments] caches system-profile segment references to reduce cold mapped reads.
[CL 8688624][sparse] moves scans outside a sparse-histogram lock and batches imports.
Both were open when checked on `2026-11-03`.
Their described mechanisms differ from the captured live mapped write.
CL 8707393 references issue `580929788`,
whose discussion was inaccessible during this check;
its scope is therefore unknown.

## Reproduction and design limits

There is no deterministic reproduction or tested Chromium patch in this report.
Latest Chromium/Canary has not been reproduced with a disposable profile,
and the user-space instruction has not been symbolized.
The historical capture came from ordinary use,
without synthetic host load.
No inference is made about freeze frequency on other systems.

A local fallback check removed the `.pma` mapping,
but loses persistent metrics and does not prove that every freeze is fixed.
Replacing live mapped recording with RAM-backed recording and separate persistence would need
an agreed crash-loss window,
allocator lifetime,
cross-process import,
snapshot consistency,
and consent/deletion contract.
No replacement design is proposed for landing here.

The next contribution boundary is a disposable regression fixture that holds the relevant storage work pending
while checking browser-main task progress and retained metrics.
Please route this evidence to existing work if the live-write case is already covered.

Prepared with AI assistance from retained local observations and Chromium source inspection.
No profile contents or raw private trace files are included.

[startup]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/chrome/app/chrome_main_delegate.cc#L1803-L1825
[allocation]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/components/metrics/persistent_histograms.cc#L124-L222
[segments]: https://chromium-review.googlesource.com/c/chromium/src/+/8707393
[sparse]: https://chromium-review.googlesource.com/c/chromium/src/+/8688624
