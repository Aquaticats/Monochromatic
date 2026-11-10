/**
 The answers file's shape, and the readers that enforce it.

 One file describes one recording:
 the block to insert,
 the heading it goes before,
 and every part of the file it replaces.

 @module
 */

import { AnswersError, } from './errors.ts';

/**
 Heading that starts a second-level section,
 which is where a record block belongs.
 */
export const SECTION_HEADING = '## ';

/**
 Headings that end a third-level section.
 */
export const SUBSECTION_HEADINGS: readonly string[] = [
  '### ',
  '## '
];

/**
 Lines a bullet's continuation may start with.
 */
export const BULLET_CONTINUATION = '  ';

/**
 One recording of the owner's answers.
 */
export type Answers = {
  readonly file: string;
  readonly record: string;
  readonly anchor: string;
  readonly remove: {
    readonly sections: readonly string[];
    readonly bullets: readonly string[];
    readonly lineRuns: readonly (readonly string[])[];
  };
};

/**
 A parsed JSON object, read-only so passing one on cannot mutate it.
 */
export type JsonObject = Readonly<Record<string, unknown>>;

/**
 Narrows an unknown value to a keyed record.
 
 @param value - Value read from the answers file.
 
 @returns True for a non-array object.
 
 @example
 ```ts
 if (!isRecord(parsed)) throw new AnswersError('the answers file must hold an object');
 ```
 */
export function isRecord(value: unknown,): value is JsonObject {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Refuses one field.
 
 @param at - Field path inside the answers file.
 
 @param expects - What the field must hold.
 
 @throws {@link AnswersError} naming the field.
 
 @example
 ```ts
 refuse({ at: 'record', expects: 'a non-empty string' });
 ```
 */
function refuse({
  at,
  expects,
}: {
  readonly at: string;
  readonly expects: string
},): never {
  throw new AnswersError(`The answers file's ${at} must be ${expects}.`,);
}

/**
 Reads one field as a non-empty string.
 
 @param value - Value read from the answers file.
 
 @param at - Field path inside the answers file.
 
 @returns The field as a string.
 
 @throws {@link AnswersError} when the field is not a non-empty string.
 
 @example
 ```ts
 const record = asString({ value: '## Answered', at: 'record' });
 ```
 */
function asString({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): string {
  if (((typeof value) !== 'string') || (value.trim() === '')) {
    refuse({
      at,
      expects: 'a non-empty string',
    },);
  }
  return value;
}

/**
 Reads one field as an array of strings.
 
 @param value - Value read from the answers file.
 
 @param at - Field path inside the answers file.
 
 @returns The field as a list of strings, empty when the field is absent.
 
 @throws {@link AnswersError} when the field is not an array of strings.
 
 @example
 ```ts
 const sections = asStringList({ value: [], at: 'remove.sections' });
 ```
 */
function asStringList({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): readonly string[] {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value,)) {
    refuse({
      at,
      expects: 'an array of strings',
    },);
  }
  return value.map(function toString(
    item,
    index,
  ): string {
    return asString({
      value: item,
      at: `${at}[${String(index,)}]`,
    },);
  },);
}

/**
 Reads one field as a list of line runs.
 
 @param value - Value read from the answers file.
 
 @param at - Field path inside the answers file.
 
 @returns One list of exact consecutive lines per run, empty when the field is absent.
 
 @throws {@link AnswersError} when the field is not a list of non-empty line runs.
 
 @example
 ```ts
 const runs = asLineRuns({ value: [['a', 'b']], at: 'remove.lineRuns' });
 ```
 */
function asLineRuns({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): readonly (readonly string[])[] {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value,)) {
    refuse({
      at,
      expects: 'an array of line runs',
    },);
  }
  return value.map(function toRun(
    run,
    index,
  ): readonly string[] {
    /**
     Lines of one run.
     */
    const lines = asStringList({
      value: run,
      at: `${at}[${String(index,)}]`,
    },);
    if (lines.length === 0) {
      refuse({
        at: `${at}[${String(index,)}]`,
        expects: 'at least one line',
      },);
    }
    return lines;
  },);
}

/**
 Reads one field as a keyed record.
 
 @param value - Value read from the answers file.
 
 @param at - Field path inside the answers file.
 
 @returns The field as a record, empty when the field is absent.
 
 @throws {@link AnswersError} when the field is not a keyed record.
 
 @example
 ```ts
 const remove = asRecord({ value: {}, at: 'remove' });
 ```
 */
function asRecord({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): JsonObject {
  if (value === undefined) {
    return {};
  }
  if (!isRecord(value,)) {
    refuse({
      at,
      expects: 'a JSON object',
    },);
  }
  return value;
}

/**
 Parses answers text, naming the source when it is not JSON.
 
 @param text - Answers file contents.
 
 @param at - How failures name the source.
 
 @returns The parsed value, before the schema is checked.
 
 @throws {@link AnswersError} when the text is not JSON.
 
 @example
 ```ts
 const parsed = parseText({ text: '{}', at: 'answers.json' });
 ```
 */
function parseText({
  text,
  at,
}: {
  readonly text: string;
  readonly at: string;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new AnswersError(`${at} is not valid JSON: ${error.message}`,);
  }
}

/**
 Parses and validates answers text.
 
 @param text - Answers file contents.
 
 @param at - How failures name the source.
 
 @returns The validated answers.
 
 @throws {@link AnswersError} when the text is not a valid answers file.
 
 @example
 ```ts
 const answers = parseAnswers({ text: '{"record":"## Answered","anchor":"## Recorded"}', at: 'answers.json' });
 ```
 */
export function parseAnswers({
  text,
  at,
}: {
  readonly text: string;
  readonly at: string;
},): Answers {
  /**
   Answers text parsed, before its fields are read.
   */
  const parsed = parseText({
    text,
    at,
  },);
  if (!isRecord(parsed,)) {
    refuse({
      at,
      expects: 'a JSON object',
    },);
  }
  /**
   What the answers block replaces in the questions file.
   */
  const remove = asRecord({
    value: parsed.remove,
    at: `${at}.remove`,
  },);
  return {
    file: asString({
      value: parsed.file,
      at: `${at}.file`,
    },),
    record: asString({
      value: parsed.record,
      at: `${at}.record`,
    },),
    anchor: asString({
      value: parsed.anchor,
      at: `${at}.anchor`,
    },),
    remove: {
      sections: asStringList({
        value: remove.sections,
        at: `${at}.remove.sections`,
      },),
      bullets: asStringList({
        value: remove.bullets,
        at: `${at}.remove.bullets`,
      },),
      lineRuns: asLineRuns({
        value: remove.lineRuns,
        at: `${at}.remove.lineRuns`,
      },),
    },
  };
}
