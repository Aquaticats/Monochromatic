import type {
  ExtensionAPI,
  ExtensionContext,
  MessageEndEvent,
  ToolCallEvent,
  ToolResultEvent,
  TurnEndEvent,
} from '@earendil-works/pi-coding-agent';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { ReadonlyDeep, } from 'type-fest';
import { createHash, } from 'node:crypto';
import {
  appendFile,
  mkdir,
  readFile,
} from 'node:fs/promises';
import {
  homedir,
} from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import { unitsOfMessage, } from './message-units.ts';

import {
  COST_TERMS,
  DISMISSAL_TERMS,
  SENTENCE_END,
  SIZE_DISMISSAL,
  WHITESPACE,
} from './patterns.ts';

/**
 Module logger for the pi thinking-dismissal shadow.
 */
const logger = tagged({ tag: 'pi-thinking-dismissal', },);

//region Detection

/**
 Whether the character at one offset is whitespace.

 @param text - Text to inspect

 @param offset - Character offset to test

 @returns True when a whitespace character sits at the offset

 @example
 ```ts
 whitespaceAt({ text: 'a b', offset: 1 }); // => true
 ```
 */
function whitespaceAt(
  {
    text,
    offset,
  }: {
    readonly text: string;
    readonly offset: number;
  },
): boolean {
  /**
   True when the offset sits past the end of the text.
   */
  const past = offset >= text.length;
  if (past) return false;
  return WHITESPACE.has(text.charAt(offset,),);
}

/**
 Splits prose into sentences in one left-to-right pass.

 @param prose - Text to split

 @returns Sentences in order, without their trailing separators

 @example
 ```ts
 sentencesOf('One. Two.'); // => ['One.', 'Two.']
 ```
 */
function sentencesOf(prose: string,): readonly string[] {
  /**
   Sentences collected so far.
   */
  const sentences: string[] = [];
  /**
   Start offset of the sentence being accumulated.
   */
  let start = 0;
  for (let index = 0; index < prose.length; index += 1) {
    if (!SENTENCE_END.has(prose.charAt(index,))) continue;
    /**
     Offset just past this punctuation character.
     */
    const after = index + 1;
    /**
     True when the punctuation closes the text.
     */
    const atEnd = after >= prose.length;
    /**
     True when whitespace follows the punctuation.
     */
    const atWhitespace = whitespaceAt({
      text: prose,
      offset: after,
    },);
    /**
     True when the punctuation ends the sentence.
     */
    const closed = atEnd || atWhitespace;
    if (!closed) continue;
    /**
     Sentence just closed, punctuation included.
     */
    const sentence = prose.slice(
      start,
      after,
    );
    sentences.push(sentence,);
    /**
     Offset of the next sentence's first character.
     */
    let next = after;
    while (whitespaceAt({
      text: prose,
      offset: next,
    },)) {
      next += 1;
    }
    index = next - 1;
    start = next;
  }
  /**
   True when text remains after the last sentence-ending punctuation.
   */
  const trailing = start < prose.length;
  if (trailing) {
    /**
     Closing sentence without trailing punctuation.
     */
    const sentence = prose.slice(start,);
    sentences.push(sentence,);
  }
  return sentences;
}

/**
 Whether a sentence carries any term of one vocabulary.

 @param sentence - Lowercased sentence under test

 @param vocabulary - Lowercase terms to look for

 @returns True when any term occurs in the sentence

 @example
 ```ts
 hasTerm({ sentence: 'the lines are short', vocabulary: ['short'] }); // => true
 ```
 */
function hasTerm(
  {
    sentence,
    vocabulary,
  }: {
    readonly sentence: string;
    readonly vocabulary: readonly string[];
  },
): boolean {
  for (const term of vocabulary) {
    if (sentence.includes(term)) return true;
  }
  return false;
}

/**
 One detection inside one assistant text unit.
 */
type DismissalHit = {
  /**
   Pattern the sentence matches.
   */
  readonly pattern: typeof SIZE_DISMISSAL;

  /**
   First 16 hex characters of the SHA-256 of the whole unit.
   */
  readonly blockHash: string;

  /**
   Zero-based index of the matching sentence inside the unit.
   */
  readonly sentenceIndex: number;
};

/**
 Finds size-dismissal sentences in one assistant text unit.

 @param unit - Assistant prose of one block

 @returns Hits in sentence order, empty when the unit dismisses nothing

 @example
 ```ts
 findSizeDismissal('Quadratic, but the rows are short.');
 ```
 */
function findSizeDismissal(unit: string,): readonly DismissalHit[] {
  /**
   SHA-256 of the unit, truncated, so records can locate it without text.
   */
  const digest = createHash('sha256',)
    .update(unit,)
    .digest('hex',);
  /**
   Short unit identifier recorded instead of any text.
   */
  const blockHash = digest.slice(
    0,
    16,
  );
  /**
   Hits found so far.
   */
  const hits: DismissalHit[] = [];
  /**
   Sentences of the unit, matched one at a time.
   */
  const sentences = sentencesOf(unit,);
  for (const [sentenceIndex, sentence,] of sentences.entries()) {
    /**
     Lowercased sentence, matched case-insensitively.
     */
    const lowered = sentence.toLowerCase();
    /**
     True when the sentence puts algorithmic cost in scope.
     */
    const cost = hasTerm({
      sentence: lowered,
      vocabulary: COST_TERMS,
    },);
    /**
     True when the sentence waves the cost away.
     */
    const dismissal = hasTerm({
      sentence: lowered,
      vocabulary: DISMISSAL_TERMS,
    },);
    if (cost && dismissal) {
      hits.push({
        pattern: SIZE_DISMISSAL,
        blockHash,
        sentenceIndex,
      },);
    }
  }
  return hits;
}

//endregion

//region Shadow record

/**
 Environment variable overriding the shadow log path,
 for tests and disposable runs.
 */
const SHADOW_LOG_ENV = 'MONOCHROMATIC_THINKING_DISMISSAL_LOG' as const;

/**
 One shadow record, appended per event.
 */
type ShadowRecord = {
  /**
   Session the event belongs to.
   */
  readonly sessionId: string;

  /**
   Time the record was written.
   */
  readonly time: string;

  /**
   pi event that produced this observation.
   */
  readonly hookEvent: string;

  /**
   Tool the dismissal preceded,
   on the tool events only.
   */
  readonly toolName?: string;

  /**
   Identifier of the tool call the dismissal preceded,
   on the tool events only.
   */
  readonly toolUseId?: string;

  /**
   Assistant text units folded so far.
   */
  readonly textBlocks: number;

  /**
   Hits not recorded by an earlier event of this kind.
   */
  readonly hits: readonly DismissalHit[];
};

/**
 Resolves the shared shadow log path,
 the same file the Claude Code and DeepSeek Harness legs write.

 @param env - Environment to read the override from

 @returns Path of the append-only shadow log

 @example
 ```ts
 shadowLogPath({}); // => '<home>/.local/state/monochromatic/thinking-dismissal/shadow.jsonl'
 ```
 */
function shadowLogPath(env: ReadonlyDeep<NodeJS.ProcessEnv>,): string {
  /**
   Configured override, empty when the environment carries none.
   */
  const override = env[SHADOW_LOG_ENV] ?? '';
  if (override === '') {
    return join(
      homedir(),
      '.local',
      'state',
      'monochromatic',
      'thinking-dismissal',
      'shadow.jsonl',
    );
  }
  return override;
}

/**
 Shadow writer for the shared log,
 created once per extension load.
 */
type ShadowWriter = {
  /**
   Appends one record carrying only the hits new under this event.

   @param record - Record metadata and the session's hits
   */
  readonly append: (record: ShadowRecord,) => Promise<void>;
};

/**
 Creates the shadow writer for one shared log.

 Hits an earlier record already carries under the same event name are dropped.

 @param logPath - Shadow log to append to

 @returns The writer, frozen

 @example
 ```ts
 createShadowWriter('/state/shadow.jsonl');
 ```
 */
function createShadowWriter(logPath: string,): ShadowWriter {
  /**
   Hit keys already recorded, seeded once from the log tail.
   */
  const seen = new Set<string>();
  /**
   True once the seen set was seeded from the log.
   */
  let seeded = false;
  /**
   Seeds the seen set from the log tail, once per process.
   */

  async function seed(): Promise<void> {
    if (seeded) return;
    seeded = true;
    try {
      /**
       Log text of the tail read.
       */
      const raw = await readFile(
        logPath,
        'utf8',
      );
      for (const line of raw.split('\n',)) {
        if (line === '') continue;
        /**
         Parsed record, or the skip sentinel for a malformed line.
         */
        const record = parseRecord(line,);
        if (record === NOT_PARSED) continue;
        for (const hit of record.hits) {
          seen.add(`${hit.blockHash}:${hit.sentenceIndex}:${hit.pattern}:${record.hookEvent}`,);
        }
      }
    } catch (error) {
      logger.debug(`no shadow log to read back yet: ${String(error,)}`,);
    }
  }
  /**
   Appends one record carrying only the hits new under this event.

   @param record - Record metadata and the session's hits
   */
  async function append(record: ShadowRecord,): Promise<void> {
    await seed();
    /**
     Hits this record contributes, in unit order.
     */
    const fresh: DismissalHit[] = [];
    for (const hit of record.hits) {
      /**
       Deduplication key of the hit under this event.
       */
      const key = `${hit.blockHash}:${hit.sentenceIndex}:${hit.pattern}:${record.hookEvent}`;
      if (seen.has(key)) continue;
      seen.add(key,);
      fresh.push(hit,);
    }
    await mkdir(
      dirname(logPath,),
      { recursive: true, },
    );
    await appendFile(
      logPath,
      `${JSON.stringify({
        ...record,
        hits: fresh,
      },)}\n`,
      'utf8',
    );
  }
  /**
   Frozen writer handed to the extension.
   */
  const writer = Object.freeze({ append, },);
  return writer;
}

/**
 Sentinel returned when a log line does not parse into its record shape.
 */
const NOT_PARSED = 'not-parsed' as const;

/**
 Parses one shadow-log JSONL line,
 or returns {@link NOT_PARSED} when the line is not a record.

 @param line - One JSONL line

 @returns The parsed record, or {@link NOT_PARSED}

 @example
 ```ts
 parseRecord('{"hits":[]}');
 ```
 */
function parseRecord(line: string,): ShadowRecord | typeof NOT_PARSED {
  try {
    /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing a machine-written log line after its parse succeeds */
    return JSON.parse(line,) as ShadowRecord;
  } catch (error) {
    logger.warn(`skipping an unparseable shadow line: ${String(error,)}`,);
    return NOT_PARSED;
  }
}

//endregion

//region Extension

/**
 Pi thinking-dismissal extension:
 folds assistant prose for size dismissals
 and shadow-records them at the tool and turn boundaries.

 Everything runs on this machine and the record carries hashes and indexes
 only, so no reasoning text reaches the log or any provider.
 The extension records and never blocks.

 @param pi - pi extension API

 @example
 ```json
 { "packages": ["/var/home/user/Monochromatic/package/pi-plugin/thinking-dismissal"] }
 ```
 */
export default function piThinkingDismissal(pi: ExtensionAPI,): void {
  /**
   Hits folded per session id.
   */
  const hitsBySession = new Map<string, DismissalHit[]>();
  /**
   Text units folded per session id.
   */
  const unitsBySession = new Map<string, number>();
  /**
   Writer for the shared shadow log.
   */
  const writer = createShadowWriter(shadowLogPath(process.env,),);

  /**
   Records one event's view of the session's hits.

   @param hookEvent - pi event name that produced the observation

   @param ctx - Extension context carrying the session

   @param toolName - Tool the dismissal preceded

   @param toolUseId - Identifier of the tool call
   */
  async function record(
    {
      hookEvent,
      ctx,
      toolName,
      toolUseId,
    }: {
      readonly hookEvent: string;
      readonly ctx: ExtensionContext;
      readonly toolName?: string;
      readonly toolUseId?: string;
    },
  ): Promise<void> {
    try {
      /**
       Session manager of the owning context.
       */
      const { sessionManager, } = ctx;
      /**
       Session the event belongs to.
       */
      const sessionId = sessionManager.getSessionId();
      await writer.append({
        sessionId,
        time: new Date().toISOString(),
        hookEvent,
        ...(toolName === undefined ? {} : { toolName, }),
        ...(toolUseId === undefined ? {} : { toolUseId, }),
        textBlocks: unitsBySession.get(sessionId,) ?? 0,
        hits: hitsBySession.get(sessionId,) ?? [],
      },);
    } catch (error) {
      logger.warn(`thinking-dismissal shadow skipped: ${String(error,)}`,);
    }
  }

  pi.on(
    'message_end',
    function foldMessage(
      event: ForeignBorrowed<MessageEndEvent>,
      ctx: ExtensionContext,
    ) {
    /**
     Session the message belongs to.
     */
      const { sessionManager, } = ctx;
      /**
       Session the turn belongs to.
       */
      const sessionId = sessionManager.getSessionId();
    /**
     Assistant prose of the finished message.
     */
      const units = unitsOfMessage(event.message,);
      if (units.length > 0) {
        unitsBySession.set(
          sessionId,
          (unitsBySession.get(sessionId,) ?? 0) + units.length,
        );
      /**
       Hits of this message, appended to the session's.
       */
        const found: DismissalHit[] = [];
        for (const unit of units) found.push(...findSizeDismissal(unit,),);
        if (found.length > 0) {
          hitsBySession.set(
            sessionId,
            [
              ...(hitsBySession.get(sessionId,) ?? []),
              ...found
            ],
          );
        }
      }
      return undefined;
    },
  );
  pi.on(
    'tool_call',
    function recordIntent(
      event: ForeignBorrowed<ToolCallEvent>,
      ctx: ExtensionContext,
    ) {
      void record({
        hookEvent: 'tool_call',
        ctx,
        toolName: event.toolName,
        toolUseId: event.toolCallId,
      },);
      return undefined;
    },
  );
  pi.on(
    'tool_result',
    function recordCompletion(
      event: ForeignBorrowed<ToolResultEvent>,
      ctx: ExtensionContext,
    ) {
      void record({
        hookEvent: 'tool_result',
        ctx,
        toolUseId: event.toolCallId,
      },);
      return undefined;
    },
  );
  pi.on(
    'turn_end',
    function recordTurnEnd(
      _event: ForeignBorrowed<TurnEndEvent>,
      ctx: ExtensionContext,
    ) {
      void record({
        hookEvent: 'turn_end',
        ctx,
      },);
      return undefined;
    },
  );
}

//endregion

export type { DismissalHit, };

export { findSizeDismissal, };
