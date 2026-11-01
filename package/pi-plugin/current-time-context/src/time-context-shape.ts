/**
 Shape checks for current-time context payloads.
 
 @module
 */

//region Constants

/**
 Prefix opening hidden time context tags.
 */
const TIME_CONTEXT_OPEN_TAG = '<time>';

/**
 Suffix closing hidden time context tags.
 */
const TIME_CONTEXT_CLOSE_TAG = '</time>';

/**
 Template fixing exact hidden time context length.
 */
const TIME_CONTEXT_TEMPLATE = '<time>HH:MM</time>';

/**
 Index of the first hour digit inside `HH:MM`.
 */
const FIRST_HOUR_DIGIT_INDEX = 0;

/**
 Index of the second hour digit inside `HH:MM`.
 */
const SECOND_HOUR_DIGIT_INDEX = 1;

/**
 Index of the separator inside `HH:MM`.
 */
const TIME_SEPARATOR_INDEX = 2;

/**
 Index of the first minute digit inside `HH:MM`.
 */
const FIRST_MINUTE_DIGIT_INDEX = 3;

/**
 Index of the second minute digit inside `HH:MM`.
 */
const SECOND_MINUTE_DIGIT_INDEX = 4;

//endregion Constants

//region Shape checks

/**
 Determines whether a string is one ASCII decimal digit.
 
 @param value - single character to check
 
 @returns whether value is between `0` and `9`
 
 @example
 ```typescript
 isAsciiDigit('7');
 // true
 ```
 */
function isAsciiDigit(value: string,): boolean {
  return (value >= '0') && (value <= '9');
}

/**
 Determines whether content has exact `<time>HH:MM</time>` shape.
 
 @param value - candidate hidden current-time context content
 
 @returns whether value matches bounded time context shape
 
 @example
 ```typescript
 isTimeContextContent('<time>07:05</time>');
 // true
 ```
 */
function isTimeContextContent(value: string,): boolean {
  if (value.length
    !== TIME_CONTEXT_TEMPLATE
      .length)
    return false;
  if (!value.startsWith(TIME_CONTEXT_OPEN_TAG,))
    return false;
  if (!value.endsWith(TIME_CONTEXT_CLOSE_TAG,))
    return false;

  /**
   Middle `HH:MM` segment inside the time context tags.
   */
  const middle = value.slice(
    TIME_CONTEXT_OPEN_TAG.length,
    value.length
      - TIME_CONTEXT_CLOSE_TAG
        .length,
  );

  return isAsciiDigit(middle.charAt(FIRST_HOUR_DIGIT_INDEX,),)
    && isAsciiDigit(middle.charAt(SECOND_HOUR_DIGIT_INDEX,),)
    && (middle.charAt(TIME_SEPARATOR_INDEX,)
      === ':')
    && isAsciiDigit(middle.charAt(FIRST_MINUTE_DIGIT_INDEX,),)
    && isAsciiDigit(middle.charAt(SECOND_MINUTE_DIGIT_INDEX,),);
}

//endregion Shape checks

export { isTimeContextContent, };
