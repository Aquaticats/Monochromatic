/**
 Expectations of a repository that configures hooks the wrapper no longer runs.

 The wrapper supports no external hook program
 (`doc/planning/cli-git-no-self-invocation-dispatcher.md`, "What exists now"):
 a repository that configures `hook.<name>.event` gets a `configuration-warning` event with the code
 `external-hooks-unsupported` on every commit, and nothing the configuration names runs.
 The hook program of the suite stays installed as the canary that proves it.

 @module
 */

import { join, } from 'node:path';

import { extractPolicyEvents, } from './jsonl-event-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { readOptionalText, } from './process-fixture.ts';
import type {
  Expectations,
  ScenarioContext,
} from './scenario-model-fixture.ts';

//region Constants

/**
 Code of the warning the wrapper gives for a configured hook.
 */
export const EXTERNAL_HOOKS_CODE = 'external-hooks-unsupported';

/**
 Characters of a log that a failure detail quotes.
 */
const STDERR_EXCERPT_CHARACTERS = 400;

//endregion Constants

//region Expectations

/**
 Whether an attempt's events carry the external-hooks warning.

 @param attempt - finished attempt

 @returns true when a `configuration-warning` event has the external-hooks code

 @example
 ```ts
 warnsAboutHooks(attempt);
 ```
 */
function warnsAboutHooks(attempt: AttemptRecord,): boolean {
  return extractPolicyEvents(attempt.outcome
    .stderr,)
    .events
    .some(function isWarning(event,) {
      return (event.type === 'configuration-warning') && (event.code === EXTERNAL_HOOKS_CODE);
    },);
}

/**
 Expectations for attempts made in a repository with configured hooks.

 @param context - scenario context

 @param attempts - finished attempts that landed

 @returns the canary stayed silent, and every attempt warned

 @example
 ```ts
 return await hooksIgnored({ context, attempts });
 ```
 */
export async function hooksIgnored({
  context,
  attempts,
}: Readonly<{
  context: ScenarioContext;
  attempts: readonly AttemptRecord[];
}>,): Promise<Expectations> {
  /**
   Runs the hook program logged, empty when it never ran.
   */
  const ran = (await readOptionalText(join(
    context.repository
      .logDir,
    'hooks.jsonl',
  ),)).trim();
  /**
   Attempts that gave no warning.
   */
  const silent = attempts.filter(function unwarned(attempt,) {
    return !warnsAboutHooks(attempt,);
  },);
  /**
   Start of the first silent attempt's standard error, which says what it printed instead.
   */
  const printed = (silent[0]
    ?.outcome
    .stderr
    ?? '').slice(
    0,
    STDERR_EXCERPT_CHARACTERS,
    );
  /**
   Start of the hook program's log, which says what ran.
   */
  const ranExcerpt = ran.slice(
    0,
    STDERR_EXCERPT_CHARACTERS,
  );
  return [
    {
      name: 'configured-hooks-not-run',
      holds: ran === '',
      detail: `the configured hook program ran: ${ranExcerpt}`,
    },
    {
      name: 'configured-hooks-warned',
      holds: silent.length === 0,
      detail: `${silent.map(function label(attempt,) {
        return attempt.label;
      },)
        .join(', ',)} gave no ${EXTERNAL_HOOKS_CODE} warning; the first printed: ${printed}`,
    },
  ];
}

//endregion Expectations
