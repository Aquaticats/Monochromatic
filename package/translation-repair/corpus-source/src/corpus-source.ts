import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';
import spawn, { SubprocessError, } from 'nano-spawn';

import {
  type CorpusCommitState,
  probeKeepingRead,
} from './corpus-commit-probe.ts';
import {
  CORPUS_GIT_FLAGS,
  corpusGitEnvironment,
  gitSearchCeiling,
} from './corpus-git-context.ts';
import { foldCarriageReturns, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Corpus source
// Reads benchmark texts from the user's local clone of `one-among-us/data`.
// That repository is UNLICENSED (all rights reserved): its content is read at
// runtime for benchmarking and must never be committed into this repository.
// Reads go through `git show <sha>:<path>` against a pinned commit, so benchmark
// runs stay reproducible even while the clone itself moves.

/* oxlint-disable typescript/strict-void-return -- promisify deliberately ignores Node execFile's ChildProcess return while adapting its callback */
/**
 Promise adapter for byte-exact subprocess capture;
 blob reads cannot go through nano-spawn because its line-oriented stdout
 strips the final newline, and repairs must preserve corpus text
 byte-for-byte.
 */
const execFileAsync = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

/**
 Bytes per kibibyte, named for the blob ceiling arithmetic.
 */
const KIBI = 1_024;

/**
 Mebibytes granted to one blob read;
 corpus pages are kilobytes, so this bounds runaway reads generously.
 */
const MAX_BLOB_MEBIBYTES = 64;

/**
 Ceiling for one blob read in bytes.
 */
const MAX_BLOB_BYTES = MAX_BLOB_MEBIBYTES
  * KIBI
  * KIBI;

/**
 Commit of `one-among-us/data` the milestone-one benchmark pins to.
 Verified 2026-07-16: HEAD of upstream `main` and of the user's local clone.
 At this commit `people/` holds 92 zh pages each paired with an en page.
 */
export const CORPUS_COMMIT_SHA = 'a41fc607ea5a70d8a7625cc67d5ed8c444f53379';

/**
 Location of one pinned corpus checkout:
 where the clone lives and which commit reads resolve against.

 @example
 ```ts
 const pin: CorpusPin = {
   cloneDir: `${homedir()}/one-among-us/data`,
   commitSha: CORPUS_COMMIT_SHA,
 };
 ```
 */
export type CorpusPin = {
  /**
   Local clone directory of `one-among-us/data`.
   */
  readonly cloneDir: string;

  /**
   Commit every read resolves against.
   */
  readonly commitSha: string;

  /**
   Git binary to run;
   defaults to resolving the real binary,
   because the repo PATH exposes a policy shim whose staging guards are not
   meant for read-only corpus access.
   */
  readonly gitPath?: string;
};

/**
 What kind of failure a corpus read met, each one thing a walker or an
 operator acts on in one way: only a path absent at a commit the clone holds
 may be stepped past, and each kind's refusal names its own remedy.

 @example
 ```ts
 const failure: CorpusReadFailure = 'missing-object';
 ```
 */
export type CorpusReadFailure =
  /**
   The clone holds the commit and the path is not in it, which is what an
   incomplete pair looks like: `fatal: path 'x' does not exist in 'sha'`.
   */
  | 'missing-object'
  /**
   The clone holds no commit by the revision asked for: a full hash never
   fetched into it, an abbreviation or a name that resolves to no commit in
   it, or the id of an object that is no commit, such as a tree. Git prints
   the same `does not exist in` text for a full hash it lacks as for a path
   absent at a commit it holds, so only a probe of the commit tells the two
   apart.
   */
    | 'missing-commit'
  /**
   Git could not open the clone as a repository, as the probe of the commit
   and the probe of the clone's own git directory both found (exit status
   128): the directory is missing, is not the top of one (git searches no
   higher than the clone, `corpus-git-context.ts`, so a directory inside
   another repository is no clone), or is one git refuses to read. A clone git
   opens and then dies in is `other`.
   */
    | 'unreadable-clone'
  /**
   Anything else at a clone git could open: a blob over the read ceiling, an
   object git cannot inflate, a git child stopped by a signal or never
   started. Git's words are read in the C locale (`corpus-git-context.ts`
   sets `LC_ALL=C`), so no message language turns an absent page into this
   kind.
   */
    | 'other';

/**
 Stderr phrases with which git reports a path absent at a commit.

 MEASURED against git 2.55 rather than recalled: a missing path and a full
 hash the clone lacks both say `does not exist in`, and a path present in the
 working tree but not at the commit says `exists on disk, but not in`
 (also for a hash the clone lacks). The phrases therefore say the path was
 not found, never that the commit is held: the probe of the commit decides.

 THESE ARE GIT'S ENGLISH WORDS, and git prints them whatever the operator's
 locale says: `corpus-git-context.ts` hands every corpus git child
 `LC_ALL=C`, which overrides `LANG`, `LANGUAGE` and `LC_MESSAGES`. Measured
 with git 2.55, `LANGUAGE=de` alone had git word an absent path in German.
 */
const MISSING_OBJECT_PHRASES: readonly string[] = [
  'does not exist in',
  'exists on disk, but not in',
];

/**
 Reads which failure a subprocess error reports.

 BOTH SUBPROCESS SHAPES ARE READ. Blob reads go through `execFile`, whose
 promisified rejection carries `stderr` as a buffer; listings go through
 `nano-spawn`, whose error carries it as a string. Anything without a
 readable stderr is `other`, since nothing then says the object was missing.

 @param cause - underlying subprocess failure

 @returns Failure kind git's text alone supports, which `kindOfFailure` reads
 beside the probe of the commit before any refusal names it

 @example
 ```ts
 const kind = classifyCorpusReadFailure({ cause: error, },);
 ```
 */
function classifyCorpusReadFailure({ cause, }: { readonly cause: unknown; },): CorpusReadFailure {
  if ((typeof cause) !== 'object')
    return 'other';
  if (cause === null)
    return 'other';
  if (!('stderr' in cause))
    return 'other';

  /**
   Whatever stderr the subprocess layer attached, as text.
   */
  const text = stderrText({ stderr: cause.stderr, },);

  /**
   Whether git said the object is absent at the commit.
   */
  const missing = MISSING_OBJECT_PHRASES.some(function appears(phrase,): boolean {
    return text.includes(phrase,);
  },);
  return missing ? 'missing-object' : 'other';
}

/**
 Reads a subprocess layer's stderr as text.

 @param stderr - whatever the layer attached

 @returns Text, or nothing when it is neither a buffer nor a string

 @example
 ```ts
 const text = stderrText({ stderr: error.stderr, },);
 ```
 */
function stderrText({ stderr, }: { readonly stderr: unknown; },): string {
  if (Buffer.isBuffer(stderr,))
    return stderr.toString('utf8',);
  if ((typeof stderr) === 'string')
    return stderr;
  return '';
}

/**
 What each kind says was found and the one remedy that fits it.

 THE KIND AND ITS SENTENCE CLOSE THE MESSAGE AND A LINE NEVER CUTS THEM: a
 `PROBE`, `TALLY` or `CLEANUP` line cuts a refusal's opening at a fixed
 length (`sentinel-probe-line.ts`, `tally-error-text.ts`) and prints this
 closing whole after it, since a long entry id or picture name once pushed
 the remedy past the cut.
 */
const REMEDY_SENTENCES: Readonly<Record<CorpusReadFailure, string>> = {
  'missing-object': 'the commit has no such path: check the path, or pin a commit that has it.',
  'missing-commit': 'the clone holds no commit by that revision: fetch it, or pin a commit the clone holds.',
  'unreadable-clone': 'git could not open the clone: check that the directory exists, is the top of a git '
    + 'repository, and is one git may read.',
  other: 'the read failed another way: run the same git read in the clone by hand to see why.',
};

/**
 Decides the kind from git's text and the probe of the commit.

 A PATH-ABSENT TEXT IS A MISSING OBJECT ONLY WHEN THE COMMIT IS HELD: a probe
 that found the commit lacking makes the kind `missing-commit` whatever git
 printed, a probe git could not run in the clone makes it `unreadable-clone`,
 and a probe that got no answer leaves the failure `other`, since nothing then
 says the commit is held and a walker must not step past it. Git's text alone
 decides only at a commit the probe found held.

 @param reading - git's stderr alone, which can say an object is absent but
 never whether the commit holding it is

 @param commit - probe's answer, which decides every kind but the two git's
 text tells apart at a held commit

 @returns Failure kind the refusal names, the one fact a walker reads to step
 past an entry or stop

 @example
 ```ts
 const kind = kindOfFailure({ reading: 'missing-object', commit: 'lacking', },); // 'missing-commit'
 ```
 */
function kindOfFailure(
  {
    reading,
    commit,
  }: {
    readonly reading: CorpusReadFailure;
    readonly commit: CorpusCommitState;
  },
): CorpusReadFailure {
  if (commit === 'held')
    return reading;
  if (commit === 'lacking')
    return 'missing-commit';
  if (commit === 'unopened')
    return 'unreadable-clone';
  return 'other';
}

/**
 Signals a corpus read that git refused: an absent path at a commit the clone
 holds, a commit the clone lacks, a clone git could not open, or another
 failure, each named by its kind with the remedy that fits it.

 @example
 ```ts
 throw new CorpusReadError({ detail: 'people/whiskers/page.md at a41fc60', cause: error, commit: 'held', },);
 ```
 */
export class CorpusReadError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the corpus path and revision that were asked for, never what they hold.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Which failure git reported, read off its stderr and the probe of the
   commit.

   THE FIELD EVERY CATCHER NEEDED. Until it existed a non-zero git exit, a
   spawn failure, an unreadable clone and an oversized blob all reached a
   caller as one class, and every caller read that class as the expected
   missing side of an incomplete pair: a pass whose clone had gone away
   dropped every entry in silence and ranked its bands over nothing.
   */
  readonly kind: CorpusReadFailure;

  /**
   The kind in brackets and its remedy sentence, which close the message: a
   line that cuts the message at a fixed length cuts what precedes this and
   prints this whole.
   */
  readonly kindAndRemedy: string;

  /**
   Builds failure naming what was read and why git refused.

   @param detail - object spec or listing that failed, which the message names
   so the operator knows which page or revision to check

   @param cause - underlying subprocess failure, whose git stderr tells a path
   absent at a held commit from every other failure

   @param commit - probe's answer about the pinned commit, required since
   git's text alone reads a commit the clone lacks as an absent page, which a
   walker would step past; a reader a case scripts states the answer it stands
   for

   @example
   ```ts
   new CorpusReadError({ detail: 'people/ at deadbeef', cause: error, commit: 'held', },);
   ```
   */
  public constructor(
    {
      detail,
      cause,
      commit,
    }: {
      readonly detail: string;
      readonly cause: unknown;
      readonly commit: CorpusCommitState;
    },
  ) {
    /**
     What git's stderr and the probe of the commit say the failure was.
     */
    const kind = kindOfFailure({
      reading: classifyCorpusReadFailure({ cause, },),
      commit,
    },);

    /**
     The closing a cut leaves whole.
     */
    const kindAndRemedy = `(${kind}); ${REMEDY_SENTENCES[kind]}`;
    super({
      says: `corpus read failed for ${detail} ${kindAndRemedy}`,
      cause,
    },);
    this.name = 'CorpusReadError';
    this.kind = kind;
    this.kindAndRemedy = kindAndRemedy;
  }
}

/**
 Whether a caught value is a corpus read that failed because the path is not
 at a commit the clone holds, which is the one failure a walk over the corpus
 may step past: an entry with one side is an ordinary state of this corpus,
 and a commit the clone lacks is not one.

 POSITIONAL, since a type predicate cannot narrow a destructured binding.

 @param error - caught value

 @returns Whether it is a missing-object corpus read failure

 @example
 ```ts
 if (!isMissingCorpusObject(error,)) throw error;
 ```
 */
export function isMissingCorpusObject(error: unknown,): error is CorpusReadError {
  return (error instanceof CorpusReadError) && (error.kind === 'missing-object');
}

/**
 Builds the refusal for a failed read, with the probe of the commit its kind
 depends on.

 A PROBE THAT GOT NO ANSWER IS KEPT BESIDE THE READ: the refusal's cause is
 then every failure, the read's first, since the kind it names (`other`)
 says nothing of why the probe failed.

 A PROBE THAT FAILED IN THIS PROCESS STOPS WITH BOTH: nano-spawn's
 preparation of the probe throws before any child exists (a working
 directory removed under this process), which is no fact about the clone, so
 no refusal is built; what stops the caller holds the read's failure first
 and the probe's after it, so neither is lost.

 @param pin - clone and commit the failed read resolved against, which the
 probe asks about

 @param gitPath - git binary the failed read ran, which the probe runs too

 @param detail - object spec or listing that failed, which the message names

 @param cause - failure the read raised, whose git stderr the kind is read from

 @returns Refusal the caller throws in place of the read's failure, its kind
 decided by the probe, so a walker steps past a page absent at a held commit
 and nothing else

 @throws AggregateError holding the read's failure and then what the probe's
 preparation threw, when the probe failed before any git child ran

 @example
 ```ts
 throw await corpusReadRefusal({ pin, gitPath, detail: spec, cause: error, },);
 ```
 */
async function corpusReadRefusal(
  {
    pin,
    gitPath,
    detail,
    cause,
  }: {
    readonly pin: CorpusPin;
    readonly gitPath: string;
    readonly detail: string;
    readonly cause: unknown;
  },
): Promise<CorpusReadError> {
  /**
   What the probe of the pinned commit came to.
   */
  const probe = await probeKeepingRead({
    pin,
    gitPath,
    cause,
  },);
  return new CorpusReadError({
    detail,
    cause: (probe.state === 'unasked')
      ? new AggregateError(
        [
          cause,
          ...probe.failures,
        ],
        'the corpus read failed, and the probe of its commit got no answer about it',
      )
      : cause,
    commit: probe.state,
  },);
}

/**
 Runs one git command against the clone, returning stdout.

 @param pin - clone and commit reads resolve against

 @param args - git argument vector, passed without shell interpretation

 @param detail - what the read means, for error reporting

 @returns Captured stdout

 @throws {@link CorpusReadError} when the git subprocess fails: a non-zero
 exit, a signal, or a spawn the system refused

 @throws {@link StatedRefusalError} when the clone's real parent path holds a
 colon, before any git child runs (`corpus-git-context.ts`)

 @throws AggregateError holding the read's failure and what the probe of the
 commit threw, when the probe failed in this process before git ran

 @throws Whatever the subprocess layer's preparation of the call throws,
 unchanged: a fault of this process, such as a working directory removed
 under it, and no fact about the corpus

 @example
 ```ts
 const out = await gitOutput({ pin, args: ['show', spec,], detail: spec, },);
 ```
 */
async function gitOutput(
  {
    pin,
    args,
    detail,
  }: {
    readonly pin: CorpusPin;
    readonly args: readonly string[];
    readonly detail: string;
  },
): Promise<string> {
  /**
   Real Git binary, resolved per call when the pin does not name one.
   Batch callers supply one resolved path because self-shim detection reads candidate files.
   */
  const gitPath = pin.gitPath ?? await resolveGit();

  /**
   Where git's search for a repository stops, resolved before the call so a
   clone refused there is refused as itself.
   */
  const searchCeiling = await gitSearchCeiling({ cloneDir: pin.cloneDir, },);

  try {
    /**
     Subprocess result; only stdout is consumed.
     */
    const { stdout, } = await spawn(
      gitPath,
      [
        ...CORPUS_GIT_FLAGS,
        '-C',
        pin.cloneDir,
        ...args,
      ],
      { env: corpusGitEnvironment({ searchCeiling, },), },
    );
    return stdout;
  }
  catch (error) {
    // ONLY A FAILURE OF THE SUBPROCESS IS A CORPUS READ FAILURE. nano-spawn
    // (2.1.0, `source/spawn.js` and `source/result.js`) hands every failure
    // of the child to its caller as `SubprocessError`: an exit code, a
    // signal, a spawn the system refused, a stream error. Its preparation of
    // the call throws raw, before any child exists and inside this `try`
    // since `spawn` throws synchronously: `source/options.js` resolves the
    // working directory through `process.cwd()`, which throws `ENOENT` once
    // the directory this process stands in is removed, and
    // `source/context.js` throws a `TypeError` for a command part that is no
    // string. Those are faults of this process, and the advice a
    // `CorpusReadError` gives, to check the clone and the pin, would misname
    // them, so they propagate as themselves. The `corpus-source.unit.test.ts`
    // case on a working directory removed under the process holds this. The
    // kind field keeps the distinction callers read: `missing-object` only
    // where git says the object is absent at a commit the probe found held,
    // and a kind of its own for a commit the clone lacks, a clone git could
    // not open, and every other subprocess failure.
    if (!(error instanceof SubprocessError))
      throw error;
    throw await corpusReadRefusal({
      pin,
      gitPath,
      detail,
      cause: error,
    },);
  }
}

/**
 Reads one file of the corpus at the pinned commit.

 LINE ENDINGS ARE FOLDED TO LF, which is the one place the whole package
 needs it: every splitter downstream looks for `\n`, and the one CRLF page in
 the pinned corpus (a source page, measured in `line-endings.ts`) defeated
 the line-structure predicate, the invisible-line mask and the quote
 normalizer at once. Bytes are otherwise untouched.

 @param pin - clone and commit the read resolves against

 @param relPath - repository-relative path, e.g. `people/<id>/page.md`

 @returns File content at the pinned commit, with CRLF folded to LF

 @throws {@link CorpusReadError} when git cannot produce the file: the path
 absent at a commit the clone holds (`missing-object`), a commit the clone
 lacks, a clone git cannot open, or another failure, each its own kind

 @throws {@link StatedRefusalError} when the clone's real parent path holds a
 colon, before any git child runs (`corpus-git-context.ts`)

 @throws AggregateError holding the read's failure and what the probe of the
 commit threw, when the probe failed in this process before git ran

 @example
 ```ts
 const zh = await readCorpusFile({ pin, relPath: 'people/whiskers/page.md', },);
 ```
 */
export async function readCorpusFile(
  {
    pin,
    relPath,
  }: {
    readonly pin: CorpusPin;
    readonly relPath: string;
  },
): Promise<string> {
  /**
   Git object spec pinning path to commit.
   */
  const spec = `${pin.commitSha}:${relPath}`;

  /**
   Real git binary, resolved when the pin does not name one.
   */
  const gitPath = pin.gitPath ?? await resolveGit();

  /**
   Where git's search for a repository stops, resolved before the call so a
   clone refused there is refused as itself and never read as a failed read.
   */
  const searchCeiling = await gitSearchCeiling({ cloneDir: pin.cloneDir, },);

  try {
    /**
     Physical blob bytes captured without any newline normalization or lazy fetch.
     */
    const { stdout, } = await execFileAsync(
      gitPath,
      [
        ...CORPUS_GIT_FLAGS,
        '-C',
        pin.cloneDir,
        'show',
        spec,
      ],
      {
        encoding: 'buffer',
        maxBuffer: MAX_BLOB_BYTES,
        env: corpusGitEnvironment({ searchCeiling, },),
      },
    );
    /**
     Content with CRLF folded to LF; the count is not logged here since this
     module carries no logger, and `line-endings.ts` records the population.
     */
    const { text, } = foldCarriageReturns({ text: stdout.toString('utf8',), },);
    return text;
  }
  catch (error) {
    throw await corpusReadRefusal({
      pin,
      gitPath,
      detail: spec,
      cause: error,
    },);
  }
}

/**
 Reads one corpus blob as BYTES at the pinned commit.

 THE BINARY SIBLING of {@link readCorpusFile}, and the reason it exists is
 that a picture is not text: decoding one as UTF-8 maps every byte sequence
 that is not valid UTF-8 onto the replacement character, which silently
 corrupts the asset and produces a data URI no model can decode.

 @param pin - corpus clone and commit

 @param relPath - repository-relative path, e.g. `people/<id>/photos/<name>`

 @returns Blob bytes exactly as committed

 @throws {@link CorpusReadError} when git cannot produce that blob

 @throws {@link StatedRefusalError} when the clone's real parent path holds a
 colon, before any git child runs (`corpus-git-context.ts`)

 @throws AggregateError holding the read's failure and what the probe of the
 commit threw, when the probe failed in this process before git ran

 @example
 ```ts
 const bytes = await readCorpusBytes({ pin, relPath: 'people/whiskers/photos/intro.webp', },);
 ```
 */
export async function readCorpusBytes(
  {
    pin,
    relPath,
  }: {
    readonly pin: CorpusPin;
    readonly relPath: string;
  },
): Promise<Uint8Array> {
  /**
   Git object spec pinning path to commit.
   */
  const spec = `${pin.commitSha}:${relPath}`;

  /**
   Real git binary, resolved when the pin does not name one.
   */
  const gitPath = pin.gitPath ?? await resolveGit();

  /**
   Where git's search for a repository stops, resolved before the call so a
   clone refused there is refused as itself and never read as a failed read.
   */
  const searchCeiling = await gitSearchCeiling({ cloneDir: pin.cloneDir, },);

  try {
    /**
     Physical blob bytes exactly as committed, without replacement refs or lazy fetch.
     */
    const { stdout, } = await execFileAsync(
      gitPath,
      [
        ...CORPUS_GIT_FLAGS,
        '-C',
        pin.cloneDir,
        'show',
        spec,
      ],
      {
        encoding: 'buffer',
        maxBuffer: MAX_BLOB_BYTES,
        env: corpusGitEnvironment({ searchCeiling, },),
      },
    );
    return stdout;
  }
  catch (error) {
    throw await corpusReadRefusal({
      pin,
      gitPath,
      detail: spec,
      cause: error,
    },);
  }
}

/**
 Lists person entry ids under `people/` at the pinned commit.

 @param pin - clone and commit the listing resolves against

 @returns Entry ids in git listing order

 @throws {@link CorpusReadError} when the clone lacks the commit, git cannot
 open the clone, or the listing fails another way

 @throws {@link StatedRefusalError} when the clone's real parent path holds a
 colon, before any git child runs (`corpus-git-context.ts`)

 @throws AggregateError holding the read's failure and what the probe of the
 commit threw, when the probe failed in this process before git ran

 @example
 ```ts
 const ids = await listCorpusPeople({ pin, },);
 ```
 */
export async function listCorpusPeople(
  { pin, }: { readonly pin: CorpusPin; },
): Promise<readonly string[]> {
  /**
   Newline-separated `people/<id>` lines from git.
   */
  const listing = await gitOutput({
    pin,
    args: [
      'ls-tree',
      '--name-only',
      pin.commitSha,
      'people/',
    ],
    detail: `people/ at ${pin.commitSha}`,
  },);

  return listing
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .map(function stripPrefix(line,) {
      return line.replace(
        'people/',
        '',
      );
    },);
}

//endregion Corpus source
