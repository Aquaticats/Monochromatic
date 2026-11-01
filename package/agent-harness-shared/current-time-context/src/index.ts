/**
 Shared formatter for hidden current-time context payloads.
 
 @module
 */

//region Constants

/**
 Width used for zero-padded 24-hour clock fields.
 */
const CLOCK_FIELD_WIDTH = 2;

/**
 Opening XML-like tag wrapped around hidden wall-clock context.
 */
const TIME_CONTEXT_OPEN_TAG = '<time>';

/**
 Closing XML-like tag wrapped around hidden wall-clock context.
 */
const TIME_CONTEXT_CLOSE_TAG = '</time>';

//endregion Constants

//region Formatting

/**
 Pads local clock fields to two decimal digits.
 
 @param value - non-negative clock field value below 100
 
 @returns decimal text with leading zero when needed
 
 @example
 ```ts
 padTwoDigits(7);
 // '07'
 ```
 */
function padTwoDigits(value: number,): string {
  return value.toString()
    .padStart(
    CLOCK_FIELD_WIDTH,
    '0',
    );
}

/**
 Formats a {@link Date} as hidden current-time context using local wall-clock time,
 zero-padding each field with {@link padTwoDigits}.
 
 The output intentionally carries only hour and minute. It omits seconds,
 date, and timezone so agents receive coarse local time without unrelated
 temporal details.
 
 @param now - timestamp to format through local 24-hour clock fields
 
 @returns `<time>HH:MM</time>` with zero-padded hour and minute
 
 @example
 ```ts
 formatTimeContext(new Date(2_026, 4, 1, 20, 48));
 // '<time>20:48</time>'
 ```
 */
function formatTimeContext(now: Date,): string {
  return `${TIME_CONTEXT_OPEN_TAG}${padTwoDigits(now.getHours(),)}:${
    padTwoDigits(now.getMinutes(),)
  }${TIME_CONTEXT_CLOSE_TAG}`;
}

//endregion Formatting

export { formatTimeContext, };
