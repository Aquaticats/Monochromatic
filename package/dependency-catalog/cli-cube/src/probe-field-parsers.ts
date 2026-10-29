/**
 Pure parser helpers for npm package metadata.
 
 @example
 ```ts
 const repo = parseRepository('github:preactjs/preact');
 const licenseClass = classifyLicense('MIT');
 ```
 */

import type {
  LicenseClass,
  NpmPackage,
  NpmVersion,
  RepositoryInfo,
} from './probe-field-types.ts';

//region Constants

/**
 Permissive license SPDX identifiers (uppercase).
 */
const PERMISSIVE_LICENSES = new Set([
  'MIT',
  'BSD-2-CLAUSE',
  'BSD-3-CLAUSE',
  'APACHE-2.0',
  'ISC',
  '0BSD',
  'CC0-1.0',
  'UNLICENSE',
],);

//endregion Constants

//region Repository normalisation

/**
 Parsed `{owner, repo}` pair returned by the GitHub source parsers.
 */
type GithubOwnerRepo = {
  /**
   Captured owner segment.
   */
  owner: string;
  /**
   Captured repo segment with the optional `.git` suffix stripped.
   */
  repo: string;
};

/**
 Lowercase shorthand prefix that selects the `github:owner/repo` parser.
 */
const GITHUB_SHORTHAND_PREFIX = 'github:';

/**
 `.git` suffix stripped from repository URL paths so the canonical URL is clean.
 */
const GIT_SUFFIX = '.git';

/**
 Lowercase substring that selects the GitHub URL parser branch.
 */
const GITHUB_DOMAIN = 'github.com';

/**
 Absence marker for {@link parseGithubShorthand} meaning "string is not in
 `github:owner/repo` shorthand form"; never a captured owner/repo pair.
 */
const NOT_SHORTHAND: unique symbol = Symbol('deps-cube/not-github-shorthand',);

/**
 Absence marker for {@link parseGithubUrl} meaning "string is not a GitHub
 URL"; never a captured owner/repo pair.
 */
const NOT_GITHUB_URL: unique symbol = Symbol('deps-cube/not-github-url',);

/**
 Absence marker for {@link parseRepository} meaning "repository field carries
 no parseable repo URL"; never a parsed {@link RepositoryInfo}.
 
 @example
 ```ts
 const info = parseRepository(versionManifest.repository,);
 if (info === REPO_UNPARSEABLE)
   return;
 ```
 */
export const REPO_UNPARSEABLE: unique symbol = Symbol('deps-cube repository field cannot be parsed',);

/**
 Parses the `github:owner/repo` shorthand string. Returns
 {@link NOT_SHORTHAND} for any other shape. Case-insensitive on the prefix
 to match the prior `/^github:.../i` flag.
 
 Linear: one prefix check, one `indexOf`, and a trailing `.git` strip.
 
 @param s - candidate string from the `repository` field
 
 @returns owner/repo pair, or {@link NOT_SHORTHAND} when the shape does not match
 */
function parseGithubShorthand(s: string,): GithubOwnerRepo | typeof NOT_SHORTHAND {
  if (s.length
    <= GITHUB_SHORTHAND_PREFIX
      .length)
    return NOT_SHORTHAND;
  if (s
    .slice(
    0,
    GITHUB_SHORTHAND_PREFIX.length,
    )
    .toLowerCase()
    !== GITHUB_SHORTHAND_PREFIX)
    {
    return NOT_SHORTHAND;
  }
  /**
   Substring after the `github:` prefix; split into owner/repo on the first `/`.
   */
  const rest = s.slice(GITHUB_SHORTHAND_PREFIX.length,);
  /**
   Position of the first `/`; `-1` or `0` means there is no owner half.
   */
  const slashIdx = rest.indexOf('/',);
  if (slashIdx <= 0)
    return NOT_SHORTHAND;
  /**
   Owner segment captured up to the first slash.
   */
  const owner = rest.slice(
    0,
    slashIdx,
  );
  /**
   Repo segment after the slash, with the trailing `.git` (if any) stripped.
   */
  const repoRaw = rest.slice(slashIdx + 1,);
  if (repoRaw === '')
    return NOT_SHORTHAND;
  /**
   Repo with `.git` stripped when present so the URL form matches the prior regex output.
   */
  const repo = repoRaw.endsWith(GIT_SUFFIX,)
    ? repoRaw.slice(
      0,
      -GIT_SUFFIX.length,
    )
    : repoRaw;
  if (repo === '')
    return NOT_SHORTHAND;
  return {
    owner,
    repo,
  };
}

/**
 Parses a GitHub URL of the forms `https://github.com/...`,
 `git+https://github.com/...`, `git@github.com:...`, etc.
 Case-insensitive on the domain to match the prior `/github.com.../i` flag.
 
 Mirrors `/github\.com[/:]([^/]+)\/([^/]+?)(?:\.git)?(?:[/?#].*)?$/` with a
 single linear pass: locate `github.com` via `indexOf`, require the next
 char to be `/` or `:`, split owner/repo on the next `/`, then truncate
 the repo at the first of `[/?#]`. Trailing `.git` is stripped.
 
 @param s - candidate URL from the `repository` field
 
 @returns owner/repo pair, or {@link NOT_GITHUB_URL} when the URL is not on GitHub
 */
function parseGithubUrl(s: string,): GithubOwnerRepo | typeof NOT_GITHUB_URL {
  /**
   Lower-cased copy so the domain scan is case-insensitive; offsets line up with `s`.
   */
  const lowered = s.toLowerCase();
  /**
   Position of the domain in the lower-cased copy.
   */
  const ghIdx = lowered.indexOf(GITHUB_DOMAIN,);
  if (ghIdx === (-1))
    return NOT_GITHUB_URL;
  /**
   Index immediately after the domain; the separator byte must live here.
   */
  const afterDomain = ghIdx + GITHUB_DOMAIN
    .length;
  if (afterDomain >= s
    .length)
    return NOT_GITHUB_URL;
  /**
   Separator char between domain and path; per the URL forms must be `/` or `:`.
   */
  const sep = s.charAt(afterDomain,);
  if ((sep !== '/') && (sep !== ':'))
    return NOT_GITHUB_URL;
  /**
   Path segment between the separator and the rest of the URL.
   */
  const tail = s.slice(afterDomain + 1,);
  /**
   Position of the slash splitting owner from repo.
   */
  const slashIdx = tail.indexOf('/',);
  if (slashIdx <= 0)
    return NOT_GITHUB_URL;
  /**
   Owner segment captured up to the splitting slash.
   */
  const owner = tail.slice(
    0,
    slashIdx,
  );
  /**
   Walks the repo span until the first `/`, `?`, or `#` delimiter.
   
   Single linear pass: the cursor advances one char at a time and never
   revisits a byte, so it stays O(n) time and O(1) stack. The prior
   recursive `return scanRepoEnd(idx + 1)` grew stack depth with the repo
   length and overflowed on V8, which has no tail-call elimination.
   
   @param from - cursor into `tail` where the repo span begins
   
   @returns exclusive end of the repo span
   */
  function scanRepoEnd(from: number,): number {
    /**
     Scan cursor; walked forward to the first URL delimiter or end of `tail`.
     */
    let end = from;
    while (end < tail
      .length) {
      /**
       Char at cursor; URL delimiters end the repo span.
       */
      const c = tail.charAt(end,);
      if ((c === '/')
        || (c === '?')
        || (c === '#'))
        break;
      end += 1;
    }
    return end;
  }
  /**
   Repo span before any URL delimiter.
   */
  const repoRaw = tail.slice(
    slashIdx + 1,
    scanRepoEnd(slashIdx + 1,),
  );
  if (repoRaw === '')
    return NOT_GITHUB_URL;
  /**
   Repo with the trailing `.git` (if any) stripped.
   */
  const repo = repoRaw.endsWith(GIT_SUFFIX,)
    ? repoRaw.slice(
      0,
      -GIT_SUFFIX.length,
    )
    : repoRaw;
  if (repo === '')
    return NOT_GITHUB_URL;
  return {
    owner,
    repo,
  };
}

/**
 Extracts GitHub `{owner, repo}` from heterogeneous npm `repository` fields.
 
 Handles plain strings, `{url, directory}` objects, `github:owner/repo`
 shortcuts, `git+https://...`, `git@github.com:owner/repo.git`. Returns
 {@link REPO_UNPARSEABLE} when no parseable repo URL is present; non-GitHub
 hosts return `{host: 'other', ...}` so the caller can mark TS/SLOC unknown.
 
 @param repoField - Raw `repository` value from version manifest.
 
 @returns Parsed repository info, or {@link REPO_UNPARSEABLE} when unparseable.
 
 @example
 ```ts
 parseRepository({ type: 'git', url: 'git+https://github.com/preactjs/preact.git' });
 // { host: 'github', owner: 'preactjs', repo: 'preact', url: 'https://github.com/preactjs/preact' }
 parseRepository('github:lezer-parser/common');
 // { host: 'github', owner: 'lezer-parser', repo: 'common', url: 'https://github.com/lezer-parser/common' }
 ```
 */
export function parseRepository(repoField: NpmVersion['repository'],): RepositoryInfo | typeof REPO_UNPARSEABLE {
  if ((repoField === undefined) || (repoField === null))
    return REPO_UNPARSEABLE;

  /**
   `true` when `repository` field is a plain string instead of an object.
   */
  const isStringForm = (typeof repoField) === 'string';
  /**
   Unified string form of `repository`, regardless of plain-string or object shape.
   */
  const rawString = isStringForm ? repoField : (repoField.url
    ?? '');
  /**
   Optional monorepo sub-directory; only object-form entries carry one.
   */
  const directory = isStringForm ? undefined : repoField.directory;
  /**
   Spread fragment carrying `directory` only when present, so it never lands as explicit `undefined`.
   */
  const directoryPart = directory === undefined ? {} : { directory, };

  if (rawString === '')
    return REPO_UNPARSEABLE;

  /**
   Parsed `github:owner/repo` shorthand; {@link NOT_SHORTHAND} when the prefix does not match.
   */
  const shortParts = parseGithubShorthand(rawString,);
  if (shortParts !== NOT_SHORTHAND) {
    return {
      host: 'github',
      owner: shortParts.owner,
      repo: shortParts.repo,
      ...directoryPart,
      url: `https://github.com/${shortParts.owner}/${shortParts.repo}`,
    };
  }

  /**
   Parsed `github.com` URL; {@link NOT_GITHUB_URL} when the URL is on a different host.
   */
  const urlParts = parseGithubUrl(rawString,);
  if (urlParts !== NOT_GITHUB_URL) {
    return {
      host: 'github',
      owner: urlParts.owner,
      repo: urlParts.repo,
      ...directoryPart,
      url: `https://github.com/${urlParts.owner}/${urlParts.repo}`,
    };
  }

  return {
    host: 'other',
    owner: '',
    repo: '',
    ...directoryPart,
    url: rawString,
  };
}

//endregion Repository normalisation

//region Version resolution

/**
 Absence marker for {@link resolveVersion} meaning "neither a pinned version
 nor a `dist-tags.latest` resolves for this range"; never a version string.
 
 @example
 ```ts
 const resolved = resolveVersion({ range, pkg, },);
 const version = resolved === VERSION_UNRESOLVED ? entry.range : resolved;
 ```
 */
export const VERSION_UNRESOLVED: unique symbol = Symbol('deps-cube version range cannot resolve',);

/**
 Picks a concrete version that satisfies the catalog range.
 
 Heuristic: pinned versions (no leading operator) are used as-is; otherwise
 `dist-tags.latest` is used. Acceptable for an audit overview; documented
 limitation in `doc/decision/deps-cube.md`.
 
 @param range - Range string from catalog.
 
 @param pkg - Package-level npm registry response.
 
 @returns Concrete version string, or {@link VERSION_UNRESOLVED} if neither pin nor latest resolves.
 
 @example
 ```ts
 resolveVersion({ range: '10.26.0', pkg }); // '10.26.0' when present in pkg.versions
 resolveVersion({ range: '^10.0.0', pkg }); // falls back to pkg['dist-tags'].latest
 ```
 */
export function resolveVersion(
  {
    range,
    pkg,
  }: {
    readonly range: string;
    readonly pkg: NpmPackage;
  },
): string | typeof VERSION_UNRESOLVED {
  if (looksLikePinnedSemver(range,)
    && (pkg.versions?.[range]
      !== undefined))
    return range;
  return pkg['dist-tags']
    ?.latest
    ?? VERSION_UNRESOLVED;
}

/**
 Returns true when `s` begins with three dot-separated digit runs, matching
 the prior `/^\d+\.\d+\.\d+/` test used to decide whether a catalog range
 is a literal pinned version.
 
 Linear: three nested digit walks separated by `.` literals; the cursor
 never revisits any byte.
 
 @param s - candidate range
 
 @returns whether `s` opens with a pinned `major.minor.patch` shape
 */
function looksLikePinnedSemver(s: string,): boolean {
  /**
   Walks the run of ASCII digits starting at `from`.
   
   Single linear pass: the cursor advances one char at a time and never
   revisits a byte, so it stays O(n) time and O(1) stack. The prior
   recursive `return scanDigits(idx + 1)` grew stack depth with the digit
   run length and overflowed on V8, which has no tail-call elimination.
   
   @param from - cursor into `s` where the digit run begins
   
   @returns exclusive end of the digit run
   */
  function scanDigits(from: number,): number {
    /**
     Scan cursor; walked forward over each ASCII digit from `from`.
     */
    let end = from;
    while (end < s
      .length) {
      /**
       Char at cursor; only ASCII digits advance.
       */
      const c = s.charAt(end,);
      if ((c < '0') || (c > '9'))
        break;
      end += 1;
    }
    return end;
  }
  /**
   Exclusive end of the major digit run.
   */
  const major = scanDigits(0,);
  if (major === 0)
    return false;
  if (s.charAt(major,)
    !== '.')
    return false;
  /**
   Exclusive end of the minor digit run.
   */
  const minor = scanDigits(major + 1,);
  if (minor === (major + 1))
    return false;
  if (s.charAt(minor,)
    !== '.')
    return false;
  /**
   Exclusive end of the patch digit run.
   */
  const patch = scanDigits(minor + 1,);
  return patch > (minor + 1);
}

//endregion Version resolution

//region License classification

/**
 Reduces SPDX license strings to coarse audit categories.
 
 @param license - Raw `license` field from version manifest.
 
 @returns One of `permissive`, `copyleft`, `non-oss`, or `unknown`.
 
 @example
 ```ts
 classifyLicense('MIT'); // 'permissive'
 classifyLicense('GPL-3.0'); // 'copyleft'
 classifyLicense({ type: 'UNLICENSED' }); // 'non-oss'
 classifyLicense(undefined); // 'unknown'
 ```
 */
export function classifyLicense(license: NpmVersion['license'],): LicenseClass {
  /**
   `true` when `license` field is a plain SPDX string instead of object form.
   */
  const isStringForm = (typeof license) === 'string';
  /**
   Raw license string before normalisation, either field itself or `.type` subfield.
   */
  const unnormalised = isStringForm ? license : (license?.type
    ?? '');
  /**
   Normalised license string, object form unwrapped, uppercased, trimmed, ready for checks.
   */
  const raw = unnormalised
    .toUpperCase()
    .trim();
  if (raw === '')
    return 'unknown';
  if (PERMISSIVE_LICENSES.has(raw,))
    return 'permissive';
  if (raw.startsWith('GPL',)
    || raw
      .startsWith('LGPL',)
    || raw
      .startsWith('AGPL',)
    || raw
      .includes('COPYLEFT',))
    {
    return 'copyleft';
  }
  if ((raw === 'UNLICENSED')
    || raw
      .startsWith('SEE LICENSE',)
    || raw
      .startsWith('SEE LGPL-3.0-OR-LATER.TXT',)
    || raw
      .startsWith('PROPRIETARY',))
    {
    return 'non-oss';
  }
  return 'unknown';
}

//endregion License classification
