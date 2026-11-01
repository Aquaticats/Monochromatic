import type {
  UserPromptSubmitInput,
  UserPromptSubmitOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import {
  containsAnyOfWordBounded,
  PHRASE_NOT_FOUND,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

//region Patterns

/**
 Phrases that indicate the user is correcting a substantive claim from the
 previous assistant response.
 
 Drawn from session evidence (`1cbe8d82`) where four consecutive user
 corrections produced four fresh errors: each revision shared the blind
 spot of the claim it replaced. The pattern this hook targets is treating
 a correction as a small patch rather than an approach-change moment that
 warrants concrete re-verification.
 
 Stored as lowercase phrase literals so {@link containsAnyOfWordBounded} can
 scan with word-boundary checks. Each apostrophe-bearing variant is paired
 with its no-apostrophe form to mirror the original regexes' `['']?`
 optional marker (e.g. both `you're wrong` and `youre wrong`).
 */
const CORRECTION_PHRASES: readonly string[] = [
  // \bdemonstrably (?:false|wrong)\b
  'demonstrably false',
  'demonstrably wrong',
  // \byou missed\b
  'you missed',
  // \bdidn['']?t you\b
  "didn't you",
  'didnt you',
  // \byou['']?re wrong\b
  "you're wrong",
  'youre wrong',
  // \byou are wrong\b
  'you are wrong',
  // \bshouldn['']?t have\b
  "shouldn't have",
  'shouldnt have',
  // \bshould not have\b
  'should not have',
  // \bwhy would you\b
  'why would you',
  // \bthat['']?s wrong\b
  "that's wrong",
  'thats wrong',
  // \bthis is wrong\b
  'this is wrong',
  // \byou got (?:that|this|it) wrong\b
  'you got that wrong',
  'you got this wrong',
  'you got it wrong',
  // \byou['']?re not (?:feeling|paying) (?:well|attention)\b
  "you're not feeling well",
  "you're not feeling attention",
  "you're not paying well",
  "you're not paying attention",
  'youre not feeling well',
  'youre not feeling attention',
  'youre not paying well',
  'youre not paying attention',
  // \bplease be more careful\b
  'please be more careful',
  // \b(?:are you|aren['']?t you) (?:not )?paying attention\b
  'are you paying attention',
  'are you not paying attention',
  "aren't you paying attention",
  "aren't you not paying attention",
  'arent you paying attention',
  'arent you not paying attention',
];

//endregion

//region Detection

/**
 Unicode left single quotation mark (`U+2018`).
 */
const LEFT_SINGLE_QUOTE = '‘';

/**
 Unicode right single quotation mark (`U+2019`).
 */
const RIGHT_SINGLE_QUOTE = '’';

/**
 Normalises curly quotation marks (`U+2018`, `U+2019`) to ASCII apostrophes
 so phrase lookups stay simple.
 
 Replaces the original regexes' `['']?` optional marker (which accepted
 either apostrophe shape); after normalisation the phrase list only needs
 the straight-apostrophe form for each variant.
 
 @param prompt - raw user input
 
 @returns prompt with curly single quotes replaced by `'`
 
 @example
 ```ts
 normaliseApostrophes('that’s wrong'); // => "that's wrong"
 ```
 */
function normaliseApostrophes(prompt: string,): string {
  return prompt
    .replaceAll(
    LEFT_SINGLE_QUOTE,
    "'",
    )
    .replaceAll(
    RIGHT_SINGLE_QUOTE,
    "'",
    );
}

/**
 Tests whether a user prompt matches any correction-phrase pattern.
 
 Apostrophes are normalised first so curly-quote prompts match the
 straight-quote phrase list entries. Lookup is delegated to
 {@link containsAnyOfWordBounded}, which scans case-insensitively with
 `\b<phrase>\b` semantics.
 
 @param prompt - raw user input from the UserPromptSubmit event
 
 @returns `true` when the prompt looks like a correction of a prior claim
 
 @example
 ```ts
 detectCorrection('That\'s demonstrably false. Please be more careful.');
 // => true
 
 detectCorrection('What\'s the status of the build?');
 // => false
 ```
 */
function detectCorrection(prompt: string,): boolean {
  return containsAnyOfWordBounded({
    haystack: normaliseApostrophes(prompt,),
    phrases: CORRECTION_PHRASES,
  },)
    !== PHRASE_NOT_FOUND;
}

//endregion

//region Output

/**
 Text injected into Claude's context when a correction is detected.
 
 Mirrors the language of AGENTS.md rule CKB so the
 reminder fires in the same vocabulary the agent is already trained on.
 */
const CORRECTION_REMINDER_TEXT = [
  '<correction-detected>',
  'The user is correcting a substantive claim from your previous response.',
  'Per AGENTS.md rule CKB: a user correction is an',
  'approach-change moment, not a small patch. The blind spot that produced',
  'the original claim is still active for the revision.',
  '',
  'Before your next substantive response, re-check the claim against primary',
  'sources, local files, logs, or command output. Do not run a same-session',
  'self-review or write an `Advisor pass: ...` line; self-review is not',
  'independent evidence. See `doc/agent/self-review.md`.',
  '</correction-detected>',
]
  .join('\n',);

/**
 Output of the correction-reminder handler, a {@link UserPromptSubmitOutput}.
 
 Always uses `additionalContext`: the handler never blocks the prompt.
 */
type CorrectionReminderOutput = UserPromptSubmitOutput;

//endregion

/**
 Scans user input for correction phrases and injects a concrete-verification
 reminder when one is found. Returns an empty additionalContext when no
 correction is detected so the rest of the pipeline runs unchanged.
 
 @param event - parsed {@link UserPromptSubmitInput} event from Claude Code
 
 @returns UserPromptSubmit response with `additionalContext` populated when
   a correction phrase fires, empty string otherwise
 
 @example
 ```ts
 correctionReminderHandler({
   hook_event_name: 'UserPromptSubmit',
   prompt: 'That\'s demonstrably false.',
   session_id: 's', transcript_path: '/t', cwd: '/c',
 });
 // additionalContext contains the <correction-detected> reminder
 ```
 */
function correctionReminderHandler(
  event: ReadonlyDeep<UserPromptSubmitInput>,
): CorrectionReminderOutput {
  /**
   True when the user's prompt contains a correction phrase; gates the reminder text.
   */
  const triggered = detectCorrection(event.prompt,);
  return {
    hookSpecificOutput: {
      hookEventName: 'UserPromptSubmit',
      additionalContext: triggered ? CORRECTION_REMINDER_TEXT : '',
    },
  };
}

/**
 Parses raw stdin as a {@link UserPromptSubmitInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @param raw - JSON-encoded event payload received on stdin
 
 @returns the parsed UserPromptSubmit event
 
 @example
 ```ts
 correctionReminderParser('{"hook_event_name":"UserPromptSubmit","prompt":"hi"}');
 // => { hook_event_name: 'UserPromptSubmit', prompt: 'hi', ... }
 ```
 */
function correctionReminderParser(raw: string,): UserPromptSubmitInput {
  /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted input from Claude Code hook system */
  return JSON.parse(raw,) as UserPromptSubmitInput;
}

/**
 Serializes the correction-reminder output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @param output - {@link CorrectionReminderOutput} handler result to serialize
 
 @returns JSON-encoded payload for Claude Code's hook reader
 
 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 correctionReminderWriter({ hookSpecificOutput: { hookEventName: 'UserPromptSubmit', additionalContext: '' } });
 // => '{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":""}}'
 ```
 */
function correctionReminderWriter(output: CorrectionReminderOutput,): string {
  return JSON.stringify(output,);
}

export type { CorrectionReminderOutput, };

export {
  correctionReminderHandler,
  correctionReminderParser,
  correctionReminderWriter,
  detectCorrection,
};
