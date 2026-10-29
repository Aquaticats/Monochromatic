# module-const

Ready to publish.

Universal constants for the Monochromatic monorepo.
This package is the canonical home for values whose authority is context-free:
mathematical relationships,
 physical unit relationships,
 and externally specified code or character sets.
App-specific defaults and domain-format sentinels stay with the package that owns that policy or parser.

## Exports

The package is source-only.
The `.` entry point resolves to `./src/index.ts`,
which re-exports every constant from per-category sibling files.

### ASCII

Defined in `src/ascii.ts`:

- `ASCII_LOWERCASE_LETTER_CHARS`:
   lowercase ASCII letters in code-point order.
- `ASCII_DECIMAL_DIGIT_CHARS`:
   decimal ASCII digits in code-point order.
- `ASCII_LOWERCASE_ALPHANUMERIC_CHARS`:
   lowercase ASCII letters followed by decimal ASCII digits.

### Time

Defined in `src/time.ts`:

- `MS_PER_SECOND`:
   milliseconds in one second.
- `SECONDS_PER_MINUTE`:
   seconds in one minute.
- `MINUTES_PER_HOUR`:
   minutes in one hour.
- `HOURS_PER_DAY`:
   hours in one day.
- `DAYS_PER_WEEK`:
   days in one week.
- `DAYS_PER_YEAR`:
   days in a non-leap calendar year.
- `MONTHS_PER_YEAR`:
   months in one year.
- `MS_PER_MINUTE`:
   `MS_PER_SECOND * SECONDS_PER_MINUTE`.
- `MS_PER_HOUR`:
   `MS_PER_MINUTE * MINUTES_PER_HOUR`.
- `MS_PER_DAY`:
   `MS_PER_HOUR * HOURS_PER_DAY`.
- `MS_PER_WEEK`:
   `MS_PER_DAY * DAYS_PER_WEEK`.
- `SECONDS_PER_HOUR`:
   `SECONDS_PER_MINUTE * MINUTES_PER_HOUR`.
- `SECONDS_PER_DAY`:
   `SECONDS_PER_HOUR * HOURS_PER_DAY`.

### Byte and bit units

Defined in `src/byte.ts`:

- `BITS_PER_BYTE`:
   bits in one byte.
- `BYTES_PER_KIB`:
   IEC binary kibibyte,
   `1_024` bytes.
- `BYTES_PER_MIB`:
   IEC binary mebibyte,
   `2^20` bytes.
- `BYTES_PER_GIB`:
   IEC binary gibibyte,
   `2^30` bytes.
- `BYTES_PER_TIB`:
   IEC binary tebibyte,
   `2^40` bytes.
- `BYTES_PER_KB`:
   SI decimal kilobyte,
   `1_000` bytes.
- `BYTES_PER_MB`:
   SI decimal megabyte,
   `10^6` bytes.
- `BYTES_PER_GB`:
   SI decimal gigabyte,
   `10^9` bytes.
- `BYTES_PER_TB`:
   SI decimal terabyte,
   `10^12` bytes.

### HTTP status codes

Defined in `src/http-status.ts`:

- `HTTP_OK`:
   RFC 9110 success.
- `HTTP_CREATED`:
   RFC 9110 resource created.
- `HTTP_NO_CONTENT`:
   RFC 9110 success with empty body.
- `HTTP_NOT_MODIFIED`:
   RFC 9110 conditional request cache hit.
- `HTTP_BAD_REQUEST`:
   RFC 9110 client malformed request.
- `HTTP_UNAUTHORIZED`:
   RFC 9110 missing authentication.
- `HTTP_FORBIDDEN`:
   RFC 9110 authorization refused.
- `HTTP_NOT_FOUND`:
   RFC 9110 resource missing.
- `HTTP_CONFLICT`:
   RFC 9110 state conflict.
- `HTTP_INTERNAL_SERVER_ERROR`:
   RFC 9110 unexpected server failure.

### Fractions

Defined in `src/fraction.ts`:

- `HALF`:
   `1 / 2`.
- `QUARTER`:
   `HALF / 2`.
- `THREE_QUARTERS`:
   `HALF + QUARTER`.
- `THIRD`:
   `1 / 3`.
- `TWO_THIRDS`:
   `THIRD + THIRD`.

## Usage

```ts
import {
  ASCII_LOWERCASE_ALPHANUMERIC_CHARS,
  BYTES_PER_KIB,
  HALF,
  HTTP_NOT_FOUND,
  MS_PER_SECOND,
} from '@monochromatic-dev/const-core';

const ttlMs = 30 * MS_PER_SECOND;
const bufferBytes = 64 * BYTES_PER_KIB;
const midpoint = (start + end) * HALF;
const tokenChars = ASCII_LOWERCASE_ALPHANUMERIC_CHARS;
return new Response(tokenChars[0], { status: HTTP_NOT_FOUND, },);
```

## Scope

In scope:

- Mathematical and physical relationships,
   including time-unit ratios,
   byte-unit ratios,
   and fractions.
- Codes and character sets defined by external specifications,
   including RFC 9110 HTTP status codes and ASCII.
- Context-free constants whose meaning does not change when a consuming package changes policy.

Out of scope:

- App-specific defaults,
   including timeouts,
   ports,
   retry counts,
   and z-index tiers.
  Centralising these would couple unrelated apps into lockstep when one wants to tune a default.
  Define such constants in the package that owns the policy.
- Domain-specific magic numbers,
   including ZIP format signatures and file-format thresholds.
  Define these in the package's `constants.ts`,
   next to the code that interprets them.

## KB vs KiB

The IEC and SI byte units differ at every prefix beyond bytes.
This package exports both with their spec-correct values:

- `BYTES_PER_KIB = 1_024`,
   binary,
   IEC 80000-13.
- `BYTES_PER_KB = 1_000`,
   decimal,
   SI.

Earlier callers used `BYTES_PER_KB = 1_024`,
 which conflated the two.
When migrating an inline `BYTES_PER_KB = 1_024` definition,
rename to `BYTES_PER_KIB`.

## Design decisions

- Source-only package.
  The package has no build step;
   consumers import directly from `src/index.ts`.
  Constants do not benefit from bundling,
   and skipping the build keeps the workspace install fast.
- Universal/spec scope instead of a global const registry.
  A constant belongs here when its authority is an external spec or context-free relationship.
  Values owned by one package stay in that package even when their spelling is reusable.
- Conservative inclusion.
  Add constants after a second use site,
   or when one concrete consumer needs a spec-defined primitive
  that has no app or domain owner.
  Speculative categories stay in the consuming package until a real extraction point appears.
- Composed values use named primitives,
   not literals.
  `MS_PER_HOUR` is `MS_PER_MINUTE * MINUTES_PER_HOUR`,
   not `3_600_000`.
  This makes corrections propagate automatically and documents the relationship.
- Per-category sibling files.
  `ascii.ts`,
   `time.ts`,
   `byte.ts`,
   `http-status.ts`,
   and `fraction.ts` each own their domain.
  `index.ts` is pure re-exports.
  New categories arrive as new sibling files,
   not as additions to one growing file.
