import { stat, } from 'node:fs/promises';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { spawnResult, } from './spawn.ts';

/**
 Sentinel returned by {@link originUrl} when the repo has no `origin` remote.
 */
export const NO_ORIGIN: unique symbol = Symbol('git-clone-size/origin-remote-url-absent',);

/**
 Recognised hosts with a metadata/storage API the estimator can query.
 */
export type Host = 'github' | 'gitlab' | 'unknown';

/**
 A remote source: a clone URL plus any parsed host/owner/repo used by the
 host-API signal. `host` is `unknown` for servers without a supported API.
 */
export type RemoteSource = {
  readonly kind: 'remote';
  readonly url: string;
  readonly host: Host;
  readonly owner?: string;
  readonly repo?: string;
};

/**
 A complete local repository on disk, measured exactly without cloning.
 */
export type LocalSource = {
  readonly kind: 'local';
  readonly path: string;
};

/**
 Either input shape the tool accepts.
 */
export type Source = RemoteSource | LocalSource;

/**
 Heuristic for whether an input string is a remote clone URL rather than a
 local path. Covers `scheme://` URLs and scp-like `git@host:owner/repo`.
 String scans only; no regex.
 
 @param input - raw positional argument
 
 @returns true when the input should be parsed as a remote URL
 
 @example
 ```ts
 isRemoteInput({ input: 'git\@github.com:o/r.git' }); // true
 ```
 */
export function isRemoteInput({ input, }: { readonly input: string; },): boolean {
  if (input.includes('://',))
    return true;
  /**
   Index of the first `@`, present in scp-like `user@host:path` URLs.
   */
  const atIndex = input.indexOf('@',);
  if (atIndex === (-1))
    return false;
  /**
   A scp-like URL has a `:` after the `@` and before any `/`.
   */
  const colonIndex = input.indexOf(
    ':',
    atIndex,
  );
  if (colonIndex === (-1))
    return false;
  /**
   First slash index, used to reject `user@host/path` (not scp-like).
   */
  const slashIndex = input.indexOf('/',);
  return (slashIndex === (-1)) || (colonIndex < slashIndex);
}

/**
 Classifies a hostname into a supported host bucket.
 
 @param hostname - DNS hostname from the clone URL
 
 @returns matching {@link Host}
 */
function classifyHost(hostname: string,): Host {
  /**
   Lower-cased hostname for substring matching.
   */
  const lower = hostname.toLowerCase();
  if (lower.includes('github',))
    return 'github';
  if (lower.includes('gitlab',))
    return 'gitlab';
  return 'unknown';
}

/**
 Splits a `hostname` and a `owner/repo[.git]` path fragment into a parsed
 remote, stripping a trailing `.git` and a leading slash.
 
 @param hostname - DNS hostname
 
 @param pathPart - path fragment after the host
 
 @param url - original URL, retained for cloning verbatim
 
 @returns parsed remote source
 */
function assembleRemote(
  {
    hostname,
    pathPart,
    url,
  }: {
    readonly hostname: string;
    readonly pathPart: string;
    readonly url: string
  },
): RemoteSource {
  /**
   Suffix stripped from clone paths so `owner/repo.git` yields `repo`.
   */
  const gitSuffix = '.git';
  /**
   Path fragment with a trailing `.git` removed, if present.
   */
  const withoutGit = pathPart.endsWith(gitSuffix,) ? pathPart.slice(
    0,
    -gitSuffix.length,
  ) : pathPart;
  /**
   Path segments with empties dropped.
   */
  const segments = withoutGit
    .split('/',)
    .filter(function nonEmpty(segment,) {
      return segment !== '';
    },);
  /**
   Owner is the first segment; repo is the last, when present.
   */
  const owner = segments.at(0,);
  /**
   Repo name is the final segment, allowing nested group paths on GitLab.
   */
  const repo = segments.at(-1,);
  return {
    kind: 'remote',
    url,
    host: classifyHost(hostname,),
    ...owner === undefined ? {} : { owner, },
    ...repo === undefined ? {} : { repo, },
  };
}

/**
 Parses a remote clone URL (scheme or scp-like) into host/owner/repo. The
 original URL is preserved verbatim so probes clone exactly what was asked.
 
 @param url - remote clone URL
 
 @returns parsed remote source; host `unknown` when unrecognised
 
 @example
 ```ts
 parseRemoteUrl({ url: 'https://github.com/o/r.git' });
 // { kind: 'remote', host: 'github', owner: 'o', repo: 'r', url }
 ```
 */
export function parseRemoteUrl({ url, }: { readonly url: string; },): RemoteSource {
  /**
   Tagged logger naming remote URL parsing.
   */
  const rl = tagged({
    tag: parseRemoteUrl.name,
    l: logger,
  },);
  if (url.includes('://',)) {
    try {
      /**
       Parsed URL for scheme-based remotes.
       */
      const parsed = new URL(url,);
      return assembleRemote({
        hostname: parsed.hostname,
        pathPart: parsed.pathname,
        url,
      },);
    }
    catch (error: unknown) {
      rl.debug(`scheme remote URL parse failed: ${String(error,)}`,);
      return {
        kind: 'remote',
        url,
        host: 'unknown',
      };
    }
  }
  /**
   scp-like `user@host:owner/repo.git`: host sits between `@` and `:`.
   */
  const atIndex = url.indexOf('@',);
  /**
   Colon separating host from path in scp-like syntax.
   */
  const colonIndex = url.indexOf(
    ':',
    atIndex,
  );
  if ((atIndex === (-1)) || (colonIndex === (-1)))
    return {
      kind: 'remote',
      url,
      host: 'unknown',
    };
  return assembleRemote({
    hostname: url.slice(
      atIndex + 1,
      colonIndex,
    ),
    pathPart: url.slice(colonIndex + 1,),
    url,
  },);
}

/**
 Whether a local repo is a shallow clone, which lacks full history and so is
 not a valid full-clone reference.
 
 @param path - repository working directory or git dir
 
 @returns true when `git rev-parse --is-shallow-repository` reports true
 
 @example
 ```ts
 const shallow = await isShallowRepo({ path: '/repo' });
 ```
 */
export async function isShallowRepo({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   Captured `--is-shallow-repository` output and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      path,
      'rev-parse',
      '--is-shallow-repository',
    ],
  },);
  return (exitCode === 0) && (stdout === 'true');
}

/**
 Whether a local repo is a partial clone (blob/tree filter), which also lacks
 objects a full clone would carry.
 
 @param path - repository working directory or git dir
 
 @returns true when `extensions.partialClone` is configured
 
 @example
 ```ts
 const partial = await isPartialRepo({ path: '/repo' });
 ```
 */
export async function isPartialRepo({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   Captured `extensions.partialClone` config value and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      path,
      'config',
      '--get',
      'extensions.partialClone',
    ],
  },);
  return (exitCode === 0) && (stdout !== '');
}

/**
 Reads a local repo's `origin` remote URL, used to fall through to the remote
 estimator when the local store is itself incomplete.
 
 @param path - repository working directory or git dir
 
 @returns origin URL, or {@link NO_ORIGIN} when there is no origin remote
 
 @example
 ```ts
 const url = await originUrl({ path: '/repo' });
 ```
 */
export async function originUrl(
  { path, }: { readonly path: string; },
): Promise<string | typeof NO_ORIGIN> {
  /**
   Captured `origin` remote URL and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      path,
      'remote',
      'get-url',
      'origin',
    ],
  },);
  if ((exitCode !== 0) || (stdout === ''))
    return NO_ORIGIN;
  return stdout;
}

/**
 Whether a filesystem path exists, swallowing the stat error as a false.
 
 @param path - candidate path
 
 @returns true when `stat` resolves, false when it throws
 
 @example
 ```ts
 const exists = await pathExists({ path: '/repo' });
 ```
 */
async function pathExists({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   Tagged logger naming path existence checks.
   */
  const rl = tagged({
    tag: pathExists.name,
    l: logger,
  },);
  try {
    await stat(path,);
    return true;
  }
  catch (error: unknown) {
    rl.debug(`path existence check failed: ${String(error,)}`,);
    return false;
  }
}

/**
 Whether a path is inside a git repository.
 
 @param path - candidate directory
 
 @returns true when `git -C path rev-parse --git-dir` succeeds
 
 @example
 ```ts
 const isRepo = await isGitRepo({ path: '/repo' });
 ```
 */
async function isGitRepo({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   Exit code of the `rev-parse --git-dir` probe; 0 inside a repository.
   */
  const { exitCode, } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      path,
      'rev-parse',
      '--git-dir',
    ],
  },);
  return exitCode === 0;
}

/**
 Classifies the input into a {@link Source}. A remote URL parses directly. A
 local path that is a complete repo measures exactly; a shallow or partial
 local repo falls through to its `origin` URL, since an incomplete store is
 not a valid full-clone reference. A non-repo, non-URL string is treated as a
 remote URL attempt rather than refused.
 
 @param input - positional argument, or undefined to default to `cwd`
 
 @param cwd - directory used when `input` is omitted
 
 @returns resolved source descriptor
 
 @example
 ```ts
 await detectSource({ input: undefined, cwd: process.cwd() }); // local cwd
 ```
 */
export async function detectSource(
  {
    input,
    cwd,
  }: {
    readonly input?: string;
    readonly cwd: string;
  },
): Promise<Source> {
  /**
   Tagged logger naming source classification.
   */
  const rl = tagged({
    tag: detectSource.name,
    l: logger,
  },);

  if ((input !== undefined) && isRemoteInput({ input, },)) {
    rl.debug(`remote URL input: ${input}`,);
    return parseRemoteUrl({ url: input, },);
  }

  /**
   Local candidate path: the explicit input or the current directory.
   */
  const candidate = input ?? cwd;

  /**
   Whether the candidate path exists on disk.
   */
  const candidateExists = await pathExists({ path: candidate, },);

  if ((!candidateExists) || (!(await isGitRepo({ path: candidate, },)))) {
    rl.debug(`non-repo input treated as remote URL: ${candidate}`,);
    return parseRemoteUrl({ url: candidate, },);
  }

  if (await isShallowRepo({ path: candidate, },) || await isPartialRepo({ path: candidate, },)) {
    /**
     Origin URL for the incomplete local repo, when present.
     */
    const origin = await originUrl({ path: candidate, },);
    if (origin !== NO_ORIGIN) {
      rl.debug(`local repo is shallow/partial; using origin ${origin}`,);
      return parseRemoteUrl({ url: origin, },);
    }
    rl.debug('local repo is shallow/partial with no origin; measuring incomplete store',);
  }

  rl.debug(`complete local repo: ${candidate}`,);
  return {
    kind: 'local',
    path: candidate,
  };
}
