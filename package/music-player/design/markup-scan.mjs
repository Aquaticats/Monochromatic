/**
 Regex-free scans over the markup, hierarchy XML and generated pages this package validates.

 Each scan mirrors one regular expression that used to sit at a call site, with the same shape and
 the same verdicts: the audit recorded in `doc/audit/regex-necessity.md` asks for the pattern to go
 where a string scan answers the same question.

 - `countTagStarts` replaces a global element-name match with a trailing word boundary.
 - `tagElements` replaces a global element match that runs to the next `>`.
 - `hasQuotedAttributeValue` replaces an attribute test that requires a non-empty value.
 - `boundsRectangle` replaces the four-number `bounds` extraction from a hierarchy node.
 - `hasRequiredControlMarker` replaces the textarea-or-select required-attribute test.
 - `hasExternalSubresourceReference` replaces the external script-or-stylesheet test.
 - `hasClipboardUse` replaces the clipboard API test.

 Case folding follows the engine's rule for a case-insensitive match without the `u` flag: only ASCII
 letters fold, because a code unit of 128 or more keeps itself or takes a non-ASCII uppercase form
 and so never equals an ASCII letter of a scanned literal. Word boundaries use the engine's word
 character: ASCII letters, digits and underscore. Whitespace uses the engine's `\s` set: the
 WhiteSpace and LineTerminator code points.
 */

//region Character classes

/**
 Code unit of the first and last ASCII uppercase letter, in order.
 */
const ASCII_UPPER_FIRST = 'A'.charCodeAt(0);
const ASCII_UPPER_LAST = 'Z'.charCodeAt(0);

/**
 Code unit of the first and last ASCII lowercase letter, in order.
 */
const ASCII_LOWER_FIRST = 'a'.charCodeAt(0);
const ASCII_LOWER_LAST = 'z'.charCodeAt(0);

/**
 Distance between an ASCII uppercase letter and its lowercase counterpart.
 */
const ASCII_CASE_OFFSET = ASCII_LOWER_FIRST - ASCII_UPPER_FIRST;

/**
 Code unit of the first and last ASCII digit, in order.
 */
const DIGIT_FIRST = '0'.charCodeAt(0);
const DIGIT_LAST = '9'.charCodeAt(0);

/**
 Code units of the punctuation the scanned grammars name.
 */
const DOUBLE_QUOTE = '"'.charCodeAt(0);
const SINGLE_QUOTE = '\''.charCodeAt(0);
const LESS_THAN = '<'.charCodeAt(0);
const GREATER_THAN = '>'.charCodeAt(0);
const EQUALS = '='.charCodeAt(0);
const UNDERSCORE = '_'.charCodeAt(0);

/**
 Whitespace code points of the engine's `\s` set outside the U+2000 to U+200A space separator run.
 */
const WHITESPACE_CODE_POINTS = new Set([
  0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x0020, 0x00A0, 0x1680, 0x2028, 0x2029, 0x202F, 0x205F,
  0x3000, 0xFEFF,
]);

/**
 First and last space separator code points that the `\s` set covers as a contiguous run.
 */
const SPACE_SEPARATOR_FIRST = 0x2000;
const SPACE_SEPARATOR_LAST = 0x200A;

/**
 Value returned when a scan finds no occurrence, matching the index a failed string search returns.
 */
const NOT_FOUND = -1;

//endregion Character classes

//region Character predicates

/**
 Whether a code unit is one the engine's word boundary treats as a word character.

 @param codeUnit - code unit read from the scanned text, or NaN past its end

 @returns whether the code unit is an ASCII letter, digit or underscore
 */
function isWordCodeUnit(codeUnit) {
  if ((codeUnit >= ASCII_UPPER_FIRST) && (codeUnit <= ASCII_UPPER_LAST)) return true;
  if ((codeUnit >= ASCII_LOWER_FIRST) && (codeUnit <= ASCII_LOWER_LAST)) return true;
  if ((codeUnit >= DIGIT_FIRST) && (codeUnit <= DIGIT_LAST)) return true;
  return codeUnit === UNDERSCORE;
}

/**
 Whether a code unit belongs to the engine's `\s` set.

 @param codeUnit - code unit read from the scanned text, or NaN past its end

 @returns whether the code unit is ECMAScript whitespace or a line terminator
 */
function isWhitespaceCodeUnit(codeUnit) {
  if ((codeUnit >= SPACE_SEPARATOR_FIRST) && (codeUnit <= SPACE_SEPARATOR_LAST)) return true;
  return WHITESPACE_CODE_POINTS.has(codeUnit);
}

/**
 Whether a code unit is an ASCII digit.

 @param codeUnit - code unit read from the scanned text, or NaN past its end

 @returns whether the code unit is `0` through `9`
 */
function isDigitCodeUnit(codeUnit) {
  return (codeUnit >= DIGIT_FIRST) && (codeUnit <= DIGIT_LAST);
}

/**
 Folds one ASCII uppercase letter to lowercase and leaves every other code unit alone.

 The scanned literals are ASCII and lowercase, and a code unit of 128 or more never folds onto an
 ASCII letter, so this reproduces a case-insensitive match without the `u` flag.

 @param codeUnit - code unit read from the scanned text

 @returns the code unit to compare against a literal's own code unit
 */
function foldAsciiCodeUnit(codeUnit) {
  if ((codeUnit >= ASCII_UPPER_FIRST) && (codeUnit <= ASCII_UPPER_LAST)) {
    return codeUnit + ASCII_CASE_OFFSET;
  }
  return codeUnit;
}

//endregion Character predicates

//region Literal scans

/**
 Whether a lowercase ASCII literal occurs at `index`, comparing without case.

 @param text - text being scanned

 @param literal - lowercase ASCII literal to compare against

 @param index - position where the literal would begin

 @returns whether the folded code units of the literal match the text at that position
 */
function matchesFoldedLiteral({
  text,
  literal,
  index
}) {
  if (index < 0) return false;
  if ((index + literal.length) > text.length) return false;
  for (let offset = 0; offset < literal.length; offset += 1) {
    if (foldAsciiCodeUnit(text.charCodeAt(index + offset)) !== literal.charCodeAt(offset)) {
      return false;
    }
  }
  return true;
}

/**
 Index of the first occurrence of a lowercase ASCII literal at or after `from`, ignoring case.

 @param text - text being scanned

 @param literal - lowercase ASCII literal to look for

 @param from - position to resume the scan at

 @param until - position the literal must end before, defaulting to the end of the text

 @returns zero-based index of the occurrence, or -1 when there is none
 */
function indexOfFoldedLiteral({
  text,
  literal,
  from,
  until = text.length
}) {
  const lastStart = Math.min(until, text.length) - literal.length;
  for (let index = Math.max(from, 0); index <= lastStart; index += 1) {
    if (matchesFoldedLiteral({
      text,
      literal,
      index
    })) return index;
  }
  return NOT_FOUND;
}

/**
 Reads the run of ASCII digits starting at `index`.

 @param text - text being scanned

 @param index - position the run would begin at

 @returns the run's numeric value and the position after it, or undefined when no digit is there
 */
function digitRunAt({
  text,
  index
}) {
  let position = index;
  while (isDigitCodeUnit(text.charCodeAt(position))) position += 1;
  if (position === index) return undefined;
  return {
    value: Number(text.slice(index, position)),
    nextIndex: position
  };
}

//endregion Literal scans

//region Element scans

/**
 Counts `<name` occurrences whose name is followed by a word boundary, ignoring case.

 `<textareatextarea` holds one such occurrence and not two, because the character after the first
 name is a word character. Two matches cannot overlap here: a later match would have to start on the
 literal's own second character, which is a letter rather than `<`.

 @param text - markup or hierarchy text being scanned

 @param name - element name without its opening angle bracket

 @returns number of occurrences, zero when the element is absent
 */
function countTagStarts({
  text,
  name
}) {
  const opening = `<${name}`;
  let count = 0;
  let from = 0;
  for (;;) {
    const index = indexOfFoldedLiteral({
      text,
      literal: opening,
      from
    });
    if (index === NOT_FOUND) return count;
    if (!isWordCodeUnit(text.charCodeAt(index + opening.length))) count += 1;
    from = index + 1;
  }
}

/**
 Reads every `<name ...>` element from text, exact case, each running to the first following `>`.

 An occurrence whose name is followed by a word character is skipped, and one with no closing `>`
 ahead ends the scan because no later occurrence has one either.

 @param text - markup or hierarchy text being scanned

 @param name - element name without its opening angle bracket

 @returns element text including both angle brackets, empty when the element is absent
 */
function tagElements({
  text,
  name
}) {
  const opening = `<${name}`;
  const elements = [];
  let from = 0;
  for (;;) {
    const index = text.indexOf(opening, from);
    if (index === NOT_FOUND) return elements;
    if (isWordCodeUnit(text.charCodeAt(index + opening.length))) {
      from = index + 1;
      continue;
    }
    const close = text.indexOf('>', index + opening.length);
    if (close === NOT_FOUND) return elements;
    elements.push(text.slice(index, close + 1));
    from = close + 1;
  }
}

/**
 Whether text holds `attribute="` followed by a non-empty value and a closing quote, exact case.

 @param text - markup text being scanned

 @param attribute - attribute name without its quote or equals sign

 @returns whether a non-empty quoted value for the attribute is present
 */
function hasQuotedAttributeValue({
  text,
  attribute
}) {
  const opening = `${attribute}="`;
  let from = 0;
  for (;;) {
    const index = text.indexOf(opening, from);
    if (index === NOT_FOUND) return false;
    const valueStart = index + opening.length;
    const close = text.indexOf('"', valueStart);
    if (close === NOT_FOUND) return false;
    if (close > valueStart) return true;
    from = index + 1;
  }
}

//endregion Element scans

//region Hierarchy scans

/**
 Reads the `bounds="[left,top][right,bottom]"` rectangle out of a hierarchy node.

 Any occurrence is accepted, so a node may carry the rectangle after another attribute. A run that
 starts the shape but fails it makes the scan resume at the next `bounds="[` occurrence.

 @param text - hierarchy node text, or any text that may carry one rectangle

 @returns the four corner values, or undefined when no well-formed rectangle is present
 */
function boundsRectangle({ text }) {
  const opening = 'bounds="[';
  let from = 0;
  for (;;) {
    const index = text.indexOf(opening, from);
    if (index === NOT_FOUND) return undefined;
    const rectangle = rectangleAt({
      text,
      index: index + opening.length
    });
    if (rectangle !== undefined) return rectangle;
    from = index + 1;
  }
}

/**
 Reads one corner value and the separator that must follow it.

 @param text - hierarchy node text

 @param index - position of the corner value

 @param separator - separator expected after the value

 @returns the corner value and the position after the separator, or undefined
 */
function cornerAndSeparator({
  text,
  index,
  separator
}) {
  const run = digitRunAt({
    text,
    index
  });
  if (run === undefined) return undefined;
  if (!text.startsWith(separator, run.nextIndex)) return undefined;
  return {
    value: run.value,
    nextIndex: run.nextIndex + separator.length
  };
}

/**
 Reads the four corner values and separators that follow `bounds="[`.

 @param text - hierarchy node text

 @param index - position of the first corner value

 @returns the four corner values, or undefined when the rectangle is malformed
 */
function rectangleAt({
  text,
  index
}) {
  const left = cornerAndSeparator({
    text,
    index,
    separator: ','
  });
  if (left === undefined) return undefined;
  const top = cornerAndSeparator({
    text,
    index: left.nextIndex,
    separator: ']['
  });
  if (top === undefined) return undefined;
  const right = cornerAndSeparator({
    text,
    index: top.nextIndex,
    separator: ','
  });
  if (right === undefined) return undefined;
  const bottom = cornerAndSeparator({
    text,
    index: right.nextIndex,
    separator: ']'
  });
  if (bottom === undefined) return undefined;
  if (text.charAt(bottom.nextIndex) !== '"') return undefined;
  return {
    left: left.value,
    top: top.value,
    right: right.value,
    bottom: bottom.value
  };
}

//endregion Hierarchy scans

//region Generated-page scans

/**
 Element names whose required attribute makes a review demand an answer.
 */
const REQUIRED_CONTROL_NAMES = ['textarea', 'select'];

/**
 Attribute word that marks a control required.
 */
const REQUIRED_WORD = 'required';

/**
 Whether a review holds a textarea or select element that marks itself required, ignoring case.

 The element name needs a word boundary after it, then attribute positions are walked one token at
 a time: a quoted value is skipped whole, an angle bracket ends the element, and any other character
 is one position. At each position a whitespace run, the `required` word and one of whitespace, `=`
 or `>` must follow, which is what the pattern allowed between the element name and the attribute.

 @param text - generated page text being scanned

 @returns whether such an element is present
 */
function hasRequiredControlMarker({ text }) {
  for (const name of REQUIRED_CONTROL_NAMES) {
    let from = 0;
    for (;;) {
      const index = indexOfFoldedLiteral({
        text,
        literal: `<${name}`,
        from
      });
      if (index === NOT_FOUND) break;
      const afterName = index + name.length + 1;
      if (!isWordCodeUnit(text.charCodeAt(afterName))
        && elementHoldsRequiredWord({
          text,
          afterName
        })) return true;
      from = index + 1;
    }
  }
  return false;
}

/**
 Walks one element's attribute positions looking for the `required` word at a boundary.

 @param text - generated page text being scanned

 @param afterName - position immediately after the element name

 @returns whether the word appears where an attribute would start
 */
function elementHoldsRequiredWord({
  text,
  afterName
}) {
  let position = afterName;
  for (;;) {
    if (requiredWordStartsAt({
      text,
      position
    })) return true;
    const value = attributeTokenEnd({
      text,
      position
    });
    if (value === undefined) return false;
    position = value;
  }
}

/**
 Whether a whitespace run, the `required` word and a following delimiter start at `position`.

 @param text - generated page text being scanned

 @param position - position to test

 @returns whether the word appears as an attribute here
 */
function requiredWordStartsAt({
  text,
  position
}) {
  if (!isWhitespaceCodeUnit(text.charCodeAt(position))) return false;
  let wordStart = position;
  while (isWhitespaceCodeUnit(text.charCodeAt(wordStart))) wordStart += 1;
  if (!matchesFoldedLiteral({
    text,
    literal: REQUIRED_WORD,
    index: wordStart
  })) return false;
  const afterWord = text.charCodeAt(wordStart + REQUIRED_WORD.length);
  return isWhitespaceCodeUnit(afterWord)
    || (afterWord === EQUALS)
    || (afterWord === GREATER_THAN);
}

/**
 Reads one attribute token, ending the element at an angle bracket or an unterminated quote.

 @param text - generated page text being scanned

 @param position - position the token starts at

 @returns position after the token, or undefined when the element cannot be walked further
 */
function attributeTokenEnd({
  text,
  position
}) {
  const codeUnit = text.charCodeAt(position);
  if (Number.isNaN(codeUnit)) return undefined;
  if ((codeUnit === LESS_THAN) || (codeUnit === GREATER_THAN)) return undefined;
  if (codeUnit === DOUBLE_QUOTE) {
    const close = text.indexOf('"', position + 1);
    return (close === NOT_FOUND) ? undefined : close + 1;
  }
  if (codeUnit === SINGLE_QUOTE) {
    const close = text.indexOf('\'', position + 1);
    return (close === NOT_FOUND) ? undefined : close + 1;
  }
  return position + 1;
}

/**
 Element names and the external-resource attribute each one may carry.
 */
const SUBRESOURCE_ELEMENTS = [
  {
    name: 'script',
    attribute: 'src='
  },
  {
    name: 'link',
    attribute: 'href='
  },
];

/**
 Whether a review loads a script or stylesheet from outside itself, ignoring case.

 The element name needs whitespace after it, then the attribute must appear before the element's
 closing `>`, so an attribute of a later element cannot satisfy an earlier one.

 @param text - generated page text being scanned

 @returns whether an external subresource is referenced
 */
function hasExternalSubresourceReference({ text }) {
  return SUBRESOURCE_ELEMENTS.some((element) => elementHoldsAttribute({
    text,
    ...element
  }));
}

/**
 Whether one element name carries its external-resource attribute before its closing bracket.

 @param text - generated page text being scanned

 @param name - element name without its opening angle bracket

 @param attribute - attribute and equals sign to look for

 @returns whether the attribute appears inside that element
 */
function elementHoldsAttribute({
  text,
  name,
  attribute
}) {
  const opening = `<${name}`;
  let from = 0;
  for (;;) {
    const index = indexOfFoldedLiteral({
      text,
      literal: opening,
      from
    });
    if (index === NOT_FOUND) return false;
    const whitespace = index + opening.length;
    if (isWhitespaceCodeUnit(text.charCodeAt(whitespace))) {
      const close = text.indexOf('>', whitespace);
      const until = (close === NOT_FOUND) ? text.length : close;
      if (indexOfFoldedLiteral({
        text,
        literal: attribute,
        from: whitespace + 1,
        until
      }) !== NOT_FOUND) return true;
    }
    from = index + 1;
  }
}

/**
 Clipboard API names that a review must not carry.
 */
const CLIPBOARD_LITERALS = ['navigator.clipboard', 'clipboardData'];

/**
 Property access whose copy command a review must not carry.
 */
const EXEC_COMMAND_OPENING = 'execCommand(';

/**
 Command word that marks a clipboard write.
 */
const COPY_WORD = 'copy';

/**
 Whether a review reads or writes the clipboard, exact case.

 @param text - generated page text being scanned

 @returns whether a clipboard API is called
 */
function hasClipboardUse({ text }) {
  if (CLIPBOARD_LITERALS.some((literal) => text.includes(literal))) return true;
  let from = 0;
  for (;;) {
    const index = text.indexOf(EXEC_COMMAND_OPENING, from);
    if (index === NOT_FOUND) return false;
    let position = index + EXEC_COMMAND_OPENING.length;
    while (isWhitespaceCodeUnit(text.charCodeAt(position))) position += 1;
    const quote = text.charCodeAt(position);
    if (((quote === DOUBLE_QUOTE) || (quote === SINGLE_QUOTE))
      && text.startsWith(COPY_WORD, position + 1)) return true;
    from = index + 1;
  }
}

//endregion Generated-page scans

export {
  boundsRectangle,
  countTagStarts,
  hasClipboardUse,
  hasExternalSubresourceReference,
  hasQuotedAttributeValue,
  hasRequiredControlMarker,
  tagElements,
};
