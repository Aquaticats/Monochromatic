/**
 GitHub URL planning and gh execution types.
 
 @module
 */

/**
 GitHub request kind recognized by the URL planner.
 */
type GitHubFetchRequestKind =
  | 'api-endpoint'
    | 'commit-diff'
    | 'comparison-diff'
    | 'directory-listing'
    | 'file-content'
    | 'gist'
    | 'issue-thread'
    | 'pull-request-diff'
    | 'pull-request-thread'
    | 'release-index'
    | 'release-notes'
    | 'repository-home';

/**
 One gh invocation belonging to a single fetch attempt.
 */
type GhInvocation = {
  /**
   Argument vector passed to gh, excluding the executable name.
   */
  readonly args: readonly string[];
  /**
   Whether attempt success requires this invocation to exit zero.
   */
  readonly required: boolean;
};

/**
 One ordered fetch attempt whose invocations run concurrently.
 */
type GhFetchAttempt = {
  /**
   Attempt name used in diagnostics and fallback reasons.
   */
  readonly label: string;
  /**
   Invocations run concurrently, whose non-empty outputs join in order.
   */
  readonly invocations: readonly GhInvocation[];
};

/**
 Planner result for a URL that gh can serve.
 */
type PlannedGitHubFetch = {
  /**
   Whether gh can serve this URL.
   */
  readonly planned: true;
  /**
   Recognized GitHub request kind.
   */
  readonly kind: GitHubFetchRequestKind;
  /**
   Attempts in preference order; the first fully successful attempt answers.
   */
  readonly attempts: readonly GhFetchAttempt[];
};

/**
 Planner result for a URL that gh cannot serve.
 */
type UnplannedGitHubFetch = {
  /**
   Whether gh can serve this URL.
   */
  readonly planned: false;
  /**
   Safe explanation of why no gh attempt was planned.
   */
  readonly reason: string;
};

/**
 Planner result for one fetched URL.
 */
type GitHubFetchPlan = PlannedGitHubFetch | UnplannedGitHubFetch;

/**
 Repository path split candidate used to resolve blob and tree URL ambiguity.
 */
type GitHubRefPathSplit = {
  /**
   Git reference formed from leading path segments.
   */
  readonly ref: string;
  /**
   Repository-relative path formed from remaining segments, empty for a repository root.
   */
  readonly path: string;
};

/**
 gh child process result after a completed run.
 */
type GhCommandRan = {
  /**
   Whether gh started and exited.
   */
  readonly ran: true;
  /**
   Numeric gh exit code.
   */
  readonly exitCode: number;
  /**
   Standard output decoded as UTF-8, lossy when output was not valid UTF-8.
   */
  readonly stdout: string;
  /**
   Whether standard output is safe to render as text.
   
   False when decoding lost bytes or the payload carries a NUL character.
   */
  readonly stdoutIsText: boolean;
  /**
   Standard output byte count before decoding.
   */
  readonly stdoutByteLength: number;
  /**
   Standard error decoded as UTF-8.
   */
  readonly stderr: string;
};

/**
 gh child process result when no usable run happened.
 */
type GhCommandNotRan = {
  /**
   Whether gh started and exited.
   */
  readonly ran: false;
  /**
   Safe explanation of why no run result exists.
   */
  readonly reason: string;
};

/**
 gh child process outcome.
 */
type GhCommandOutcome = GhCommandRan | GhCommandNotRan;

/**
 One gh command execution request.
 */
type GhCommandRequest = {
  /**
   Argument vector passed to gh, excluding the executable name.
   */
  readonly args: readonly string[];
  /**
   Cancellation signal owned by the calling tool execution.
   */
  readonly signal?: AbortSignal;
};

/**
 Injectable gh command boundary.
 */
type GhCommandRunner = (request: GhCommandRequest,) => Promise<GhCommandOutcome>;

/**
 gh command runner construction options.
 */
type GhCommandRunnerOptions = {
  /**
   Optional executable override, primarily for tests.
   */
  readonly executable?: string;
  /**
   Optional child deadline override in milliseconds.
   */
  readonly deadlineMs?: number;
  /**
   Optional captured output ceiling override in bytes.
   */
  readonly maxOutputBytes?: number;
  /**
   Optional working directory override keeping gh away from ambient repository context.
   */
  readonly cwd?: string;
};

/**
 gh fetch client constructor options.
 */
type GhClientOptions = {
  /**
   Optional gh command boundary override for tests.
   */
  readonly runner?: GhCommandRunner;
};

/**
 gh fetch request accepted by the client.
 */
type GhClientFetchOptions = {
  /**
   Fetched URL, used only in diagnostics and binary-content notices.
   */
  readonly url: string;
  /**
   Recognized GitHub request kind.
   */
  readonly kind: GitHubFetchRequestKind;
  /**
   Planned attempts in preference order.
   */
  readonly attempts: readonly GhFetchAttempt[];
  /**
   Cancellation signal owned by the calling tool execution.
   */
  readonly signal?: AbortSignal;
};

/**
 gh-backed fetch client surface.
 */
type GhClient = {
  /**
   Execute planned gh attempts and shape one markdown-only response.
   */
  readonly fetch: (options: GhClientFetchOptions,) => Promise<unknown>;
};

/**
 Parsed GitHub URL ready for host dispatch.
 */
type ParsedGitHubUrl = {
  /**
   Whether the URL parsed with an accepted scheme and mapped host.
   */
  readonly parsed: true;
  /**
   Parsed URL.
   */
  readonly url: URL;
  /**
   Lowercase host without one www label or one trailing root dot.
   */
  readonly host: string;
  /**
   Path segments without leading or trailing empty entries.
   */
  readonly segments: readonly string[];
} | {
  /**
   Whether the URL parsed with an accepted scheme and mapped host.
   */
  readonly parsed: false;
  /**
   Safe rejection reason.
   */
  readonly reason: string;
};

/**
 Positional argument validation result.
 */
type TokenValidation = {
  /**
   Whether the value is safe as one gh argument.
   */
  readonly safe: true;
} | {
  /**
   Whether the value is safe as one gh argument.
   */
  readonly safe: false;
  /**
   Safe rejection reason.
   */
  readonly reason: string;
};

/**
 Path segment validation result.
 */
type SegmentValidation = {
  /**
   Whether every segment is safe to forward to gh.
   */
  readonly valid: true;
} | {
  /**
   Whether every segment is safe to forward to gh.
   */
  readonly valid: false;
  /**
   Safe rejection reason naming the first rejected segment.
   */
  readonly reason: string;
};

/**
 Result of one URL parse attempt.
 */
type UrlParseResult = {
  /**
   Whether the URL parsed.
   */
  readonly parsed: true;
  /**
   Parsed URL.
   */
  readonly url: URL;
} | {
  /**
   Whether the URL parsed.
   */
  readonly parsed: false;
  /**
   Safe rejection reason.
   */
  readonly reason: string;
};

/**
 Repository owner and name resolved from path segments.
 */
type RepositoryTarget = {
  /**
   Whether owner and name resolved.
   */
  readonly found: true;
  /**
   Repository owner login.
   */
  readonly owner: string;
  /**
   Repository name.
   */
  readonly repo: string;
  /**
   Path segments following the repository name.
   */
  readonly rest: readonly string[];
} | {
  /**
   Whether owner and name resolved.
   */
  readonly found: false;
  /**
   Safe rejection reason.
   */
  readonly reason: string;
};

/**
 Repository section planning context.
 */
type RepositorySectionContext = {
  /**
   Repository owner login.
   */
  readonly owner: string;
  /**
   Repository name.
   */
  readonly repo: string;
  /**
   Path segments after the section name.
   */
  readonly rest: readonly string[];
};

/**
 Repository contents planning context.
 */
type RepositoryContentContext = {
  /**
   Repository owner login.
   */
  readonly owner: string;
  /**
   Repository name.
   */
  readonly repo: string;
  /**
   Ordered reference and path split candidates.
   */
  readonly splits: readonly GitHubRefPathSplit[];
  /**
   Whether the response is a directory listing rather than file bytes.
   */
  readonly listing: boolean;
};

/**
 Markdown-only response shape shared with Linkup fetch responses.
 */
type GhMarkdownResponse = {
  /**
   Model-visible response text.
   */
  readonly markdown: string;
};

export type {
  GhClient,
  GhClientFetchOptions,
  GhClientOptions,
  GhCommandNotRan,
  GhCommandOutcome,
  GhCommandRan,
  GhCommandRequest,
  GhCommandRunner,
  GhCommandRunnerOptions,
  GhFetchAttempt,
  GhInvocation,
  GhMarkdownResponse,
  GitHubFetchRequestKind,
  GitHubFetchPlan,
  GitHubRefPathSplit,
  ParsedGitHubUrl,
  PlannedGitHubFetch,
  RepositoryContentContext,
  RepositorySectionContext,
  RepositoryTarget,
  SegmentValidation,
  TokenValidation,
  UnplannedGitHubFetch,
  UrlParseResult,
};
