/**
 Git-derived publication and update dates for MDX posts.
 
 Probes `git log --follow` for the oldest and newest commits touching
 each post file. Falls back to file mtime when the file is untracked
 or uncommitted. When the local repository is a shallow clone,
 the oldest commit is fetched via the GitHub REST API through `gh`
 because shallow history does not contain it.
 
 Authorship note: dev previews of pre-commit posts will show mtime-based
 dates; first commit snaps both dates to commit author timestamps.
 Authoring without committing is undefined behavior.
 */
import { stat, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import spawn, { type Result, } from 'nano-spawn';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { Logger, } from './types.ts';

//region Helpers

/**
 Sentinel marking a git or GitHub probe that produced no value: no GitHub
 remote, no commit date from the API, or no local commit history. A real,
 unique symbol rather than the empty string, so an absent value can never be
 confused with a genuine (if implausible) empty date or slug.
 */
export const ABSENT: unique symbol = Symbol('git dates probe returned nothing',);

/**
 Result of running a git (or gh) command with stdout captured as UTF-8.
 */
type CommandResult = Pick<Result, 'stdout' | 'stderr'>;

/**
 Executes a command, returning captured stdout and stderr strings.
 
 Wraps `nano-spawn` so callers see a narrow return type containing only
 the stream fields they consume.
 
 @param cmd - executable name or absolute path
 
 @param args - argument vector passed without shell interpretation
 
 @returns captured stdout and stderr as strings
 
 @throws when the command exits with a non-zero status
 
 @example
 ```ts
 const { stdout } = await runCapture({ cmd: 'git', args: ['rev-parse', 'HEAD'] });
 ```
 */
async function runCapture(
  {
    cmd,
    args,
    cwd,
  }: {
    readonly cmd: string;
    readonly args: readonly string[];
    readonly cwd?: string;
  },
): Promise<CommandResult> {
  /**
   Destructured spawn result; only stdout and stderr are forwarded to the caller.
   */
  const {
    stdout,
    stderr,
  } = await spawn(
    cmd,
    args,
    { cwd, },
  );
  return {
    stdout,
    stderr,
  };
}

/**
 Runs git with the cwd pinned to the repository root.
 
 @param args - git subcommand and arguments
 
 @returns captured stdout and stderr
 
 @example
 ```ts
 const { stdout } = await runGit(['rev-parse', 'HEAD']);
 ```
 */
async function runGit(args: readonly string[],): Promise<CommandResult> {
  /**
   Repository root cached lookup pinning the git cwd.
   */
  const root = await findRootCached({ marker: MISE_MONOREPO, },);
  return runCapture({
    cmd: 'git',
    args,
    cwd: root,
  },);
}

//endregion Helpers

//region Repository probes

/**
 Reads the current `HEAD` commit SHA of the working repository.
 
 Used as a coarse invalidation key for cached git-derived dates: when the
 SHA matches the one stored in the build manifest, cached dates are still
 accurate and no per-file git probing is needed.
 
 @returns 40-character hexadecimal commit SHA
 
 @example
 ```ts
 const sha = await getHeadSha();
 ```
 */
export async function getHeadSha(): Promise<string> {
  /**
   Captured stdout from `git rev-parse HEAD` containing the 40-char SHA plus newline.
   */
  const { stdout, } = await runGit([
    'rev-parse',
    'HEAD',
  ],);
  return stdout.trim();
}

/**
 Checks whether the current repository is a shallow clone.
 
 Shallow clones lack the oldest commits that `git log --reverse --follow`
 needs to derive the publication date. When shallow, {@link getPostDates} falls
 back to the GitHub REST API through `gh` for the "published" lookup.
 
 @returns `true` when the repository is shallow, `false` otherwise
 
 @example
 ```ts
 const shallow = await detectShallow();
 ```
 */
export async function detectShallow(): Promise<boolean> {
  /**
   Captured stdout from `git rev-parse --is-shallow-repository`; trimmed to `true`/`false`.
   */
  const { stdout, } = await runGit([
    'rev-parse',
    '--is-shallow-repository',
  ],);
  return stdout.trim()
    === 'true';
}

/**
 Parses `owner/repo` from the `origin` remote URL.
 
 Accepts both SSH (`git@github.com:owner/repo.git`) and HTTPS
 (`https://github.com/owner/repo.git`) remotes. Strips the optional
 trailing `.git` suffix.
 
 @returns `owner/repo` identifier, or {@link ABSENT} when the remote is not GitHub
 
 @example
 ```ts
 const slug = await getGithubSlug(); // 'Aquaticat/Monochromatic'
 ```
 */
async function getGithubSlug(): Promise<string | typeof ABSENT> {
  try {
    /**
     Captured stdout from `git remote get-url origin`; may be SSH or HTTPS form.
     */
    const { stdout, } = await runGit([
      'remote',
      'get-url',
      'origin',
    ],);
    /**
     Trimmed remote URL fed to the slug regex.
     */
    const url = stdout.trim();
    /* oxlint-disable no-restricted-syntax/no-regex -- anchored at `$` to parse the trailing `owner/repo[.git]` slug from either SSH or HTTPS GitHub URLs. Input is `git remote get-url` output (bounded URL length); `[^/]+` and `[^/.]+` are negated classes so each matches its own slice with no overlap, no backtracking risk. */
    /**
     Regex capture extracting `owner/repo` from either supported URL form.
     */
    const match = /github\.com[:/](?<slug>[^/]+\/[^/.]+)(?:\.git)?$/u.exec(url,);
    /* oxlint-enable no-restricted-syntax/no-regex */
    return match?.groups
      ?.slug
      ?? ABSENT;
  }
  catch (error) {
    tagged({
      tag: getGithubSlug.name,
      l: logger,
    },)
      .debug(`origin remote not resolvable as a GitHub slug; treating as absent (${String(error,)})`,);
    return ABSENT;
  }
}

//endregion Repository probes

//region Local git lookups

/**
 Runs `git log --follow` and returns commit author dates as ISO-8601 strings
 in reverse-chronological order (newest first, oldest last).
 
 `--follow` traces the file across renames so a post that moved directories
 still reports its original first-commit date.
 
 Intentionally fetches the full history rather than combining `--reverse`
 with `-n 1`: git applies `-n` during traversal before `--reverse` reverses
 the already-limited set, so `--reverse -n 1` silently returns no lines.
 For blog posts with modest commit counts, fetching all lines is negligible.
 
 @param filePath - path to the file whose history to query
 
 @returns array of ISO-8601 author dates newest-first, empty when the file has no history
 
 @example
 ```ts
 const dates = await gitLogDates({ filePath: 'post.mdx' });
 // dates[0] is newest, dates.at(-1) is oldest
 ```
 */
async function gitLogDates(
  { filePath, }: { readonly filePath: string; },
): Promise<string[]> {
  /**
   Absolute file path passed to git so `--follow` can trace renames consistently.
   */
  const absolute = resolve(
    process.cwd(),
    filePath,
  );
  /**
   Captured stdout: newline-separated ISO-8601 author dates newest-first.
   */
  const { stdout, } = await runGit([
    'log',
    '--follow',
    '--format=%aI',
    '--',
    absolute,
  ],);
  return stdout.split('\n',)
    .filter(function keepNonEmpty(line,) {
      return line.trim()
        .length
        > 0;
    },);
}

//endregion Local git lookups

//region GitHub API fallback

/**
 Resolves the repository-relative path for a file, as required by the
 GitHub REST API's `path` query parameter.
 
 @param filePath - absolute or cwd-relative path to the file
 
 @returns path relative to the repository root using forward slashes
 
 @example
 ```ts
 const rel = await getRepoRelativePath('src/content/en/post.mdx');
 // → 'package/ssg/aquati.cat/src/content/en/post.mdx'
 ```
 */
async function getRepoRelativePath(filePath: string,): Promise<string> {
  /**
   Repository root cached lookup pinning the git cwd.
   */
  const root = await findRootCached({ marker: MISE_MONOREPO, },);
  /**
   Absolute path normalised before handing to `git ls-files --full-name`.
   */
  const absolute = resolve(
    process.cwd(),
    filePath,
  );
  /**
   Captured stdout containing the repo-relative path with forward slashes.
   */
  const { stdout, } = await runCapture({
    cmd: 'git',
    args: [
      'ls-files',
      '--full-name',
      '--',
      absolute,
    ],
    cwd: root,
  },);
  return stdout.trim();
}

/**
 Fetches the oldest commit author date for a file via the GitHub REST API.
 
 Pages through `GET /repos/{owner}/{repo}/commits?path=<path>` with
 `--paginate`; the response is ordered newest-first, so the last entry
 is the oldest commit that touched the path.
 
 @param slug - GitHub `owner/repo` identifier
 
 @param repoRelPath - path relative to the repository root
 
 @returns ISO-8601 author date string, or {@link ABSENT} when the API returns
 no commits for the path (e.g., path exists only in a local branch)
 
 @example
 ```ts
 const first = await ghApiFirstCommitDate({
   slug: 'Aquaticat/Monochromatic',
   repoRelPath: 'package/ssg/aquati.cat/src/content/en/post.mdx',
 });
 ```
 */
async function ghApiFirstCommitDate(
  {
    slug,
    repoRelPath,
  }: {
    readonly slug: string;
    readonly repoRelPath: string;
  },
): Promise<string | typeof ABSENT> {
  /**
   Captured stdout from `gh api` containing concatenated JSON pages.
   */
  const { stdout, } = await runCapture({
    cmd: 'gh',
    args: [
      'api',
      '--paginate',
      `repos/${slug}/commits?path=${encodeURIComponent(repoRelPath,)}&per_page=100`,
    ],
  },);

  /* `--paginate` concatenates JSON arrays with no separator between pages.
   * Parse by splitting on `][` and re-bracketing; single-page output parses
   * directly as JSON. */
  /**
   Trimmed JSON payload returned from `gh api --paginate`.
   */
  const raw = stdout.trim();
  if (raw.length
    === 0)
    return ABSENT;

  /**
   Each commit object from the GitHub REST API exposes the relevant
   `commit.author.date` field; other fields are ignored.
   */
  type GhCommit = { readonly commit: { readonly author: { readonly date: string; }; }; };

  /* oxlint-disable no-unsafe-type-assertion -- `gh api` JSON output is untyped; GhCommit mirrors the documented GitHub REST commits response and is read structurally below via `last?.commit.author.date`. */
  /**
   Flat list of commits across all returned pages.
   */
  const commits: GhCommit[] = raw.includes('][',)
    ? raw
      .split('][',)
      .flatMap(function parseChunk(
        chunk,
        i,
        arr: readonly string[],
      ) {
        /**
         Opening bracket re-inserted on every chunk except the first to rebuild the JSON array boundary.
         */
        const prefix = i === 0 ? '' : '[';
        /**
         Closing bracket re-inserted on every chunk except the last to rebuild the JSON array boundary.
         */
        const suffix = i === (arr.length
          - 1) ? '' : ']';
        return JSON.parse(`${prefix}${chunk}${suffix}`,) as GhCommit[];
      },)
    : (JSON.parse(raw,) as GhCommit[]);
  /* oxlint-enable no-unsafe-type-assertion */

  /**
   Oldest commit in the API response (last entry per GitHub's newest-first ordering).
   */
  const last = commits.at(-1,);
  return last?.commit
    .author
    .date
    ?? ABSENT;
}

//endregion GitHub API fallback

//region Public API

/**
 Dates derived for a single post from git history, with fallbacks.
 */
type PostDates = {
  /**
   Author date of the oldest commit touching the file.
   */
  published: Date;
  /**
   Author date of the newest commit touching the file.
   */
  updated: Date;
};

/**
 Resolves `published` and `updated` dates for a single post.
 
 Strategy:
 - `updated`: newest commit author date from local `git log --follow`
 - `published`: oldest commit author date from local `git log --follow --reverse`,
   or GitHub REST API when the clone is shallow
 - Fallback on both; file mtime when the file has no git history (untracked or uncommitted)
 
 @param filePath - absolute or cwd-relative path to the MDX file
 
 @param isShallow - pre-computed shallow-clone flag (from {@link detectShallow})
 
 @param githubSlug - pre-resolved `owner/repo` ({@link ABSENT} when origin is not GitHub); required when shallow
 
 @param l - tagged logger used for fallback diagnostics
 
 @returns derived `published` and `updated` dates
 
 @throws when the repository is shallow but no GitHub slug is available
 
 @example
 ```ts
 const dates = await getPostDates({
   filePath: 'src/content/en/post.mdx',
   isShallow: false,
   githubSlug: ABSENT,
   l,
 });
 ```
 */
export async function getPostDates(
  {
    filePath,
    isShallow,
    githubSlug,
    l,
  }: {
    readonly filePath: string;
    readonly isShallow: boolean;
    readonly githubSlug: string | typeof ABSENT;
    readonly l: Logger;
  },
): Promise<PostDates> {
  /**
   Newest-first list of author dates for every commit touching the file.
   */
  const localHistory = await gitLogDates({ filePath, },);
  /**
   Author date of the most recent commit; {@link ABSENT} when the file has no git history.
   */
  const latestIso = localHistory[0]
    ?? ABSENT;

  /**
   Final `published` ISO string, resolved through shallow/gh fallback when needed.
   Shallow clones omit the true first commit, so the local oldest is unreliable
   when `isShallow` is set; the gh-api fallback replaces it in that case.
   */
  const publishedIso =
    await (async function resolvePublishedIso(): Promise<string | typeof ABSENT> {
      if (!isShallow)
        return localHistory.at(-1,)
          ?? ABSENT;
      if (latestIso === ABSENT)
        return ABSENT;
      if (githubSlug === ABSENT) {
        throw new Error(
          `Shallow clone detected but no GitHub remote configured; cannot resolve published date for ${filePath}. Fetch with --unshallow or configure an origin on github.com.`,
        );
      }
      /**
       Repo-relative form required by the GitHub commits API `path` query param.
       */
      const repoRelPath = await getRepoRelativePath(filePath,);
      /**
       ISO date returned by the gh-api fallback; {@link ABSENT} when the API has no commits for the path.
       */
      const ghIso = await ghApiFirstCommitDate({
        slug: githubSlug,
        repoRelPath,
      },);
      l.info(
        `shallow clone: resolved published date for ${filePath} via gh api`,
      );
      return ghIso;
    })();

  if ((latestIso !== ABSENT) && (publishedIso !== ABSENT)) {
    return {
      published: new Date(publishedIso,),
      updated: new Date(latestIso,),
    };
  }

  /**
   File mtime fallback for untracked or uncommitted files.
   */
  const stats = await stat(filePath,);
  /**
   Date-typed mtime used to fill in whichever field is missing from git history.
   */
  const mtime = new Date(stats.mtimeMs,);
  /**
   Names of fields filled in from mtime, surfaced in the diagnostic log line.
   */
  const missing: string[] = [];
  if (publishedIso === ABSENT)
    missing.push('published',);
  if (latestIso === ABSENT)
    missing.push('updated',);
  l.info(
    `git history incomplete for ${filePath} (missing ${
      missing.join(', ',)
    }); falling back to file mtime for missing fields`,
  );
  return {
    published: publishedIso !== ABSENT ? new Date(publishedIso,) : mtime,
    updated: latestIso !== ABSENT ? new Date(latestIso,) : mtime,
  };
}

/**
 Pre-resolved repository-level context shared across all post-date lookups.
 
 One instance is built at the start of a build and threaded through
 `loadContent` and cache-overlay paths so per-file work is bounded
 to at most two git invocations plus one optional `gh api` call.
 */
export type GitDatesContext = {
  /**
   Current `HEAD` commit SHA.
   */
  readonly headSha: string;
  /**
   Whether the repository is a shallow clone.
   */
  readonly isShallow: boolean;
  /**
   GitHub `owner/repo` slug when origin is on github.com; {@link ABSENT} when origin is not GitHub.
   */
  readonly githubSlug: string | typeof ABSENT;
};

/**
 Resolves the shared git-dates context for the current build.
 
 Runs all repository-level probes concurrently.
 
 @returns populated git-dates context
 
 @example
 ```ts
 const ctx = await resolveGitDatesContext();
 ```
 */
export async function resolveGitDatesContext(): Promise<GitDatesContext> {
  /**
   Repository-level probes gathered concurrently into a single context object.
   */
  const [headSha, isShallow, githubSlug,] = await Promise.all([
    getHeadSha(),
    detectShallow(),
    getGithubSlug(),
  ],);
  return {
    headSha,
    isShallow,
    githubSlug,
  };
}

//endregion Public API
