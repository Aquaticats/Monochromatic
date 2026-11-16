/**
 What a run's free-form Pi arguments mean, decided by Pi's own argument reader, and which of them resume a session this package
 cannot mask before the start.

 The driver masks the session it is told about with its own `--session` option. Pi's command line is also open to free-form words, and
 Pi's own parser reads `--session`, `--session-id`, `--fork`, `--continue` and `--resume` there: a run started with one of those
 resumes entries that this run's hooks never saw, and Pi's compaction builds a request from them. So the words are read here with the
 same parser Pi uses, and one that resumes a session is refused with its reason instead of started.

 @module
 */

import { parseArgs, } from '@earendil-works/pi-coding-agent';

//region The words that resume

/**
 A free-form word that names or resumes a session, and why it is refused.
 */
export type RefusedFreeFormWord = {
  /**
   The word as the caller wrote it.
   */
  readonly word: string;

  /**
   Why a run cannot start with it.
   */
  readonly reason: string;
};

/**
 The words that name or resume a session, which Pi reads from its own command line.
 */
const WORDS_THAT_RESUME: ReadonlySet<string> = new Set([
  '--session',
  '--session-id',
  '--fork',
  '--continue',
  '-c',
  '--resume',
  '-r',
]);

/**
 The reason every refused word is refused with, which says where the masking happens instead.
 */
const REFUSAL = 'it resumes a session this package cannot mask before Pi reads it, because the driver masks only the session its own '
  + '--session option names; pass that option instead, so the session is masked before the start';

//endregion The words that resume

//region Reading them

/**
 Reads a run's free-form Pi words and says which of them resume a session.

 @param words - the free-form words the run was given

 @returns the refused words and why, in the order they were written

 @example
 ```ts
 const refused = refusedFreeFormWords({ words: ['--session', '/tmp/old.jsonl',], },);
 ```
 */
export function refusedFreeFormWords({ words, }: { readonly words: readonly string[] },): readonly RefusedFreeFormWord[] {
  /**
   What Pi's own reader makes of the words.
   */
  const parsed = parseArgs([...words,],) as {
    readonly session?: string;
    readonly sessionId?: string;
    readonly fork?: string;
    readonly continue?: boolean;
    readonly resume?: boolean;
  };
  /**
   Whether the words name or resume a session at all.
   */
  const resumes = (parsed.session !== undefined) || (parsed.sessionId !== undefined)
    || (parsed.fork !== undefined)
    || (parsed.continue === true)
    || (parsed.resume === true);
  if (!resumes) {
    return [];
  }
  /**
   The words that say so, as the caller wrote them.
   */
  return words
    .filter(function namesASession(word,) {
      // Pi's reader takes both `--session <path>` and `--session=<path>`, so the flag before an `=` is what is asked about.
      return WORDS_THAT_RESUME.has(word.split('=',)[0] ?? word,);
    },)
    .map(function refused(word,) {
      return {
        word,
        reason: REFUSAL,
      };
    },);
}

/**
 Says whether a run's free-form words tell Pi to keep no session at all.

 @param words - the free-form words the run was given

 @returns whether Pi will write no session, which leaves nothing for the session's own masking to clear

 @example
 ```ts
 const none = keepsNoSession({ words: ['--no-session',], },);
 ```
 */
export function keepsNoSession({ words, }: { readonly words: readonly string[] },): boolean {
  /**
   What Pi's own reader makes of the words.
   */
  const parsed = parseArgs([...words,],) as { readonly noSession?: boolean };
  return parsed.noSession === true;
}

//endregion Reading them
