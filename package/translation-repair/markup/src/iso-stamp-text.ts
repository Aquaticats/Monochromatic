//region ISO stamp text
// One rule for reading a log line's stamp back out of its text, shared by
// every reader of the logs the package writes (ledger B73).
//
// `Date.parse` IS NOT THAT RULE. The logger stamps every line with
// `Date.prototype.toISOString()` (`package/logger/core/src/sink/console.ts`),
// and `Date.parse` reads far more: a date-time without its zone as local time,
// a date alone as midnight UTC, and spellings outside the ISO format by a
// fallback the standard leaves to each engine. A stamp cut off part way reads
// as NaN, which one reader kept as a call's end and another as a sample's
// time.
//
// THE WRITER'S SPELLING, CHECKED BY WRITING IT AGAIN. A stamp counts only when
// `toISOString` writes the instant `Date.parse` read back as exactly the text
// read, so the rule needs no grammar of its own and cannot drift from the
// writer's.

/**
 Whether text is a stamp exactly as `Date.prototype.toISOString` writes one,
 which is how the logger stamps every line.

 A SPELLING OF THE RIGHT INSTANT IS STILL REFUSED when the logger does not
 write it (no milliseconds, a `+00:00` offset, a six-digit year before
 10000): a line carrying one did not come from the logger as written.

 @param text - stamp as read off a log line

 @returns Whether `Date.parse` reads it as the instant the logger wrote

 @example
 ```ts
 isIsoStampText({ text: '2026-08-25T10:00:10.000Z', },); // true
 isIsoStampText({ text: '2026-08-25T10:00:10', },); // false
 ```
 */
export function isIsoStampText({ text, }: { readonly text: string; },): boolean {
  /**
   Epoch milliseconds `Date.parse` reads, NaN where it reads none.
   */
  const read = Date.parse(text,);
  return Number.isFinite(read,) && (new Date(read,).toISOString() === text);
}

//endregion ISO stamp text
