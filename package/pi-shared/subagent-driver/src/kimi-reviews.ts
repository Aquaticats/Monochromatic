/**
 The brief one Kimi K3.1 Pro review is given, and the list of reviews a round of them is started from.

 Rule `KRV` of `AGENTS.md` gives a review of a change to Kimi K3.1 Pro alone, in a worktree of its own, outside the Neuralwatt
 slots. The owner, 2026-11-10: "THere is no  40-file limit for Kimi.", so every changed file is named.

 @module
 */

/**
 The provider a review runs on, which the owner's rule `KRV` names.
 */
export const KIMI_REVIEW_PROVIDER = 'synthetic';

/**
 The model a review runs on, which the owner's rule `KRV` names.
 */
export const KIMI_REVIEW_MODEL = 'hf:moonshotai/Kimi-K3.1-Pro';

/**
 Where a review goes when Synthetic answers 429, in the owner's words: "Use neuralwatt/kimi-k3.1-pro-flex whenever Synthetic
 429s." and "Synthetic doesn't have a concurrent limit. The Synthetic quota got blown once again."
 */
export const KIMI_FALLBACK_PROVIDER = 'neuralwatt';

/**
 The model that same pool is asked for.
 */
export const KIMI_FALLBACK_MODEL = 'kimi-k3.1-pro-flex';

/**
 The folder a review asks to be reviewed under, relative to the repository, when the review names none.
 */
export const DEFAULT_REVIEW_SOURCE = 'package/git-policy/cli/src/native';

/**
 One review to start.
 */
export type KimiReview = {
  /**
   The review's name, which names its run, its worktree, its branch and its folder.
   */
  readonly name: string;

  /**
   The commit the worktree is made at, which may be newer than the range's end.
   */
  readonly commit: string;

  /**
   The range that names what changed, as `git diff` takes it.
   */
  readonly range: string;

  /**
   What to look hardest at.
   */
  readonly focus: string;

  /**
   The folder the review reads, relative to the repository; absent for {@link DEFAULT_REVIEW_SOURCE}.
   */
  readonly source?: string;
};

/**
 A review list that cannot be read as what it claims to be.
 */
export class KimiReviewListError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'KimiReviewListError';
}

/**
 Reads one field of a review that must be text.

 @param value - the field as the list holds it

 @param where - which review and which field, for the message

 @returns the text

 @throws KimiReviewListError when the field is not text with something in it

 @example
 ```ts
 const commit = textOf({ value: 'aa7f601c6d', where: 'review 1 --commit', });
 ```
 */
function textOf({
  value,
  where,
}: {
  readonly value: unknown;
  readonly where: string;
},): string {
  if (((typeof value) !== 'string') || (value === '')) {
    throw new KimiReviewListError(`${where} must be text with something in it`,);
  }
  return value;
}

/**
 Reads one review of a review list.

 @param value - the review as the list holds it

 @param at - the review's place in the list, counting from one

 @returns the review

 @throws KimiReviewListError when the review is not an object, or a field it must carry is missing or unusable

 @example
 ```ts
 const review = reviewOf({ value: { name: 'a', commit: 'b', range: 'c', focus: 'd', }, at: 1, });
 ```
 */
function reviewOf({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: number;
},): KimiReview {
  if (((typeof value) !== 'object') || (value === null)) {
    throw new KimiReviewListError(`review ${String(at,)} must be an object`,);
  }
  /**
   The review's fields, by name.
   */
  const fields = value as Readonly<Record<string, unknown>>;
  /**
   The review's place, for a message about one of its fields.
   */
  const where = `review ${String(at,)}`;
  return {
    name: textOf({
      value: fields.name,
      where: `${where} --name`,
    },),
    commit: textOf({
      value: fields.commit,
      where: `${where} --commit`,
    },),
    range: textOf({
      value: fields.range,
      where: `${where} --range`,
    },),
    focus: textOf({
      value: fields.focus,
      where: `${where} --focus`,
    },),
    ...((fields.source === undefined) ? {} : { source: textOf({
      value: fields.source,
      where: `${where} --source`,
    },), }),
  };
}

/**
 Reads the list a review file holds, or says why it is no JSON.

 @param text - the list as the file holds it

 @returns what the list holds

 @throws KimiReviewListError when the text is not JSON

 @example
 ```ts
 const list = parsedListOf({ text: '[]', });
 ```
 */
function parsedListOf({
  text,
}: {
  readonly text: string;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new KimiReviewListError(
      `a review list must be JSON: ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads a review list.

 @param text - the list as the file holds it

 @returns the reviews, in the order the list gives them

 @throws KimiReviewListError when the text is no list of reviews

 @example
 ```ts
 const reviews = parseKimiReviewList({ text: '[]', });
 ```
 */
export function parseKimiReviewList({
  text,
}: {
  readonly text: string;
},): readonly KimiReview[] {
  /**
   What the list holds.
   */
  const list = parsedListOf({ text, },);
  if (!Array.isArray(list,)) {
    throw new KimiReviewListError('a review list must be an array of reviews',);
  }
  return list.map(function reviewAt(
    value,
    index,
  ) {
    return reviewOf({
      value,
      at: index + 1,
    },);
  },);
}

/**
 Builds the brief one review is given.

 @param review - the review to brief

 @param filesToRead - every file the review must read in full, the diff first

 @param runFolder - the folder the review writes its answer and its reproductions into

 @param runsFolder - the folder the runs are started from, which the brief forbids reading other runs of

 @param reviewsFolder - the folder every review writes into, which the brief forbids reading others of

 @returns the brief

 @example
 ```ts
 const brief = briefOf({ review, filesToRead: ['/reviews/a/change.diff',], runFolder: '/reviews/a', runsFolder: '/runs', reviewsFolder: '/reviews', });
 ```
 */
export function briefOf({
  review,
  filesToRead,
  runFolder,
  runsFolder,
  reviewsFolder,
}: {
  readonly review: KimiReview;
  readonly filesToRead: readonly string[];
  readonly runFolder: string;
  readonly runsFolder: string;
  readonly reviewsFolder: string;
},): string {
  return `You review one change to the code at commit ${review.commit}. The folder you start in is at that commit.

Never generate synthetic host load in any session or subagent: no \`stress-ng\`, busy loops, \`--stress-count\`, or repeated full test runs; cap the test's own container (\`--cpus\` sized to the test) instead.

Your review counts only if it is your own. Do not read anything under \`/var/home/user/temp/agent/sonnet-audits\`, \`/var/home/user/temp/agent/reviews\`, \`/var/home/user/temp/agent/codex-runs\`, \`/var/home/user/temp/agent/kimi-trials\` or \`${reviewsFolder}\` outside your own run folder, or any other run's folder under \`${runsFolder}\`. Those hold other reviewers' answers. Do not run \`ps\` with \`args\`, or anything else that lists other processes' command lines. Look only at the commit you start on and the diff you are given: no other branch or worktree, no GitHub (\`gh\`), no web search about this repository, no session logs under \`~/.claude\` or \`~/.pi\`. Every claim that you reproduced something must match a command you really ran. Your whole session can be read afterwards.

Change no file in this worktree, and do not commit or push. Scratch goes under \`${runFolder}\`. You may build and run the code, but only on disposable repositories under a disposable \`HOME\` and \`XDG_RUNTIME_DIR\`, in a command limited with \`systemd-run --user --scope\`. Never print the environment or read a credential file. If the model provider refuses or flags your content, stop and say so.

Review a change to a Git wrapper written in Rust that stands in front of real Git.
Read each of these files in full before you answer: ${filesToRead.join(' ',)}
Its owner requires: "never discard user data", "Commits should never fail.", and "nothing should ever wait forever or indefinitely".
A process can be killed at any instruction, and other commands run at the same time.
Report only concrete defects. For each one give the file and line, the exact sequence of steps, and the wrong result, and say whether you reproduced it or found it by reading. Put each reproduction script in \`${runFolder}/reproductions\`.
Leave out style, naming, and anything you cannot state as a sequence and a wrong result. Finding nothing is a good result.
Look hardest at: ${review.focus}

Write your review to \`${runFolder}/review.md\` and return the same text as your final message.
`;
}
