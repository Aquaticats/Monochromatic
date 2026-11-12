/**
 Reads the `need({ rule, holds, detail });` statements out of this package's builder sources.

 The Settings-pane consumer test deletes one rule from a copy of the builder and expects its own case
 to stop reporting the name, so it needs each rule's statement span as well as the list of names.
 The builder is a plain script with no export surface, so both are read from its source text: a
 statement opens on a line holding nothing but `need({` and closes on the next line holding nothing
 but `});`. A builder reshaped away from that, or a statement left unterminated, fails here instead
 of quietly leaving a rule unexercised.
 */

//region Constants

/**
 Line that opens one rule statement, once surrounding indentation is dropped.
 */
const CALL_OPENING_LINE = 'need({';

/**
 Line that closes one rule statement, once surrounding indentation is dropped.
 */
const CALL_CLOSING_LINE = '});';

/**
 Property whose value is the rule's name.
 */
const RULE_PROPERTY = 'rule:';

/**
 Value returned when a scan finds no occurrence, matching the index a failed string search returns.
 */
const NOT_FOUND = -1;

//endregion Constants

//region Character predicates

/**
 Whether a character separates tokens inside a line or across a line break.

 @param character - single character read from the source text

 @returns whether the character is a space, tab, carriage return or line feed
 */
function isSpaceCharacter(character) {
  return (character === ' ')
    || (character === '\t')
    || (character === '\r')
    || (character === '\n');
}

/**
 Whether a value is a rule name: lowercase ASCII letters and hyphens only.

 @param value - text read between the property's quotes

 @returns whether the value is a well-formed rule name
 */
function isRuleName(value) {
  if (value.length === 0) return false;
  for (const character of value) {
    if ((character === '-')) continue;
    if ((character >= 'a') && (character <= 'z')) continue;
    return false;
  }
  return true;
}

//endregion Character predicates

//region Statement scans

/**
 Splits source text into lines that remember where they start and end.

 @param source - builder source text

 @returns one entry per line, in source order, with the end of the last line at the text's end
 */
function lineSpans({ source }) {
  const spans = [];
  let start = 0;
  for (;;) {
    const end = source.indexOf('\n', start);
    if (end === NOT_FOUND) {
      spans.push({
        end: source.length,
        start,
        text: source.slice(start)
      });
      return spans;
    }
    spans.push({
      end,
      start,
      text: source.slice(start, end)
    });
    start = end + 1;
  }
}

/**
 Reads every rule statement, with the span it occupies so a copy can drop exactly that statement.

 @param source - builder source text

 @returns one entry per statement, in source order, with its rule name and its source offsets

 @throws when a statement is left unterminated, folded onto one line, or carries no rule name
 */
function needCalls({ source }) {
  const calls = [];
  let open;
  for (const span of lineSpans({ source })) {
    const trimmed = span.text.trim();
    if (open === undefined) {
      if (trimmed === CALL_OPENING_LINE) {
        open = { start: span.start };
        continue;
      }
      if (trimmed.startsWith(CALL_OPENING_LINE)) {
        throw new Error(`Settings-pane rule statement at offset ${span.start} is not a multi-line statement.`);
      }
      continue;
    }
    if (trimmed !== CALL_CLOSING_LINE) continue;
    const end = Math.min(span.end + 1, source.length);
    const text = source.slice(open.start, end);
    const rule = ruleNameOf({ text });
    if (rule === undefined) {
      throw new Error(`Settings-pane rule statement at offset ${open.start} has no rule name.`);
    }
    calls.push({
      end,
      rule,
      start: open.start,
      text
    });
    open = undefined;
  }
  if (open !== undefined) {
    throw new Error(`Settings-pane rule statement at offset ${open.start} is never closed.`);
  }
  return calls;
}

/**
 Reads the rule name out of one statement's text.

 @param text - a whole rule statement

 @returns the name between the property's quotes, or undefined when the property is absent
 */
function ruleNameOf({ text }) {
  const property = text.indexOf(RULE_PROPERTY);
  if (property === NOT_FOUND) return undefined;
  let position = property + RULE_PROPERTY.length;
  while (isSpaceCharacter(text.charAt(position))) position += 1;
  const quote = text.charAt(position);
  if ((quote !== '\'') && (quote !== '"')) return undefined;
  const close = text.indexOf(quote, position + 1);
  if (close === NOT_FOUND) return undefined;
  const name = text.slice(position + 1, close);
  return isRuleName(name) ? name : undefined;
}

/**
 Lists every rule name a builder declares, once each, in the order the statements appear.

 @param source - builder source text

 @returns rule names with duplicates collapsed
 */
function needRuleNames({ source }) {
  const names = [];
  for (const call of needCalls({ source })) {
    if (!names.includes(call.rule)) names.push(call.rule);
  }
  return names;
}

/**
 Returns a copy of the builder source with one rule's whole statement removed.

 @param source - builder source text

 @param rule - rule name whose statement is removed

 @returns source text without that statement

 @throws when no statement or more than one statement declares the rule
 */
function withoutNeedRule({
  source,
  rule
}) {
  const matching = needCalls({ source })
    .filter((call) => call.rule === rule);
  if (matching.length !== 1) {
    throw new Error(`Expected one Settings-pane rule named ${rule}; found ${matching.length}.`);
  }
  const [call] = matching;
  return source.slice(0, call.start) + source.slice(call.end);
}

//endregion Statement scans

export {
  needCalls,
  needRuleNames,
  withoutNeedRule,
};
