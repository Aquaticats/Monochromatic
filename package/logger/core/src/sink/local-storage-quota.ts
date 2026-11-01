/**
 Per-runtime default localStorage quota heuristics.
 
 The Web Storage API exposes no way to read the localStorage quota (unlike
 `navigator.storage.estimate()`, which reports the unrelated persistent-storage
 budget), so the sink caps its own footprint from a table of measured
 defaults. Each figure was fill-probed on a fresh store: values are written at
 a growing single key until a `QuotaExceededError`, binary-searching the
 largest that fits. Figures are UTF-16 code units (JS string length, counting
 key plus value) because that is what localStorage measures and what the sink
 compares `serialized.length` against.
 
 @module
 */

import {
  detectWebStorageRuntime,
  type WebStorageRuntime,
} from './web-storage-runtime.ts';

/**
 Measured default per-origin localStorage quotas, in UTF-16 code units, one
 bucket per detectable runtime:
 
 - `deno`: 10,477,569 on Deno 2.9, which lands 8 KiB short of the 10 MiB its
   sessionStorage measures, presumably backing-store overhead; the measured
   figure is kept as-is rather than rounded up past what actually fits.
 - `node`: 5 MiB on Node 26 launched with `--localstorage-file`.
 - `browser`: 5 MiB, measured on headless Chromium 149 over an
   `http://127.0.0.1` origin. Firefox and WebKit are assumed to share the
   bucket: both measured 5 MiB for sessionStorage under Playwright v1.61 and
   neither was fill-probed for localStorage here.
 
 Bun 1.3 exposes no `localStorage`, so its bucket is uncapped: its sink never
 verifies and never reaches the cap. An unrecognized runtime is also uncapped
 so the caller relies on reactive eviction alone.
 */
const RUNTIME_QUOTA_CHARS: Record<WebStorageRuntime, number> = {
  browser: 5_242_880,
  bun: Number.POSITIVE_INFINITY,
  deno: 10_477_569,
  node: 5_242_880,
  unknown: Number.POSITIVE_INFINITY,
};

/**
 Detects the current runtime's default localStorage quota in UTF-16 code
 units, or `Number.POSITIVE_INFINITY` when the runtime is unrecognized so the
 caller leaves its footprint uncapped and relies on reactive eviction alone.
 
 @returns Total quota in code units, or `Number.POSITIVE_INFINITY` if unknown.
 
 @example
 ```ts
 const capChars = detectLocalStorageQuotaChars() / 2; // half the total
 ```
 */
export function detectLocalStorageQuotaChars(): number {
  return RUNTIME_QUOTA_CHARS[detectWebStorageRuntime()];
}
