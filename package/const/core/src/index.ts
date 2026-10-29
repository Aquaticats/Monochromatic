/**
 Universal constants for the Monochromatic monorepo.
 
 This package is the canonical home for values that express mathematical or
 physical relationships (time ratios, byte units, fractions), code sets
 defined by external specifications (HTTP status codes from RFC 9110), and
 context-free character sets (ASCII letters and digits). App-specific
 defaults (timeouts, ports, z-index tiers) are deliberately excluded so
 unrelated apps stay decoupled.
 
 @example
 ```ts
 import {
   ASCII_LOWERCASE_ALPHANUMERIC_CHARS,
   BYTES_PER_KIB,
   HALF,
   HTTP_NOT_FOUND,
   MS_PER_SECOND,
 } from '@monochromatic-dev/const-core';
 ```
 
 @packageDocumentation
 */

//region ascii

export {
  ASCII_DECIMAL_DIGIT_CHARS,
  ASCII_LOWERCASE_ALPHANUMERIC_CHARS,
  ASCII_LOWERCASE_LETTER_CHARS,
} from './ascii.ts';

//endregion ascii

//region time

export {
  DAYS_PER_WEEK,
  DAYS_PER_YEAR,
  HOURS_PER_DAY,
  MINUTES_PER_HOUR,
  MONTHS_PER_YEAR,
  MS_PER_DAY,
  MS_PER_HOUR,
  MS_PER_MINUTE,
  MS_PER_SECOND,
  MS_PER_WEEK,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from './time.ts';

//endregion time

//region byte

export {
  BITS_PER_BYTE,
  BYTES_PER_GB,
  BYTES_PER_GIB,
  BYTES_PER_KB,
  BYTES_PER_KIB,
  BYTES_PER_MB,
  BYTES_PER_MIB,
  BYTES_PER_TB,
  BYTES_PER_TIB,
} from './byte.ts';

//endregion byte

//region http-status

export {
  HTTP_BAD_REQUEST,
  HTTP_CONFLICT,
  HTTP_CREATED,
  HTTP_FORBIDDEN,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NO_CONTENT,
  HTTP_NOT_FOUND,
  HTTP_NOT_MODIFIED,
  HTTP_OK,
  HTTP_UNAUTHORIZED,
} from './http-status.ts';

//endregion http-status

//region fraction

export {
  HALF,
  QUARTER,
  THIRD,
  THREE_QUARTERS,
  TWO_THIRDS,
} from './fraction.ts';

//endregion fraction
