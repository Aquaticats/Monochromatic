/**
 Writing one second review's plan, refusing any file that already holds somebody else's text.

 Each file is created with an exclusive create, never replaced: a text that appeared after the refusal check makes the create fail,
 and is left where it is. A file that already holds the plan's own text, or the beginning of it that a kill cut short, is the plan's
 own write and is left or repaired; every other text refuses the whole plan.

 @module
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import type {
  RestartPlan,
} from './restart-brief.ts';
import { textAtAPath, } from './review-check-reading.ts';
import { reviewFileOfAReview, } from './reviewer-of-a-change.ts';

/**
 The code the operating system gives a path that already exists.
 */
const ALREADY_THERE_CODE = 'EEXIST';

/**
 Says whether a path holds a text that is this plan's own write, cut short.

 A file whose text is a prefix of the plan's own text is a write of the plan that a kill cut short: it holds nothing but the
 beginning of what this plan would write, so repairing it discards nothing. Every other text is somebody else's and is refused.

 @param held - what the path holds now

 @param plan - the text the plan would write

 @returns whether the held text is the plan's own, whole or cut short

 @example
 ```ts
 const ours = isThePlansOwn({ held: 'the first half', plan: 'the first half and the rest' });
 ```
 */
function isThePlansOwn({
  held,
  plan,
}: {
  readonly held: string;
  readonly plan: string;
},): boolean {
  return (held === '') || plan.startsWith(held,);
}

/**
 Reads what one written target's path holds, and refuses it when it holds somebody else's text.

 @param target - one file the plan writes, and its text

 @param restartReview - the restart's review name, for the message

 @returns why the target refuses the plan, or nothing when it does not

 @example
 ```ts
 const refusal = await refusalOfATarget({ target, restartReview: 'a-restart-2' });
 ```
 */
async function refusalOfATarget({
  target,
  restartReview,
}: {
  readonly target: RestartPlan['texts'][number];
  readonly restartReview: string;
},): Promise<string | typeof NO_REFUSAL> {
  /**
   What is at the target's path now.
   */
  const there = await textAtAPath({ path: target.path, },);
  if (there.kind === 'no file') {
    return NO_REFUSAL;
  }
  if (there.kind === 'unreadable') {
    return `${restartReview}: ${target.path} cannot be read (${there.reason}), so it was not replaced`;
  }
  if (isThePlansOwn({
    held: there.text,
    plan: target.text,
  },)) {
    return NO_REFUSAL;
  }
  return `${restartReview}: ${target.path} is already there with other text, so it was not replaced; give this restart room by `
    + 'moving that file away';
}

/**
 No refusal of a written target.
 */
const NO_REFUSAL: unique symbol = Symbol('written target holds the plan own text',);

/**
 Writes one file the plan holds, creating it rather than replacing it.

 A `wx` create is what makes the check and the write one act: a text that appeared after the refusal check makes the create fail,
 and is never replaced. A file that already holds the plan's own text is left; a file holding its beginning, which a kill cut short,
 is removed and created again, because it holds nothing the plan would not write.

 @param target - the file and its text

 @param restartReview - the restart's review name, for the message

 @returns why the target refuses the plan, or nothing when it was written or already held its text

 @throws when the target cannot be read or created for any reason but another writer

 @example
 ```ts
 const refused = await writeOneTarget({ target, restartReview: 'a-restart-2' });
 ```
 */
async function writeOneTarget({
  target,
  restartReview,
}: {
  readonly target: RestartPlan['texts'][number];
  readonly restartReview: string;
},): Promise<string | typeof NO_REFUSAL> {
  /**
   What is at the target's path now.
   */
  const there = await textAtAPath({ path: target.path, },);
  if ((there.kind === 'the text') && (there.text === target.text)) {
    return NO_REFUSAL;
  }
  if ((there.kind === 'unreadable')) {
    return `${restartReview}: ${target.path} cannot be read (${there.reason}), so it was not replaced`;
  }
  if (there.kind === 'the text') {
    if (!isThePlansOwn({
      held: there.text,
      plan: target.text,
    },)) {
      return `${restartReview}: ${target.path} is already there with other text, so it was not replaced`;
    }
    // The beginning of this plan's own text is a write a kill cut short; it holds nothing to keep.
    await rm(
      target.path,
      { force: true, },
    );
  }
  await mkdir(
    dirname(target.path,),
    { recursive: true, },
  );
  try {
    await writeFile(
      target.path,
      target.text,
      {
        encoding: 'utf8',
        flag: 'wx',
      },
    );
    return NO_REFUSAL;
  }
  catch (error) {
    if (errorCodeOf(error,) !== ALREADY_THERE_CODE) {
      throw error;
    }
    /**
     What the other writer put there.
     */
    const other = await textAtAPath({ path: target.path, },);
    if ((other.kind === 'the text') && (other.text === target.text)) {
      return NO_REFUSAL;
    }
    return `${restartReview}: ${target.path} was written by another command while this plan ran, so it was not replaced`;
  }
}

/**
 Writes one second review's plan, refusing any file that already holds other text.

 @param plan - the plan to write

 @returns why the plan was refused, or nothing when every file was written or already held its text

 @example
 ```ts
 const written = await writeRestartPlan({ plan });
 ```
 */
export async function writeRestartPlan({ plan, }: { readonly plan: RestartPlan },): Promise<{
  readonly refused?: string
}> {
  /**
   The review file a second review that already ran would have written.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder: plan.reviewsFolder,
    review: plan.restartReview,
  },);
  if ((await textAtAPath({ path: reviewFile, },)).kind !== 'no file') {
    return {
      refused: `${plan.restartReview}: ${reviewFile} is already there, so this second review already ran and was not written again`,
    };
  }
  /**
   One refusal per target, in the plan's order, before anything is written.
   */
  const refusals = await Promise.all(plan.texts
    .map(async function refusalAt(target,) {
      return await refusalOfATarget({
        target,
        restartReview: plan.restartReview,
      },);
    },),);
  /**
   The first refusal, which is the one a reader needs.
   */
  const refusal = refusals.find(function isARefusal(one,) {
    return one !== NO_REFUSAL;
  },);
  if (refusal !== undefined) {
    return { refused: refusal, };
  }
  /* oxlint-disable eslint/no-await-in-loop -- one file after another, each created before the next, so a refusal names the file
     it belongs to and no file is replaced. See doc/lint/sequential-awaits.md. */
  for (const target of plan.texts) {
    /**
     What writing this one file answered.
     */
    const written = await writeOneTarget({
      target,
      restartReview: plan.restartReview,
    },);
    if (written !== NO_REFUSAL) {
      return { refused: written, };
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return {};
}
