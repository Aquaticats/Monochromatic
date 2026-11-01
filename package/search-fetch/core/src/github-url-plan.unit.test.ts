/**
 Unit tests for GitHub URL to gh invocation planning.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  planGitHubFetch,
  type GhInvocation,
  type GitHubFetchRequestKind,
  type PlannedGitHubFetch,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Raw media type header used for repository file reads.
 */
const RAW_ACCEPT_HEADER = 'Accept: application/vnd.github.raw';

/**
 Diff media type header used for commit and comparison reads.
 */
const DIFF_ACCEPT_HEADER = 'Accept: application/vnd.github.diff';

/**
 jq projection expected for directory listings.
 */
const DIRECTORY_LISTING_JQ = String.raw`if type == "array" then .[] else . end | "\(.type)\t\(.name)\t\(.size)"`;

/**
 One expected gh invocation.
 */
type ExpectedInvocation = {
  /**
   Expected argument vector.
   */
  readonly args: readonly string[];
  /**
   Whether attempt success requires this invocation.
   */
  readonly required: boolean;
};

/**
 One mapped URL case with its complete expected plan.
 */
type MappedCase = {
  /**
   Case name.
   */
  readonly name: string;
  /**
   Fetched URL.
   */
  readonly url: string;
  /**
   Expected request kind.
   */
  readonly kind: GitHubFetchRequestKind;
  /**
   Expected attempts in order.
   */
  readonly attempts: readonly (readonly ExpectedInvocation[])[];
};

/**
 One unmapped URL case with a reason fragment.
 */
type UnmappedCase = {
  /**
   Case name.
   */
  readonly name: string;
  /**
   Fetched URL.
   */
  readonly url: string;
  /**
   Substring the rejection reason must carry.
   */
  readonly reasonPart: string;
};

/**
 Required invocation fixture builder.
 */
function required(args: readonly string[],): ExpectedInvocation {
  return {
    args,
    required: true,
  };
}

/**
 Optional invocation fixture builder.
 */
function optional(args: readonly string[],): ExpectedInvocation {
  return {
    args,
    required: false,
  };
}

/**
 Repository contents endpoint fixture builder.
 */
function contentsEndpoint(
  {
    refQuery,
    path,
  }: {
    readonly refQuery: string;
    readonly path: string;
  },
): string {
  return `/repos/cli/cli/contents/${path}?ref=${refQuery}`;
}

/**
 Raw file invocation fixture builder.
 */
function rawFileInvocation(
  {
    refQuery,
    path,
  }: {
    readonly refQuery: string;
    readonly path: string;
  },
): ExpectedInvocation {
  return required([
    'api',
    '-H',
    RAW_ACCEPT_HEADER,
    contentsEndpoint({
      refQuery,
      path,
    },),
  ],);
}

/**
 Directory listing invocation fixture builder.
 */
function listingInvocation(
  {
    refQuery,
    path,
  }: {
    readonly refQuery: string;
    readonly path: string;
  },
): ExpectedInvocation {
  return required([
    'api',
    contentsEndpoint({
      refQuery,
      path,
    },),
    '--jq',
    DIRECTORY_LISTING_JQ,
  ],);
}

/**
 Issue thread invocation pair fixture.
 */
const ISSUE_THREAD_INVOCATIONS: readonly ExpectedInvocation[] = [
  required([
    'issue',
    'view',
    'https://github.com/cli/cli/issues/13118',
  ],),
  optional([
    'issue',
    'view',
    'https://github.com/cli/cli/issues/13118',
    '--comments',
  ],),
];

/**
 Pull request thread invocation pair fixture.
 */
const PULL_THREAD_INVOCATIONS: readonly ExpectedInvocation[] = [
  required([
    'pr',
    'view',
    'https://github.com/cli/cli/pull/14517',
  ],),
  optional([
    'pr',
    'view',
    'https://github.com/cli/cli/pull/14517',
    '--comments',
  ],),
];

/**
 Every mapped URL shape with its complete expected gh plan.
 */
const MAPPED_CASES: readonly MappedCase[] = [
  {
    name: 'repository home page reads repository overview and README',
    url: 'https://github.com/cli/cli',
    kind: 'repository-home',
    attempts: [
      [
        required([
          'repo',
          'view',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'repository home page normalizes uppercase host, www label, root dot, and trailing slash',
    url: 'https://WWW.GitHub.COM./cli/cli/',
    kind: 'repository-home',
    attempts: [
      [
        required([
          'repo',
          'view',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'repository home page ignores readme tab query and fragment',
    url: 'https://github.com/cli/cli?tab=readme-ov-file#readme',
    kind: 'repository-home',
    attempts: [
      [
        required([
          'repo',
          'view',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'plain http GitHub URL still plans a gh read',
    url: 'http://github.com/cli/cli',
    kind: 'repository-home',
    attempts: [
      [
        required([
          'repo',
          'view',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'blob file URL reads raw file bytes at the first split',
    url: 'https://github.com/cli/cli/blob/trunk/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'raw file URL reads the same raw file bytes',
    url: 'https://github.com/cli/cli/raw/trunk/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'blame file URL reads the same raw file bytes',
    url: 'https://github.com/cli/cli/blame/trunk/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'nested blob file URL keeps one attempt per split point',
    url: 'https://github.com/cli/cli/blob/trunk/internal/gh/gh.go',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'internal/gh/gh.go',
        },),
      ],
      [
        rawFileInvocation({
          refQuery: 'trunk%2Finternal',
          path: 'gh/gh.go',
        },),
      ],
      [
        rawFileInvocation({
          refQuery: 'trunk%2Finternal%2Fgh',
          path: 'gh.go',
        },),
      ],
    ],
  },
  {
    name: 'multi-segment reference URL orders the longer reference split last',
    url: 'https://github.com/cli/cli/blob/8761/allow-items/go.mod',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: '8761',
          path: 'allow-items/go.mod',
        },),
      ],
      [
        rawFileInvocation({
          refQuery: '8761%2Fallow-items',
          path: 'go.mod',
        },),
      ],
    ],
  },
  {
    name: 'blob file URL keeps percent-encoded path characters and drops plain query and line fragment',
    url: 'https://github.com/cli/cli/blob/trunk/docs/my%20file.md?plain=1#L1-L9',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'docs/my%20file.md',
        },),
      ],
      [
        rawFileInvocation({
          refQuery: 'trunk%2Fdocs',
          path: 'my%20file.md',
        },),
      ],
    ],
  },
  {
    name: 'blob file URL keeps percent-encoded non-ASCII path characters',
    url: 'https://github.com/cli/cli/blob/trunk/docs/%E4%B8%AD%E6%96%87.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'docs/%E4%B8%AD%E6%96%87.md',
        },),
      ],
      [
        rawFileInvocation({
          refQuery: 'trunk%2Fdocs',
          path: '%E4%B8%AD%E6%96%87.md',
        },),
      ],
    ],
  },
  {
    name: 'raw.githubusercontent.com file URL reads the same raw file bytes',
    url: 'https://raw.githubusercontent.com/cli/cli/trunk/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'trunk',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'tree directory URL projects one listing line per entry',
    url: 'https://github.com/cli/cli/tree/trunk/docs',
    kind: 'directory-listing',
    attempts: [
      [
        listingInvocation({
          refQuery: 'trunk',
          path: 'docs',
        },),
      ],
      [
        listingInvocation({
          refQuery: 'trunk%2Fdocs',
          path: '',
        },),
      ],
    ],
  },
  {
    name: 'tree reference-only URL lists the repository root',
    url: 'https://github.com/cli/cli/tree/trunk',
    kind: 'directory-listing',
    attempts: [
      [
        listingInvocation({
          refQuery: 'trunk',
          path: '',
        },),
      ],
    ],
  },
  {
    name: 'numbered issue URL reads the body and tolerates a failed comment read',
    url: 'https://github.com/cli/cli/issues/13118',
    kind: 'issue-thread',
    attempts: [
      ISSUE_THREAD_INVOCATIONS,
    ],
  },
  {
    name: 'numbered issue URL rebuilds a canonical URL from query and fragment noise',
    url: 'https://github.com/cli/cli/issues/13118?foo=bar#issuecomment-1',
    kind: 'issue-thread',
    attempts: [
      ISSUE_THREAD_INVOCATIONS,
    ],
  },
  {
    name: 'numbered pull request URL reads the body and tolerates a failed comment read',
    url: 'https://github.com/cli/cli/pull/14517',
    kind: 'pull-request-thread',
    attempts: [
      PULL_THREAD_INVOCATIONS,
    ],
  },
  {
    name: 'pull request changed-file URL reads the diff without color',
    url: 'https://github.com/cli/cli/pull/14517/files',
    kind: 'pull-request-diff',
    attempts: [
      [
        required([
          'pr',
          'diff',
          'https://github.com/cli/cli/pull/14517',
          '--color',
          'never',
        ],),
      ],
    ],
  },
  {
    name: 'commit URL reads one unified diff',
    url: 'https://github.com/cli/cli/commit/9b031151a825bda919203c5202876a725d637368',
    kind: 'commit-diff',
    attempts: [
      [
        required([
          'api',
          '-H',
          DIFF_ACCEPT_HEADER,
          '/repos/cli/cli/commits/9b031151a825bda919203c5202876a725d637368',
        ],),
      ],
    ],
  },
  {
    name: 'comparison URL reads one unified diff',
    url: 'https://github.com/cli/cli/compare/v2.100.0...v2.101.0',
    kind: 'comparison-diff',
    attempts: [
      [
        required([
          'api',
          '-H',
          DIFF_ACCEPT_HEADER,
          '/repos/cli/cli/compare/v2.100.0...v2.101.0',
        ],),
      ],
    ],
  },
  {
    name: 'comparison URL rejoins a slashed head reference',
    url: 'https://github.com/cli/cli/compare/trunk...8761/allow-items',
    kind: 'comparison-diff',
    attempts: [
      [
        required([
          'api',
          '-H',
          DIFF_ACCEPT_HEADER,
          '/repos/cli/cli/compare/trunk...8761/allow-items',
        ],),
      ],
    ],
  },
  {
    name: 'releases URL lists releases for an explicit repository',
    url: 'https://github.com/cli/cli/releases',
    kind: 'release-index',
    attempts: [
      [
        required([
          'release',
          'list',
          '--repo',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'release tag URL reads release notes for an explicit repository',
    url: 'https://github.com/cli/cli/releases/tag/v2.101.0',
    kind: 'release-notes',
    attempts: [
      [
        required([
          'release',
          'view',
          'v2.101.0',
          '--repo',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'release tag URL rejoins a slashed tag',
    url: 'https://github.com/cli/cli/releases/tag/release/1.0',
    kind: 'release-notes',
    attempts: [
      [
        required([
          'release',
          'view',
          'release/1.0',
          '--repo',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'gist URL with an owner reads the gist by identifier',
    url: 'https://gist.github.com/Aquaticat/a6570831a0e0a6ca2c86919b54b3c4d9',
    kind: 'gist',
    attempts: [
      [
        required([
          'gist',
          'view',
          'a6570831a0e0a6ca2c86919b54b3c4d9',
        ],),
      ],
    ],
  },
  {
    name: 'gist URL without an owner and with a file fragment reads the gist by identifier',
    url: 'https://gist.github.com/a6570831a0e0a6ca2c86919b54b3c4d9#file-npmrc',
    kind: 'gist',
    attempts: [
      [
        required([
          'gist',
          'view',
          'a6570831a0e0a6ca2c86919b54b3c4d9',
        ],),
      ],
    ],
  },
  {
    name: 'blob reference carrying plus and ampersand reaches the query as one encoded value',
    url: 'https://github.com/cli/cli/blob/topic+one&x=1/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: 'topic%2Bone%26x%3D1',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'blob reference carrying an encoded slash round-trips through decode and encode',
    url: 'https://github.com/cli/cli/blob/8761%2Fallow-items/go.mod',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: '8761%2Fallow-items',
          path: 'go.mod',
        },),
      ],
    ],
  },
  {
    name: 'blob reference carrying a malformed escape stays raw and gets encoded once',
    url: 'https://github.com/cli/cli/blob/100%/README.md',
    kind: 'file-content',
    attempts: [
      [
        rawFileInvocation({
          refQuery: '100%25',
          path: 'README.md',
        },),
      ],
    ],
  },
  {
    name: 'release tag URL decodes an encoded build metadata character',
    url: 'https://github.com/cli/cli/releases/tag/v1.0%2Bbuild',
    kind: 'release-notes',
    attempts: [
      [
        required([
          'release',
          'view',
          'v1.0+build',
          '--repo',
          'cli/cli',
        ],),
      ],
    ],
  },
  {
    name: 'GitHub API URL forwards path and query to gh api',
    url: 'https://api.github.com/repos/cli/cli?per_page=2',
    kind: 'api-endpoint',
    attempts: [
      [
        required([
          'api',
          '/repos/cli/cli?per_page=2',
        ],),
      ],
    ],
  },
];

/**
 Every rejected URL shape with the reason fragment proving which guard refused it.
 */
const UNMAPPED_CASES: readonly UnmappedCase[] = [
  {
    name: 'non-GitHub host has no gh mapping',
    url: 'https://example.com/cli/cli',
    reasonPart: 'host example.com has no gh mapping',
  },
  {
    name: 'GitHub CDN host has no gh mapping',
    url: 'https://camo.githubusercontent.com/abc123',
    reasonPart: 'has no gh mapping',
  },
  {
    name: 'GitHub root URL names no repository',
    url: 'https://github.com/',
    reasonPart: 'names no repository',
  },
  {
    name: 'owner URL has no gh equivalent',
    url: 'https://github.com/cli',
    reasonPart: 'has no gh equivalent',
  },
  {
    name: 'wiki section has no gh equivalent',
    url: 'https://github.com/cli/cli/wiki/Home',
    reasonPart: 'section /wiki/ has no gh equivalent',
  },
  {
    name: 'actions section has no gh equivalent',
    url: 'https://github.com/cli/cli/actions/runs/36224089994',
    reasonPart: 'section /actions/ has no gh equivalent',
  },
  {
    name: 'commit list section has no gh equivalent',
    url: 'https://github.com/cli/cli/commits/trunk',
    reasonPart: 'section /commits/ has no gh equivalent',
  },
  {
    name: 'issues URL without a number names no issue',
    url: 'https://github.com/cli/cli/issues',
    reasonPart: 'names no issue number',
  },
  {
    name: 'new issue form URL is not a numbered issue',
    url: 'https://github.com/cli/cli/issues/new',
    reasonPart: 'is not a number',
  },
  {
    name: 'over-long issue number is not a numbered issue',
    url: 'https://github.com/cli/cli/issues/1234567890',
    reasonPart: 'is not a number',
  },
  {
    name: 'pull request commit subsection has no gh equivalent',
    url: 'https://github.com/cli/cli/pull/14517/commits',
    reasonPart: 'subsection /commits/ has no gh equivalent',
  },
  {
    name: 'pull URL without a number names no pull request',
    url: 'https://github.com/cli/cli/pull',
    reasonPart: 'names no pull request number',
  },
  {
    name: 'blob URL without a file path is refused',
    url: 'https://github.com/cli/cli/blob/trunk',
    reasonPart: 'names no file path',
  },
  {
    name: 'blob URL with a trailing slash keeps no file path',
    url: 'https://github.com/cli/cli/blob/trunk/',
    reasonPart: 'names no file path',
  },
  {
    name: 'tree URL without a reference is refused',
    url: 'https://github.com/cli/cli/tree',
    reasonPart: 'names no reference',
  },
  {
    name: 'commit URL without a reference is refused',
    url: 'https://github.com/cli/cli/commit',
    reasonPart: 'commit reference is empty',
  },
  {
    name: 'compare URL without a specification is refused',
    url: 'https://github.com/cli/cli/compare',
    reasonPart: 'comparison specification is empty',
  },
  {
    name: 'release tag URL without a tag is refused',
    url: 'https://github.com/cli/cli/releases/tag',
    reasonPart: 'release tag is empty',
  },
  {
    name: 'release tag starting with a dash would parse as a gh flag',
    url: 'https://github.com/cli/cli/releases/tag/-rc1',
    reasonPart: 'would parse as a gh flag',
  },
  {
    name: 'release tag decoding to a leading dash would parse as a gh flag',
    url: 'https://github.com/cli/cli/releases/tag/%2Drc1',
    reasonPart: 'would parse as a gh flag',
  },
  {
    name: 'new release form URL has no gh equivalent',
    url: 'https://github.com/cli/cli/releases/new',
    reasonPart: 'subsection /new/ has no gh equivalent',
  },
  {
    name: 'release download URL has no gh equivalent',
    url: 'https://github.com/cli/cli/releases/download/v2.101.0/gh_2.101.0_linux_amd64.tar.gz',
    reasonPart: 'subsection /download/ has no gh equivalent',
  },
  {
    name: 'gist URL without an identifier names no gist',
    url: 'https://gist.github.com/',
    reasonPart: 'names no gist identifier',
  },
  {
    name: 'raw GitHub URL without a file path is refused',
    url: 'https://raw.githubusercontent.com/cli/cli/trunk',
    reasonPart: 'names no file path',
  },
  {
    name: 'raw GitHub URL without a repository is refused',
    url: 'https://raw.githubusercontent.com/cli',
    reasonPart: 'names no repository',
  },
  {
    name: 'explicit port is not a mapped GitHub host',
    url: 'https://github.com:8443/cli/cli',
    reasonPart: 'is not a mapped GitHub host',
  },
  {
    name: 'non-HTTP scheme is refused',
    url: 'ftp://github.com/cli/cli',
    reasonPart: 'is not an HTTP GitHub URL',
  },
  {
    name: 'file URL is refused',
    url: 'file:///etc/passwd',
    reasonPart: 'is not an HTTP GitHub URL',
  },
  {
    name: 'unparsable URL is refused',
    url: 'not a url',
    reasonPart: 'unparsable URL',
  },
  {
    name: 'owner login outside GitHub naming rules is refused',
    url: 'https://github.com/ow!ner/repo',
    reasonPart: 'contains characters GitHub does not allow',
  },
  {
    name: 'repository name outside GitHub naming rules is refused',
    url: 'https://github.com/cli/repo%20name',
    reasonPart: 'contains characters GitHub does not allow',
  },
  {
    name: 'owner login starting with a dash would parse as a gh flag',
    url: 'https://github.com/-cli/cli',
    reasonPart: 'would parse as a gh flag',
  },
  {
    name: 'interior empty path segment is refused before any gh argument is built',
    url: 'https://github.com/cli/cli/blob/trunk//README.md',
    reasonPart: 'is not a safe gh argument',
  },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: planGitHubFetch.name,
      children: [
        //region Mapped GitHub shapes

        ...MAPPED_CASES.map(function mapMappedCase(mappedCase: MappedCase,) {
          return it({
            name: mappedCase.name,
            fn: async () => {
              /**
               Local value for plan.
               */
              const plan = planGitHubFetch({ url: mappedCase.url, },);

              expect(plan.planned,).toBe(true,);
              /**
               Local value for planned.
               */
              const planned = plan as PlannedGitHubFetch;
              expect(planned.kind,).toBe(mappedCase.kind,);
              expect(plannedInvocations(planned,),).toEqual(mappedCase.attempts,);
            },
          },);
        },),

        //endregion Mapped GitHub shapes

        //region Rejected GitHub shapes

        ...UNMAPPED_CASES.map(function mapUnmappedCase(unmappedCase: UnmappedCase,) {
          return it({
            name: unmappedCase.name,
            fn: async () => {
              /**
               Local value for plan.
               */
              const plan = planGitHubFetch({ url: unmappedCase.url, },);

              expect(plan.planned,).toBe(false,);
              if (plan.planned)
                return;
              expect(plan.reason,).toContain(unmappedCase.reasonPart,);
            },
          },);
        },),

        //endregion Rejected GitHub shapes

        //region Attempt labels

        it({
          name: 'labels every split attempt with its position, reference, and path',
          fn: async () => {
            /**
             Local value for plan.
             */
            const plan = planGitHubFetch({ url: 'https://github.com/cli/cli/blob/trunk/internal/gh.go', },);

            expect(plan.planned,).toBe(true,);
            /**
             Local value for planned.
             */
            const planned = plan as PlannedGitHubFetch;
            expect(planned.attempts[0]?.label,).toContain('attempt 1 of 2',);
            expect(planned.attempts[0]?.label,).toContain('ref trunk',);
            expect(planned.attempts[0]?.label,).toContain('path internal/gh.go',);
            expect(planned.attempts[1]?.label,).toContain('attempt 2 of 2',);
            expect(planned.attempts[1]?.label,).toContain('ref trunk/internal',);
            expect(planned.attempts[1]?.label,).toContain('path gh.go',);
          },
        },),
        it({
          name: 'labels a repository root listing instead of naming an empty path',
          fn: async () => {
            /**
             Local value for plan.
             */
            const plan = planGitHubFetch({ url: 'https://github.com/cli/cli/tree/trunk', },);

            expect(plan.planned,).toBe(true,);
            /**
             Local value for planned.
             */
            const planned = plan as PlannedGitHubFetch;
            expect(planned.attempts[0]?.label,).toContain('repository root',);
          },
        },),

        //endregion Attempt labels

        //region Query encoding

        it({
          name: 'builds a contents endpoint whose ref query value survives URL parsing',
          fn: async () => {
            /**
             Local value for plan.
             */
            const plan = planGitHubFetch({ url: 'https://github.com/cli/cli/blob/topic+one&x=1/README.md', },);

            expect(plan.planned,).toBe(true,);
            /**
             Local value for planned.
             */
            const planned = plan as PlannedGitHubFetch;
            /**
             Local value for endpoint.
             */
            const endpoint = planned.attempts[0]?.invocations[0]?.args[3] ?? '';
            /**
             Local value for parsed.
             */
            const parsed = new URL(`https://api.github.com${endpoint}`,);

            expect(parsed.searchParams.get('ref'),).toBe('topic+one&x=1',);
            expect(parsed.searchParams.get('x'),).toBe(null,);
            expect(parsed.pathname,).toBe('/repos/cli/cli/contents/README.md',);
          },
        },),

        //endregion Query encoding
      ],
    },),
  ],
},);

//region Helpers

/**
 Reduce one plan to its expected invocation fixture shape.
 
 @param planned - successful plan
 
 @returns attempts as invocation fixture arrays
 */
function plannedInvocations(
  planned: PlannedGitHubFetch,
): readonly (readonly ExpectedInvocation[])[] {
  return planned.attempts
    .map(function mapAttempt(attempt,): readonly ExpectedInvocation[] {
      return attempt.invocations
        .map(function mapInvocation(invocation: GhInvocation,): ExpectedInvocation {
          return {
            args: [
              ...invocation.args,
            ],
            required: invocation.required,
          };
        },);
    },);
}

//endregion Helpers
