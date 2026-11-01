import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CacheVersion,
  citedHash,
} from './cache-account-read.ts';

//region Cache account commits
// WHICH COMMITS THE VERSION ACCOUNTS NAME (ledger M28). A commit that changed
// a stage's source after its cache version last moved either is named in that
// version's account or rides inside it unaccounted, and the pre-launch check
// reads every unnamed one. Pure: the entry (`cache-account-audit.ts`) asks git
// and hands the lines here.
//
// TIMES ARE UNIX SECONDS. The scratch script this replaces compared committer
// dates as ISO strings, which orders two commits wrongly once their offsets
// differ; every commit on the branch carried one offset when this moved in,
// so no answer it gave was wrong, but the next contributor's could be.

/**
 Field separator in the log lines this reads, as `%x09` writes it.
 */
const FIELD_SEPARATOR = '\t';

/**
 One source commit, as the log format `%H%x09%ct%x09%s` writes it.

 @example
 ```ts
 const commit: SourceCommit = { hash: 'f6e93ed5f…', seconds: 1_790_000_000, subject: 'fix(…): …', };
 ```
 */
export type SourceCommit = {
  /**
   Full object id.
   */
  readonly hash: string;

  /**
   Committer time in unix seconds.
   */
  readonly seconds: number;

  /**
   Subject line.
   */
  readonly subject: string;
};

/**
 One pickaxe candidate for the commit that set a version, with how many
 declarations of its current value the commit added and removed.

 @example
 ```ts
 const candidate: SettingCandidate = { commit, added: 1, removed: 0, };
 ```
 */
export type SettingCandidate = {
  /**
   Commit git's pickaxe named.
   */
  readonly commit: SourceCommit;

  /**
   Declarations of the current value it added.
   */
  readonly added: number;

  /**
   And removed.
   */
  readonly removed: number;
};

/**
 One version with the commit that set its current value and the text its
 account lives in.

 @example
 ```ts
 const setting: VersionSetting = { version, commit, account: fileText, };
 ```
 */
export type VersionSetting = {
  /**
   Constant as the source declares it.
   */
  readonly version: CacheVersion;

  /**
   Commit that set its current value.
   */
  readonly commit: SourceCommit;

  /**
   Text of the file declaring it, whose TSDoc holds its account.
   */
  readonly account: string;
};

/**
 One source commit no account names, with the versions set before it, which
 it rides inside.

 @example
 ```ts
 const unnamed: UnaccountedCommit = { commit, setBefore: ['PAIRING_CACHE_VERSION',], };
 ```
 */
export type UnaccountedCommit = {
  /**
   Commit no account names.
   */
  readonly commit: SourceCommit;

  /**
   Names of the versions whose current value it postdates.
   */
  readonly setBefore: readonly string[];
};

/**
 A log line that does not read as `%H%x09%ct%x09%s`.

 @example
 ```ts
 throw new CacheAccountLogError({ line: 'abc', },);
 ```
 */
export class CacheAccountLogError extends Error {
  /**
   Declares this message safe to forward: it quotes a line git wrote about
   this package's own history, never corpus text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal quoting the line.

   @param line - log line as git wrote it

   @example
   ```ts
   new CacheAccountLogError({ line: 'abc', },);
   ```
   */
  constructor({ line, }: { readonly line: string; },) {
    super(`git log wrote a line the cache account audit cannot read: "${line}"`,);
    this.name = 'CacheAccountLogError';
  }
}

/**
 Reads one log line into a commit.

 @param line - `%H%x09%ct%x09%s` output for one commit; the subject may hold
 tabs of its own

 @returns The commit

 @throws CacheAccountLogError where the line lacks a hash, a time in whole
 seconds or a subject field

 @example
 ```ts
 sourceCommitOf({ line: 'f6e93ed5f…\t1790000000\tfix: …', },);
 ```
 */
export function sourceCommitOf({ line, }: { readonly line: string; },): SourceCommit {
  /**
   Hash, time, and the subject with any tab it holds.
   */
  const [
    hash,
    seconds,
    ...subject
  ] = line.split(FIELD_SEPARATOR,);
  if ((hash === undefined)
    || (hash === '')
    || (seconds === undefined))
    throw new CacheAccountLogError({ line, },);
  /**
   Whether the time is whole seconds and a subject follows it.
   */
  const readable = isWholeNumberText({ text: seconds, },) && (subject.length > 0);
  if (!readable)
    throw new CacheAccountLogError({ line, },);
  return {
    hash,
    seconds: Number(seconds,),
    subject: subject.join(FIELD_SEPARATOR,),
  };
}

/**
 What the pickaxe candidates say set a version's value: a commit, or none,
 which is a value no commit holds yet.

 @example
 ```ts
 const reading: SettingReading = { kind: 'uncommitted', };
 ```
 */
export type SettingReading =
  | {
    readonly kind: 'set';

    /**
     Commit that set the value.
     */
    readonly commit: SourceCommit;
  }
    | { readonly kind: 'uncommitted'; };

/**
 The commit that set a version's current value: the newest candidate that
 added its declaration on balance.

 @param candidates - pickaxe candidates, newest first

 @returns That commit, or uncommitted where none added more than it removed

 @example
 ```ts
 settingCommit({ candidates, },);
 ```
 */
export function settingCommit(
  { candidates, }: { readonly candidates: readonly SettingCandidate[]; },
): SettingReading {
  /**
   Newest candidate that added the declaration on balance.
   */
  const setting = candidates.find(function setsValue(candidate,): boolean {
    return candidate.added > candidate.removed;
  },);
  if (setting === undefined)
    return { kind: 'uncommitted', };
  return {
    kind: 'set',
    commit: setting.commit,
  };
}

/**
 Source commits no version account names, each with the versions set before
 it.

 @param settings - every version with the commit that set it and its account

 @param commits - source commits since the earliest setting

 @returns The commits no account cites by their first nine characters, in
 the order given

 @example
 ```ts
 unaccountedCommits({ settings, commits, },);
 ```
 */
export function unaccountedCommits(
  {
    settings,
    commits,
  }: {
    readonly settings: readonly VersionSetting[];
    readonly commits: readonly SourceCommit[];
  },
): readonly UnaccountedCommit[] {
  return commits
    .filter(function unnamed(commit,): boolean {
      /**
       Hash as the accounts cite it.
       */
      const cited = citedHash({ hash: commit.hash, },);
      return !settings.some(function names(setting,): boolean {
        return setting.account
          .includes(cited,);
      },);
    },)
    .map(function withVersions(commit,): UnaccountedCommit {
      return {
        commit,
        setBefore: settings
          .filter(function setEarlier({ commit: setAt, },): boolean {
            return commit.seconds > setAt.seconds;
          },)
          .map(function nameOf(setting,): string {
            return setting.version
              .name;
          },),
      };
    },);
}

/**
 Milliseconds in a second.
 */
export const MS_PER_SECOND = 1_000;

/**
 Characters of an ISO time up to its minutes.
 */
const ISO_MINUTES_LENGTH = 16;

/**
 A commit or file time as UTC to the minute, the precision the accounts cite.

 @param seconds - unix seconds

 @returns ISO time ending in Z

 @example
 ```ts
 utcMinutes({ seconds: 0, },); // '1970-01-01T00:00Z'
 ```
 */
export function utcMinutes({ seconds, }: { readonly seconds: number; },): string {
  /**
   Full ISO time, milliseconds and all.
   */
  const iso = new Date(seconds * MS_PER_SECOND,)
    .toISOString();
  return `${iso.slice(
    0,
    ISO_MINUTES_LENGTH,
  )}Z`;
}

//endregion Cache account commits
