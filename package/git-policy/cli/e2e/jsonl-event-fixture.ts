/**
 Extracts cli-git JSONL policy events from a wrapper's stderr
 and checks them against the process exit code
 (`SPEC.md` "Stream and exit contract").

 Wrapper stderr interleaves real Git's human output with events,
 so only lines that open with `{` are treated as event candidates.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Types

/**
 One decoded policy event.
 */
export type PolicyEvent = Readonly<{
  /**
   Invocation-local sequence.
   */
  sequence: number;
  /**
   Event type such as `finding` or `engine-failure`.
   */
  type: string;
  /**
   Finding severity when present.
   */
  severity?: string;
  /**
   Event code when present.
   */
  code?: string;
  /**
   Landed commit for `commit-landed`.
   */
  oid?: string;
}>;

/**
 Decoded events plus lines that looked like events but were not.
 */
export type EventExtraction = Readonly<{
  /**
   Events in emission order.
   */
  events: readonly PolicyEvent[];
  /**
   Problems found while decoding.
   */
  issues: readonly string[];
}>;

//endregion Types

//region Extraction

/**
 Reports whether parsed JSON is an object whose fields can be read.

 @param value - parsed JSON

 @returns whether the value is a non-array object

 @example
 ```ts
 isJsonObject({}); // => true
 ```
 */
function isJsonObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Decodes one candidate line.

 @param line - stderr line starting with `{`

 @returns event, or a problem description

 @example
 ```ts
 decodeEventLine('{"schemaVersion":1,"sequence":0,"type":"finding"}');
 ```
 */
function decodeEventLine(line: string,): PolicyEvent | string {
  /**
   Parsed JSON or the parse failure.
   */
  const parsed: unknown = (function parseLine(): unknown {
    try {
      return JSON.parse(line,);
    }
    catch (error: unknown) {
      return `malformed JSONL line (${caughtValueText(error,)}): ${line}`;
    }
  })();
  if ((typeof parsed) === 'string')
    return parsed;
  if (!isJsonObject(parsed,))
    return `JSONL line is not an object: ${line}`;
  /**
   Candidate event fields.
   */
  const fields = parsed;
  if (fields.schemaVersion !== 1)
    return `JSONL line has no schemaVersion 1: ${line}`;
  // Trust warnings carry no sequence and are not policy events (SPEC.md "Trust warning object").
  if (fields.type === 'trust-warning')
    return '';
  if (((typeof fields.sequence) !== 'number') || ((typeof fields.type) !== 'string'))
    return `JSONL event lacks sequence or type: ${line}`;
  return {
    sequence: fields.sequence,
    type: fields.type,
    ...((typeof fields.severity) === 'string' ? { severity: fields.severity, } : {}),
    ...((typeof fields.code) === 'string' ? { code: fields.code, } : {}),
    ...((typeof fields.oid) === 'string' ? { oid: fields.oid, } : {}),
  };
}

/**
 Extracts policy events from wrapper stderr.

 @param stderr - complete captured stderr

 @returns events and decoding problems

 @example
 ```ts
 extractPolicyEvents('fatal: x\n{"schemaVersion":1,"sequence":0,"type":"finding"}\n');
 ```
 */
export function extractPolicyEvents(stderr: string,): EventExtraction {
  /**
   Decoded candidates in order.
   */
  const decoded = stderr
    .split('\n',)
    .filter(function candidate(line,) {
      return line.startsWith('{',);
    },)
    .map(decodeEventLine,)
    .filter(function informative(entry,) {
      return entry !== '';
    },);
  /**
   Successfully decoded events.
   */
  const events = decoded.filter(function isEvent(entry,): entry is PolicyEvent {
    return (typeof entry) !== 'string';
  },);
  /**
   Sequence numbering problems.
   */
  const sequenceIssues = events.flatMap(function sequenceCheck(
    event,
    index,
  ) {
    return event.sequence === index ? [] : [`event ${String(index,)} has sequence ${String(event.sequence,)}`,];
  },);
  return {
    events,
    issues: [
      ...decoded.filter(function isIssue(entry,): entry is string {
        return (typeof entry) === 'string';
      },),
      ...sequenceIssues,
    ],
  };
}

//endregion Extraction

//region Exit contract

/**
 Checks that the exit code agrees with the emitted events.

 @param exitCode - process exit code

 @param events - decoded events

 @returns contract violations, empty when consistent

 @example
 ```ts
 checkExitConsistency({ exitCode: 0, events: [] }); // => []
 ```
 */
export function checkExitConsistency({
  exitCode,
  events,
}: Readonly<{
  exitCode: number;
  events: readonly PolicyEvent[];
}>,): readonly string[] {
  /**
   Whether an engine failure was emitted.
   */
  const engineFailure = events.some(function isEngineFailure(event,) {
    return event.type === 'engine-failure';
  },);
  /**
   Index of a commit-landed event, or -1.
   */
  const landedIndex = events.findIndex(function isLanded(event,) {
    return event.type === 'commit-landed';
  },);
  /**
   Whether a blocking finding was emitted.
   */
  const blocking = events.some(function isBlocking(event,) {
    return (event.type === 'core-finding') || ((event.type === 'finding') && (event.severity === 'error'));
  },);
  /**
   Rules paired with their violation text.
   */
  const rules: readonly (readonly [
    boolean,
    string
  ])[] = [
    [
      engineFailure && (exitCode !== 2),
      `engine-failure event with exit ${String(exitCode,)}`,
    ],
    [
      (landedIndex !== (-1)) && (exitCode !== 2),
      `commit-landed event with exit ${String(exitCode,)}`,
    ],
    [
      (landedIndex !== (-1)) && (landedIndex !== (events.length
        - 1)),
      'commit-landed is not the final event',
    ],
    [
      blocking && (!engineFailure)
        && (landedIndex === (-1))
        && (exitCode !== 1),
      `blocking finding with exit ${String(exitCode,)}`,
    ],
    [
      (exitCode === 0) && (blocking || engineFailure
        || (landedIndex !== (-1))),
      'exit 0 with a blocking event',
    ],
  ];
  return rules.flatMap(function violated([failed, message,],) {
    return failed ? [message,] : [];
  },);
}

//endregion Exit contract
