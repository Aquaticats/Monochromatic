/**
 Stage one's TOML reader: one `RawRule` per `[[rules]]` entry of the vendored
 betterleaks configuration.

 Reads the whole configuration text. The entry point in `port-betterleaks.ts`
 renders each returned rule through `pcreToResharp`.
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { tripleQuotedBodyAfterAssignment, } from './port-betterleaks-field-line.ts';

/**
 One rule extracted from the upstream TOML, before conversion.
 
 `pathScope` and `secretGroup` are optional properties because the upstream
 config may omit them; every other field is required for a rule to be emitted.
 */
export type RawRule = {
  readonly id: string;
  readonly description: string;
  readonly regex: string;
  readonly pathScope?: string;
  readonly secretGroup?: number;
  readonly skipReport: boolean;
  readonly hasRequired: boolean;
};

/**
 The same rule while the parser is still attaching the optional fields.
 
 A rule leaves the parser as a {@link RawRule}; this shape exists only so the
 extractors can attach `pathScope` and `secretGroup` after construction.
 */
type MutableRule = {
  id: string;
  description: string;
  regex: string;
  pathScope?: string;
  secretGroup?: number;
  skipReport: boolean;
  hasRequired: boolean;
};

/**
 The per-rule fields while the extractors are still filling them in.
 
 A rule whose `id`, `description` or `regex` never arrived is dropped instead
 of being emitted half-filled, which is why every member starts absent.
 */
type RuleFields = {
  id?: string;
  description?: string;
  regex?: string;
  pathScope?: string;
  secretGroup?: number;
};

/**
 Attaches the optional upstream fields to the required ones.
 
 @param fields - the fields the extractors matched, with the required three in place
 
 @param skipReport - upstream `skipReport = true` marker, which suppresses the rule
 
 @param hasRequired - whether the rule body carried a `[[rules.required]]` note
 
 @returns the finished rule every consumer reads
 */
function ruleFromFields({
  fields,
  skipReport,
  hasRequired,
}: {
  readonly fields: Readonly<RuleFields>;
  readonly skipReport: boolean;
  readonly hasRequired: boolean;
}): RawRule {
  /**
   The rule under construction; the optional upstream fields are attached below.
   */
  const rule: MutableRule = {
    id: nonNullishOrThrow(fields.id,),
    description: nonNullishOrThrow(fields.description,),
    regex: nonNullishOrThrow(fields.regex,),
    skipReport,
    hasRequired,
  };
  if (fields.pathScope !== undefined)
    rule.pathScope = fields.pathScope;
  if (fields.secretGroup !== undefined)
    rule.secretGroup = fields.secretGroup;
  return rule;
}

/**
 Parse the betterleaks TOML and yield the subset of fields we care about,
 one {@link RawRule} per parsed entry.
 
 @param toml - verbatim contents of the vendored betterleaks configuration
 
 @returns every declared rule, in file order, carrying the fields this port keeps
 
 @example
 ```ts
 const rules = parseRules({ toml: await readFile(TOML_PATH, 'utf8') });
 console.log(rules.length);
 ```
 */
export function parseRules({ toml, }: { readonly toml: string; },): readonly RawRule[] {
  /**
   Line-split TOML so the parser can step line-by-line through the cursor.
   */
  const lines = toml.split('\n',);
  /**
   Accumulator that collects every successfully parsed rule.
   */
  const out: RawRule[] = [];
  /**
   Cursor into `lines`, shared with the nested body reader that advances it.
   
   One mutable object rather than a binding at the function root: the reader
   moves the same position the loop reads, so both name one value.
   */
  const cursor: { at: number } = { at: 0 };

  /** Read a triple-quoted string starting at the cursor's line after the `=`.
    Advances the cursor past the closing `'''` line.
   
    @param initial - remainder of the opening line after `'''`
   
    @returns the body with its surrounding `'''` removed */
  function readTripleQuoted({ initial, }: { readonly initial: string; },): string {
    // initial is the substring after `'''` on the opening line.
    if (initial.includes("'''",)) {
      /**
       Position of the closing `'''` on the same line as the opener.
       */
      const end = initial.indexOf("'''",);
      cursor.at += 1;
      return initial.slice(
        0,
        end,
      );
    }
    /**
     Accumulator for the multi-line body; first slot holds the opening-line remainder.
     */
    const parts: string[] = [initial,];
    cursor.at += 1;
    while (cursor.at < lines
      .length) {
      /**
       Line inside the multi-line body, tested for the closing `'''`.
       */
      const bodyLine = nonNullishOrThrow(lines[cursor.at],);
      if (bodyLine.includes("'''",))
        break;
      parts.push(bodyLine,);
      cursor.at += 1;
    }
    if (cursor.at < lines
      .length) {
      /**
       Closing line containing the terminating `'''`.
       */
      const close = nonNullishOrThrow(lines[cursor.at],);
      /**
       Position of `'''` inside `close`, used to trim the trailing content.
       */
      const end = close.indexOf("'''",);
      parts.push(close.slice(
        0,
        end,
      ),);
      cursor.at += 1;
    }
    return parts.join('\n',);
  }

  while (cursor.at < lines
    .length) {
    /**
     Current TOML line at the cursor, tested for table headers.
     */
    const line = nonNullishOrThrow(lines[cursor.at],);
    cursor.at += 1;
    // Look for top-level rule opener `[[rules]]` (not `[[rules.something]]`).
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML table header line, anchored at both ends, matching escaped literal brackets and `\s*`; the scan is linear and cannot backtrack.
    if (!/^\[\[rules\]\]\s*$/u.test(line,))
      continue;
    /**
     Per-rule fields, filled by the extractors below; a rule missing its
     `id`, `description` or `regex` is dropped rather than emitted half-filled.
     */
    const fields: RuleFields = {};
    /**
     Upstream `skipReport = true` marker; suppresses this rule from emission.
     */
    let skipReport = false;
    /**
     Set when the rule body contains `[[rules.required]]`; preserved as a note.
     */
    let hasRequired = false;
    // Scan rule body until the next top-level table marker.
    while (cursor.at < lines
      .length) {
      /**
       Current line inside the rule body.
       */
      const rl = nonNullishOrThrow(lines[cursor.at],);
      // Sub-tables of the current rule: `[[rules.required]]` /
      // `[[rules.allowlists]]`. Mark `hasRequired` when applicable
      // and skip through their bodies.
      /**
       Capture of any `[[rules.<name>]]` sub-table header at the cursor.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML sub-table header line, anchored at both ends, matching escaped literal brackets around one word; the scan is linear and cannot backtrack.
      const subTableMatch = /^\[\[rules\.(?<name>[\w]+)\]\]\s*$/u.exec(rl,);
      if (subTableMatch !== null) {
        /**
         The sub-table name, the part of the header after `rules.`.
         */
        const subTableName = nonNullishOrThrow(subTableMatch.groups,);
        if (subTableName?.name === 'required')
          hasRequired = true;
        cursor.at += 1;
        while (cursor.at < lines
          .length) {
          /**
           Line inside the sub-table body, tested for its end.
           */
          const subTableLine = nonNullishOrThrow(lines[cursor.at],);
          if (subTableLine.startsWith("[",))
            break;
          cursor.at += 1;
        }
        continue;
      }
      // Top-level table marker (next `[[rules]]` or `[other]`): end of rule.
      if (rl.startsWith('[',))
        break;
      // Field extractors. Each consumes its own lines via the cursor.
      /**
       Match for the `id = "..."` field.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML field line, anchored at `^`, matching a literal key, `\s*`, `=` and one quoted run; the scan is linear and cannot backtrack.
      const idMatch = /^id\s*=\s*"(?<value>[^"]*)"/u.exec(rl,);
      if (idMatch !== null) {
        /**
         The captured `id` value of this rule.
         */
        const idValue = nonNullishOrThrow(idMatch.groups,);
        fields.id = nonNullishOrThrow(idValue?.value,);
        cursor.at += 1;
        continue;
      }
      /**
       Match for the `description = "..."` field; capture handles `\"` escapes.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML field line, anchored at `^`, matching a literal key and one quoted run whose alternation consumes one character or one escape at a time; every step advances one character, so the scan is linear.
      const descriptionMatch = /^description\s*=\s*"(?<value>(?:[^"\\]|\\.)*)"/u.exec(rl,);
      if (descriptionMatch !== null) {
        /**
         The captured description, still carrying its `\"` escapes.
         */
        const descriptionValue = nonNullishOrThrow(descriptionMatch.groups,);
        /**
         Upstream description with its escapes decoded.
         */
        const escaped = nonNullishOrThrow(descriptionValue?.value,);
        fields.description = escaped.replaceAll(
          String.raw`\"`,
          '"',
        );
        cursor.at += 1;
        continue;
      }
      /**
       Body the `regex = '''` line opens, with its marker removed.
       */
      const regexBody = tripleQuotedBodyAfterAssignment({
        line: rl,
        key: 'regex',
      },);
      if ((typeof regexBody) === 'string') {
        fields.regex = readTripleQuoted({ initial: regexBody, },);
        continue;
      }
      /**
       Body the `path = '''` line opens, with its marker removed.
       */
      const pathBody = tripleQuotedBodyAfterAssignment({
        line: rl,
        key: 'path',
      },);
      if ((typeof pathBody) === 'string') {
        fields.pathScope = readTripleQuoted({ initial: pathBody, },);
        continue;
      }
      /**
       Match for the optional `secretGroup = N` capture-group selector.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML field line, anchored at `^`, matching a literal key and one digit run; the scan is linear and cannot backtrack.
      const secretGroupMatch = /^secretGroup\s*=\s*(?<value>\d+)/u.exec(rl,);
      if (secretGroupMatch !== null) {
        /**
         The captured `secretGroup` value of this rule.
         */
        const secretGroupValue = nonNullishOrThrow(secretGroupMatch.groups,);
        fields.secretGroup = Number(nonNullishOrThrow(secretGroupValue?.value,),);
        cursor.at += 1;
        continue;
      }
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- one TOML field line, anchored at `^`, matching a literal key and the literal `true`; the scan is linear and cannot backtrack.
      if (/^skipReport\s*=\s*true/u.test(rl,)) {
        skipReport = true;
        cursor.at += 1;
        continue;
      }
      cursor.at += 1;
    }
    if (
      (fields.id !== undefined)
        && (fields.description !== undefined)
        && (fields.regex !== undefined)
    ) {
      out.push(ruleFromFields({
        fields,
        hasRequired,
        skipReport,
      },),);
    }
  }
  return out;
}
