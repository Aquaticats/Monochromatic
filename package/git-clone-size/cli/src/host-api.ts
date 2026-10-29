import { BYTES_PER_KIB, } from '@monochromatic-dev/module-const/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { spawnResult, } from './spawn.ts';
import type { RemoteSource, } from './source.ts';

/**
 Sentinel returned by {@link lsRemote} when the ref listing fails.
 */
export const NO_REFS: unique symbol = Symbol('git-clone-size/ls-remote-ref-listing-failed',);

/**
 Sentinel returned by {@link hostStorageBytes} when no storage proxy is available.
 */
export const NO_STORAGE: unique symbol = Symbol('git-clone-size/host-storage-proxy-unavailable',);

/**
 Sentinel returned by {@link hostCommitCount} when the host commit count is unavailable.
 */
export const NO_HOST_COMMITS: unique symbol = Symbol('git-clone-size/host-commit-count-unavailable',);

/**
 Sentinel returned by {@link parseLastPage} when no `rel="last"` page exists.
 */
const NO_LAST_PAGE: unique symbol = Symbol('git-clone-size/github-link-last-page-absent',);

/**
 Ref inventory from `git ls-remote`, used for the branch-coverage correction
 and as a weak activity signal.
 */
export type LsRemoteResult = {
  readonly branches: number;
  readonly tags: number;
  readonly defaultBranch?: string;
};

/**
 Host-reported storage size mapped to bytes, flagged as a proxy (server
 storage, not the client-side packed object store).
 */
export type HostStorageResult = {
  readonly bytes: number;
};

/**
 Commit count from a host API, with a lower-bound flag when pagination did not
 expose a last-page number.
 */
export type HostCommitCountResult = {
  readonly count: number;
  readonly lowerBound: boolean;
};

/**
 Counts heads and tags and resolves the default branch via `git ls-remote`.
 Always cheap (refs only, no objects).
 
 @param url - remote clone URL
 
 @returns branch/tag counts and the default branch name when resolvable
 
 @example
 ```ts
 const refs = await lsRemote({ url });
 ```
 */
export async function lsRemote(
  { url, }: { readonly url: string; },
): Promise<LsRemoteResult | typeof NO_REFS> {
  /**
   Tagged logger naming the ls-remote signal.
   */
  const rl = tagged({
    tag: lsRemote.name,
    l: logger,
  },);

  /**
   Captured `ls-remote --heads --tags` listing and exit code.
   */
  const refs = await spawnResult({
    command: 'git',
    args: [
      'ls-remote',
      '--heads',
      '--tags',
      url,
    ],
  },);
  if (refs.exitCode !== 0) {
    rl.debug(`ls-remote failed: ${refs.stderr}`,);
    return NO_REFS;
  }
  /**
   Non-empty ref lines from the listing.
   */
  const lines = refs.stdout
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.includes('refs/',);
    },);
  /**
   Head refs (branches).
   */
  const branches = lines.filter(function isHead(line,) {
    return line.includes('refs/heads/',);
  },)
    .length;
  /**
   Tag refs, excluding peeled `^{}` duplicate lines.
   */
  const tags = lines.filter(function isTag(line,) {
    return line.includes('refs/tags/') && (!line.includes('^{}',));
  },)
    .length;

  /**
   Default branch from the symbolic HEAD ref.
   */
  const head = await spawnResult({
    command: 'git',
    args: [
      'ls-remote',
      '--symref',
      url,
      'HEAD',
    ],
  },);
  /**
   `ref: refs/heads/<name>\tHEAD` line, when present.
   */
  const symrefLine = head.stdout
    .split('\n',)
    .find(function isSymref(line,) {
      return line.startsWith('ref:',);
    },);
  /**
   Default branch name parsed from the symref line.
   */
  const defaultBranch = symrefLine === undefined
    ? undefined
    : symrefLine.split('refs/heads/',)
      .at(1,)
      ?.split('\t',)
      .at(0,)
      ?.trim();

  rl.debug(`ls-remote: ${String(branches,)} branches, ${String(tags,)} tags, default=${defaultBranch ?? '?'}`,);
  return {
    branches,
    tags,
    ...(defaultBranch === undefined) || (defaultBranch === '') ? {} : { defaultBranch, },
  };
}

/**
 Reads host-reported repository storage as a clone-size proxy. GitHub exposes
 `.size` in KiB via `gh`; GitLab exposes `.statistics.repository_size` in bytes
 via `glab` (requires Reporter+). Returns undefined for unsupported hosts or
 any failure (missing CLI, auth, private repo), never refusing.
 
 @param source - parsed remote with host/owner/repo
 
 @returns storage bytes proxy, or undefined when unavailable
 
 @example
 ```ts
 const storage = await hostStorageBytes({ source });
 ```
 */
export async function hostStorageBytes(
  { source, }: { readonly source: RemoteSource; },
): Promise<HostStorageResult | typeof NO_STORAGE> {
  /**
   Tagged logger naming the host storage signal.
   */
  const rl = tagged({
    tag: hostStorageBytes.name,
    l: logger,
  },);
  if ((source.owner === undefined) || (source.repo === undefined))
    return NO_STORAGE;

  if (source.host === 'github') {
    /**
     Captured `gh api repos/{o}/{r} --jq .size` output.
     */
    const result = await spawnResult({
      command: 'gh',
      args: [
        'api',
        `repos/${source.owner}/${source.repo}`,
        '--jq',
        '.size',
      ],
    },);
    /**
     GitHub `.size` is KiB; convert to bytes.
     */
    const kib = Math.trunc(Number(result.stdout,),);
    if ((result.exitCode === 0) && Number.isFinite(kib,)) {
      rl.debug(`github storage proxy: ${String(kib,)} KiB`,);
      return { bytes: kib * BYTES_PER_KIB, };
    }
    return NO_STORAGE;
  }

  if (source.host === 'gitlab') {
    /**
     URL-encoded `owner/repo` project id for the GitLab API.
     */
    const projectId = `${source.owner}%2F${source.repo}`;
    /**
     Captured `glab api projects/{id}?statistics=true` output.
     */
    const result = await spawnResult({
      command: 'glab',
      args: [
        'api',
        `projects/${projectId}?statistics=true`,
        '--jq',
        '.statistics.repository_size',
      ],
    },);
    /**
     GitLab `repository_size` is already bytes.
     */
    const bytes = Math.trunc(Number(result.stdout,),);
    if ((result.exitCode === 0) && Number.isFinite(bytes,)) {
      rl.debug(`gitlab storage proxy: ${String(bytes,)} bytes`,);
      return { bytes, };
    }
    return NO_STORAGE;
  }

  return NO_STORAGE;
}

/**
 Extracts the `rel="last"` page number from an HTTP `Link` header value.
 
 @param linkHeader - raw Link header content
 
 @returns last-page number, or {@link NO_LAST_PAGE} when no `rel="last"` segment exists
 */
function parseLastPage(linkHeader: string,): number | typeof NO_LAST_PAGE {
  /**
   Tagged logger naming GitHub Link header pagination parsing.
   */
  const rl = tagged({
    tag: parseLastPage.name,
    l: logger,
  },);
  /**
   Comma-separated `<url>; rel="x"` segment naming the last page.
   */
  const lastSegment = linkHeader.split(',',)
    .find(function isLast(segment,) {
      return segment.includes('rel="last"',);
    },);
  if (lastSegment === undefined)
    return NO_LAST_PAGE;
  /**
   URL inside the angle brackets of the segment.
   */
  const urlText = lastSegment.split('<',)
    .at(1,)
    ?.split('>',)
    .at(0,);
  if (urlText === undefined)
    return NO_LAST_PAGE;
  try {
    /**
     Textual `page` query parameter from the pagination URL.
     */
    const pageText = new URL(urlText,).searchParams
      .get('page',);
    if ((pageText === null) || (pageText === ''))
      return NO_LAST_PAGE;
    /**
     Parsed last-page number from the `page` query parameter.
     */
    const page = Math.trunc(Number(pageText,),);
    return Number.isFinite(page,) ? page : NO_LAST_PAGE;
  }
  catch (error: unknown) {
    rl.debug(`github Link header page URL parse failed: ${String(error,)}`,);
    return NO_LAST_PAGE;
  }
}

/**
 Default-branch commit count via the GitHub commits endpoint with
 `per_page=1`, reading the `Link` `rel=last` page number. A missing `rel=last`
 (GitHub omits it in some cases) yields a lower bound rather than a failure,
 widening the range. GitHub only; other hosts return undefined.
 
 @param source - parsed remote with host/owner/repo
 
 @returns commit count with a lower-bound flag, or undefined when unavailable
 
 @example
 ```ts
 const commits = await hostCommitCount({ source });
 ```
 */
export async function hostCommitCount(
  { source, }: { readonly source: RemoteSource; },
): Promise<HostCommitCountResult | typeof NO_HOST_COMMITS> {
  /**
   Tagged logger naming the host commit-count signal.
   */
  const rl = tagged({
    tag: hostCommitCount.name,
    l: logger,
  },);
  if ((source.host !== 'github') || (source.owner === undefined)
    || (source.repo === undefined))
    return NO_HOST_COMMITS;

  /**
   Captured `gh api --include` response, headers plus body.
   */
  const result = await spawnResult({
    command: 'gh',
    args: [
      'api',
      '--include',
      `repos/${source.owner}/${source.repo}/commits?per_page=1`,
    ],
  },);
  if (result.exitCode !== 0)
    return NO_HOST_COMMITS;
  /**
   Link header line from the included response headers, if present.
   */
  const linkLine = result.stdout
    .split('\n',)
    .find(function isLink(line,) {
      return line.toLowerCase()
        .startsWith('link:',);
    },);
  if (linkLine === undefined) {
    rl.debug('no Link header; commit count is a lower bound',);
    return {
      count: 1,
      lowerBound: true,
    };
  }
  /**
   Last-page number, equal to the total commit count at per_page=1.
   */
  const lastPage = parseLastPage(linkLine.slice(linkLine.indexOf(':',) + 1,),);
  if (lastPage === NO_LAST_PAGE)
    return {
      count: 1,
      lowerBound: true,
    };
  rl.debug(`github commit count: ${String(lastPage,)}`,);
  return {
    count: lastPage,
    lowerBound: false,
  };
}
