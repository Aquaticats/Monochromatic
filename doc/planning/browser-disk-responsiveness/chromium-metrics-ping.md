# BrowserMetrics mapped writes block browser main on Btrfs in Chromium 159.0.8141.0

A fresh upstream Chromium run reproduced browser-main blocking while writing through the live
`BrowserMetrics` mapping.
Is this live-write path already tracked by metrics owners,
and which crash-retention guarantees should regression coverage preserve?

## Build and environment

- Official Linux Chromium snapshot `1733707`,
  runtime version `159.0.8141.0`,
  source revision `986947eb4afc18471e51b7cb473f7c2f779f62bf`.
- Linux `7.2.15-ogc1.1.fc45.x86_64`,
  Wayland,
  Btrfs,
  newly created disposable browser profile.
- Headed browser with default histogram storage selection.
  No synthetic host load,
  filesystem cache dropping,
  or injected storage delay.
  The reproduction is intermittent under existing host activity.

## Correlated evidence

At `2026-11-03T09:44:15.429Z`,
the browser main thread was blocked in `folio_wait_writeback` through `btrfs_page_mkwrite`.
The latest write-fault instruction pointer exactly matched its sampled current instruction pointer.
Both captured faulting instructions resolve to `base::SampleVectorBase::Accumulate(int, int)`.
Disassembly identifies the faulting instruction as `lock xadd %r12d,(%rax)`.
The fault address was offset `0x2d678` inside its live `4194304`-byte `rw-s` mapping of
`<disposable-profile>/BrowserMetrics/<active-file>.pma (deleted)`.
The recovery observation followed in a sampled blocked span of `104 ms`.

At `2026-11-03T09:46:52.885Z`,
a second matching write fault targeted offset `0x78ba0` in the same mapping.
This stack shows a metadata-reservation wait:

```text
# Sanitized browser-main procfs stack
handle_reserve_ticket
reserve_bytes
btrfs_reserve_metadata_bytes
btrfs_delalloc_reserve_metadata
btrfs_page_mkwrite
do_page_mkwrite
do_wp_page
__handle_mm_fault
handle_mm_fault
do_user_addr_fault
exc_page_fault
asm_exc_page_fault
```

The sampled blocked span was `1484 ms`.
A `Browser.getWindowForTarget` request started at `09:46:53.108Z`
and took `1159.963 ms`,
finishing within that span.
These are a sampled blocked span and a browser-window command round trip,
not an exact fault duration or input-to-presentation latency.
The observations do not prove that the entire command delay was spent in this fault.

The trace filtered `page_fault_user` to writes by the browser main thread.
Procfs state was checked with a `100 ms` delay between iterations.
All kernel trace buffers reported zero overruns and dropped events;
`4922` write-fault events were read.
An unmatched read-fault sample was excluded from metrics attribution.
The `.pma` file was marked deleted by procfs but remained mapped.

## Recording scenario

The browser was launched with a disposable `--user-data-dir`,
`--remote-debugging-port=0`,
`--no-first-run`,
`--no-default-browser-check`,
and `--ozone-platform=wayland`.
The probe opened `chrome://settings/`
and dispatched these native API calls at a `1000 ms` cadence:

```javascript
// chrome://settings/ DevTools console, equivalent native recording inputs
const sampleCount = 60;
const cadenceMs = 1000;
for (let sample = 0; sample < sampleCount; sample++) {
  chrome.metricsPrivate.recordBoolean(
    'Monochromatic.BrowserMetricsRepro.CurrentBoolean', sample % 2 === 0);
  chrome.metricsPrivate.recordSparseValue(
    'Monochromatic.BrowserMetricsRepro.CurrentSparse', sample);
  await new Promise(resolve => setTimeout(resolve, cadenceMs));
}
```

The automated probe also sent `Browser.getWindowForTarget` before each recording pair.
`Browser.getHistogram` returned exactly `60` samples for each named histogram.
The rendered `chrome://histograms/#Monochromatic.BrowserMetricsRepro.Current`
confirmed the counts with subprocess imports disabled.
The last part of the recording loop continued after the bounded kernel trace ended;
the second correlated fault occurred during their overlap.

At the tested source revision,
[the native recording implementations][recording] directly call `UmaHistogramBoolean` and `UmaHistogramSparse`.
[The extension dispatcher][dispatcher] is documented to live on the UI thread.
The JavaScript recording functions have no completion callback,
so renderer evaluation completion alone was not used as proof of recording completion.

## Related work and scope

[CL 8707393][segments] addresses cold mapped reads during system-profile scanning;
[CL 8688624][sparse] addresses sparse-histogram import lock contention.
Both were open when checked on `2026-11-03`.
Their descriptions differ from this live-write fault.
Issue `580929788`,
linked by CL 8707393,
denied anonymous access,
so its discussion was not assessed.

Prepared with AI assistance from retained captures and source inspection.
The accompanying evidence is sanitized;
no profile contents or raw private traces are included.

[recording]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/browser/api/metrics_private/metrics_private_api.cc#L193-L207
[dispatcher]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/extensions/browser/extension_function_dispatcher.h#L35-L37
[segments]: https://chromium-review.googlesource.com/c/chromium/src/+/8707393
[sparse]: https://chromium-review.googlesource.com/c/chromium/src/+/8688624
