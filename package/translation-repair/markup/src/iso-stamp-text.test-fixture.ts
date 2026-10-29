//region Stamps the logger never writes
// The console sink stamps every log line with `Date.prototype.toISOString()`
// (`package/logger/core/src/sink/console.ts`), which writes
// `YYYY-MM-DDTHH:mm:ss.sssZ` for every year the logs span. `Date.parse` reads
// more than that, and each stamp here is one it reads, or may read, that the
// logger never writes (ledger B73). The readers of log stamps share the list
// so each is shown refusing the same spellings.

/**
 A stamp as the logger writes it, which the spellings in
 {@link STAMPS_NOT_WRITTEN} vary.
 */
export const WRITTEN_STAMP = '2026-08-25T10:00:10.000Z';

/**
 Spellings of {@link WRITTEN_STAMP}'s instant, or of text near it, that the
 logger never writes.
 */
export const STAMPS_NOT_WRITTEN = [
  // The zone left off, which ECMAScript reads as local time.
  '2026-08-25T10:00:10.000',
  // The date alone, which ECMAScript reads as midnight UTC.
  '2026-08-25',
  // The milliseconds left off.
  '2026-08-25T10:00:10Z',
  // An offset where the logger writes Z.
  '2026-08-25T10:00:10.000+00:00',
  // A spelling outside the ISO format, which V8 reads by a fallback the
  // standard leaves to each engine.
  'Aug 25 2026 10:00:10',
  // A stamp cut off part way through its seconds.
  '2026-08-25T10:00:1',
  // The six-digit year form, which the logger writes only past year 9999.
  '+002026-08-25T10:00:10.000Z',
  // No stamp at all.
  '',
] as const;

//endregion Stamps the logger never writes
