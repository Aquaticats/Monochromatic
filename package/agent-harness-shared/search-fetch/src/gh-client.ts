/**
 gh-backed fetch client for Pi Search Fetch.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { runGhCommand, } from './gh-process.ts';
import type {
  GhClient,
  GhClientFetchOptions,
  GhClientOptions,
  GhCommandOutcome,
  GhCommandRequest,
  GhCommandRunner,
  GhFetchAttempt,
  GhInvocation,
  GhMarkdownResponse,
  GitHubFetchRequestKind,
} from './github-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: ghClientLogger, },);
 ```
 */
const ghClientLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'gh-client',
  l: ghClientLogger,
},);

//region Constants

/**
 gh exit code documented for a cancelled command.
 */
const GH_CANCELLED_EXIT_CODE = 2;

/**
 gh exit code documented for a command needing authentication.
 */
const GH_AUTH_REQUIRED_EXIT_CODE = 4;

/**
 Explanations for gh exit codes that carry documented meaning.
 */
const GH_EXIT_CODE_MEANINGS: Record<number, string> = {
  [GH_CANCELLED_EXIT_CODE]: 'gh cancelled the request',
  [GH_AUTH_REQUIRED_EXIT_CODE]: 'gh needs authentication, run gh auth login or set GH_TOKEN',
};

/**
 Characters kept from captured standard error inside one failure reason.
 */
const GH_STDERR_EXCERPT_CHARACTERS = 400;

/**
 Separator joining concurrently fetched thread parts.
 */
const ATTEMPT_OUTPUT_SEPARATOR = '\n\n';

/**
 Suffix marking a truncated standard error excerpt.
 */
const EXCERPT_TRUNCATION_SUFFIX = '...';

/**
 gh exit code covering ordinary command failures, including a wrong reference or path split.
 */
const GH_RETRYABLE_EXIT_CODE = 1;

//endregion Constants

//region Types

/**
 One gh invocation paired with its outcome.
 */
type InvocationResult = {
  /**
   Planned invocation.
   */
  readonly invocation: GhInvocation;
  /**
   Outcome reported by the gh command boundary.
   */
  readonly outcome: GhCommandOutcome;
};

/**
 One invocation rendered as output text or as a failure reason.
 */
type InvocationDescription = {
  /**
   Whether attempt success requires this invocation.
   */
  readonly required: boolean;
  /**
   Whether this invocation produced usable output.
   */
  readonly failed: false;
  /**
   Output text contributed to the attempt result.
   */
  readonly text: string;
} | {
  /**
   Whether attempt success requires this invocation.
   */
  readonly required: boolean;
  /**
   Whether this invocation produced usable output.
   */
  readonly failed: true;
  /**
   Safe failure reason.
   */
  readonly reason: string;
  /**
   Whether another attempt could plausibly succeed after this failure.
   */
  readonly retryable: boolean;
};

/**
 One attempt rendered as joined output text or as a failure reason.
 */
type AttemptResult = {
  /**
   Whether every required invocation exited zero.
   */
  readonly succeeded: true;
  /**
   Joined non-empty invocation output.
   */
  readonly text: string;
} | {
  /**
   Whether every required invocation exited zero.
   */
  readonly succeeded: false;
  /**
   Safe failure reason from the first failed required invocation.
   */
  readonly reason: string;
  /**
   Whether another attempt could plausibly succeed after this failure.
   */
  readonly retryable: boolean;
};

//endregion Types

//region Errors

/**
 Reports every failed gh attempt for one GitHub fetch.
 */
class GhFetchError extends Error {
  /**
   Fetched URL that no gh attempt served.
   */
  public readonly url: string;

  /**
   Recognized GitHub request kind.
   */
  public readonly kind: GitHubFetchRequestKind;

  /**
   Per-attempt failure reasons in attempt order.
   */
  public readonly attemptReasons: readonly string[];

  /**
   Creates one exhausted-attempts failure.
   
   @param url - fetched URL
   
   @param kind - recognized GitHub request kind
   
   @param attemptReasons - per-attempt failure reasons in attempt order
   
   @example
   ```ts
   throw new GhFetchError({ url: 'https://github.com/cli/cli', kind: 'repository-home', attemptReasons: ['repository cli/cli overview and README: gh executable gh was not found on PATH'] });
   ```
   */
  public constructor(
    {
      url,
      kind,
      attemptReasons,
    }: {
      readonly url: string;
      readonly kind: GitHubFetchRequestKind;
      readonly attemptReasons: readonly string[];
    },
  ) {
    super(`gh fetch failed for ${url} (${kind}): ${attemptReasons.join(' | ',)}`,);
    this.name = 'GhFetchError';
    this.url = url;
    this.kind = kind;
    this.attemptReasons = attemptReasons;
  }
}

//endregion Errors

//region Client factory

/**
 Create gh-backed fetch client executing planned attempts in order.
 
 @param clientOptions - client options
 
 @returns frozen gh client
 
 @example
 ```ts
 const client = createGhClient({});
 ```
 */
function createGhClient(clientOptions: ForeignBorrowed<GhClientOptions>,): GhClient {
  /**
   gh command boundary captured by client methods.
   */
  const runner = clientOptions.runner ?? runGhCommand;

  return Object.freeze({
    /**
     Runs planned gh attempts for one caller-owned GitHub fetch.
     
     @param fetchOptions - planned attempts, fetched URL, and optional signal.
     
     @returns markdown-only response holding joined gh output.
     
     @throws {@link GhFetchError} when every planned attempt failed.
     
     @mutates fetchOptions - gh children register abort listeners on `fetchOptions.signal`.
     */
    fetch(fetchOptions: ForeignBorrowed<GhClientFetchOptions>,): Promise<GhMarkdownResponse> {
      return fetchThroughGh({
        runner,
        options: fetchOptions,
      },);
    },
  },);
}

//endregion Client factory

//region Attempt execution

/**
 Execute planned attempts in order and shape the first successful one.
 
 @param runner - gh command boundary
 
 @param options - planned attempts, fetched URL, and optional signal
 
 @returns markdown-only response holding joined gh output
 
 @throws {@link GhFetchError} when every planned attempt failed
 
 @mutates options - gh children register abort listeners on `options.signal`.
 
 @example
 ```ts
 await fetchThroughGh({ runner, options: { url: 'https://github.com/cli/cli', kind: 'repository-home', attempts: [] } });
 ```
 */
async function fetchThroughGh(
  {
    runner,
    options,
  }: {
    readonly runner: GhCommandRunner;
    readonly options: GhClientFetchOptions;
  },
): Promise<GhMarkdownResponse> {
  /**
   Logger tagged for this gh fetch.
   */
  const innerL = tagged({
    tag: fetchThroughGh.name,
    l,
  },);
  /**
   Planned attempts, fetched URL, request kind, and caller-owned signal.
   */
  const {
    url,
    kind,
    attempts,
    signal,
  } = options;
  /**
   Per-attempt failure reasons collected while attempts are exhausted.
   */
  const attemptReasons: string[] = [];
  innerL.debug(`fetching ${url} as ${kind} through ${String(attempts.length,)} gh attempt(s)`,);

  /* oxlint-disable eslint/no-await-in-loop -- Attempts are ordered fallbacks whose first success answers, so running them concurrently would spend gh calls the ordering exists to avoid. */
  for (const attempt of attempts) {
    /**
     Attempt result from one concurrent invocation group.
     */
    const result = await runAttempt({
      runner,
      attempt,
      url,
      ...(signal === undefined ? {} : { signal, }),
    },);
    if (result.succeeded) {
      innerL.debug(`gh attempt succeeded: ${attempt.label}`,);
      return { markdown: result.text, };
    }
    innerL.warn(`gh attempt failed: ${attempt.label}: ${result.reason}`,);
    attemptReasons.push(`${attempt.label}: ${result.reason}`,);
    if (!result.retryable) {
      innerL.warn(`stopping gh attempts because the failure is not reference or path ambiguity: ${result.reason}`,);
      break;
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */

  throw new GhFetchError({
    url,
    kind,
    attemptReasons,
  },);
}

/**
 Run one attempt's invocations concurrently and join their output.
 
 @param runner - gh command boundary
 
 @param attempt - planned attempt
 
 @param url - fetched URL used in binary content notices
 
 @param signal - caller-owned cancellation signal, when supplied
 
 @returns joined output text, or the first required invocation failure
 
 @throws when the caller-owned signal cancels an invocation
 
 @mutates signal - gh children register abort listeners on it.
 
 @example
 ```ts
 await runAttempt({ runner, attempt: { label: 'gist abc', invocations: [] }, url: 'https://gist.github.com/abc' });
 ```
 */
async function runAttempt(
  {
    runner,
    attempt,
    url,
    signal,
  }: {
    readonly runner: GhCommandRunner;
    readonly attempt: GhFetchAttempt;
    readonly url: string;
    readonly signal?: AbortSignal;
  },
): Promise<AttemptResult> {
  /**
   Logger tagged for this attempt.
   */
  const innerL = tagged({
    tag: runAttempt.name,
    l,
  },);
  /**
   Invocation outcomes paired with their planned invocations.
   */
  const results = await Promise.all(attempt.invocations
    .map(async function mapInvocation(invocation: GhInvocation,): Promise<InvocationResult> {
      /**
       gh command request carrying the caller-owned signal.
       */
      const request: GhCommandRequest = {
        args: invocation.args,
        ...(signal === undefined ? {} : { signal, }),
      };
      return {
        invocation,
        outcome: await runner(request,),
      };
    },),);
  /**
   Each invocation rendered as output text or as a failure reason.
   */
  const described = results.map(function mapResult(result: InvocationResult,): InvocationDescription {
    return describeInvocation({
      result,
      url,
    },);
  },);
  /**
   First failed required invocation, when one failed.
   */
  const requiredFailure = described.find(function findRequiredFailure(entry: InvocationDescription,): boolean {
    return entry.failed && entry.required;
  },);
  if (requiredFailure?.failed === true)
    return {
      succeeded: false,
      reason: requiredFailure.reason,
      retryable: requiredFailure.retryable,
    };

  /**
   First failed optional invocation, logged because it leaves a visible gap notice in output.
   */
  const optionalFailure = described.find(function findOptionalFailure(entry: InvocationDescription,): boolean {
    return entry.failed && (!entry.required);
  },);
  if (optionalFailure?.failed === true)
    innerL.warn(`optional gh invocation failed for ${attempt.label}: ${optionalFailure.reason}`,);

  /**
   Non-empty output parts in invocation order.
   */
  const parts = described.flatMap(function collectText(entry: InvocationDescription,): readonly string[] {
    return entryTextParts(entry,);
  },);
  return {
    succeeded: true,
    text: parts.join(ATTEMPT_OUTPUT_SEPARATOR,),
  };
}

/**
 Collect one invocation's non-empty output text.
 
 @param entry - rendered invocation description
 
 @returns one output part, or none when the invocation failed or printed nothing
 
 @example
 ```ts
 entryTextParts({ required: true, failed: false, text: '# Title' });
 ```
 */
function entryTextParts(entry: InvocationDescription,): readonly string[] {
  if (!entry.failed) {
    /**
     Output text contributed by this invocation.
     */
    const { text, } = entry;
    return text === ''
      ? []
      : [
        text,
      ];
  }
  // A required failure already fails the attempt; only an optional failure needs a visible gap notice.
  return entry.required
    ? []
    : [
      optionalFailureNotice(entry.reason,),
    ];
}

/**
 Build one model-visible notice replacing an omitted optional read.
 
 @param reason - safe failure reason from the optional invocation
 
 @returns notice text naming the omission and its cause
 
 @example
 ```ts
 optionalFailureNotice('gh needs authentication, run gh auth login or set GH_TOKEN (exit 4)');
 ```
 */
function optionalFailureNotice(reason: string,): string {
  return `[optional gh read failed, so its output is omitted: ${reason}]`;
}

/**
 Render one invocation outcome as output text or as a failure reason.
 
 @param result - invocation paired with its outcome
 
 @param url - fetched URL used in binary content notices
 
 @returns rendered invocation description
 
 @example
 ```ts
 describeInvocation({ result: { invocation: { args: [], required: true }, outcome: { ran: false, reason: 'gh was terminated' } }, url: 'https://github.com/cli/cli' });
 ```
 */
function describeInvocation(
  {
    result,
    url,
  }: {
    readonly result: InvocationResult;
    readonly url: string;
  },
): InvocationDescription {
  /**
   Planned invocation and its reported outcome.
   */
  const {
    invocation,
    outcome,
  } = result;
  if (!outcome.ran)
    return {
      required: invocation.required,
      failed: true,
      reason: outcome.reason,
      retryable: false,
    };
  if (outcome.exitCode !== 0)
    return {
      required: invocation.required,
      failed: true,
      reason: exitFailureReason({
        exitCode: outcome.exitCode,
        stderr: outcome.stderr,
      },),
      retryable: outcome.exitCode === GH_RETRYABLE_EXIT_CODE,
    };
  return {
    required: invocation.required,
    failed: false,
    text: outcome.stdoutIsText
      ? outcome.stdout
      : binaryContentNotice({
        url,
        byteLength: outcome.stdoutByteLength,
      },),
  };
}

//endregion Attempt execution

//region Failure and notice text

/**
 Build one failure reason for a non-zero gh exit.
 
 @param exitCode - numeric gh exit code
 
 @param stderr - captured standard error
 
 @returns safe failure reason naming documented exit meaning and stderr excerpt
 
 @example
 ```ts
 exitFailureReason({ exitCode: 4, stderr: 'To get started with GitHub CLI, please run: gh auth login' });
 ```
 */
function exitFailureReason(
  {
    exitCode,
    stderr,
  }: {
    readonly exitCode: number;
    readonly stderr: string;
  },
): string {
  /**
   Documented meaning for this exit code, when gh documents one.
   */
  const meaning = GH_EXIT_CODE_MEANINGS[exitCode];
  /**
   Single-line standard error excerpt.
   */
  const excerpt = stderrExcerpt(stderr,);
  /**
   Exit description naming the documented meaning when one exists.
   */
  const exitDescription = meaning === undefined
    ? `gh exited with code ${String(exitCode,)}`
    : `${meaning} (exit ${String(exitCode,)})`;
  return excerpt === ''
    ? exitDescription
    : `${exitDescription}: ${excerpt}`;
}

/**
 Collapse captured standard error into one bounded line.
 
 @param stderr - captured standard error
 
 @returns single-line excerpt, empty when standard error carried no content
 
 @example
 ```ts
 stderrExcerpt('gh: Not Found (HTTP 404)\n');
 ```
 */
function stderrExcerpt(stderr: string,): string {
  /**
   Non-empty trimmed standard error lines.
   */
  const lines = stderr
    .split('\n',)
    .map(function trimLine(line: string,): string {
      return line.trim();
    },)
    .filter(function keepContent(line: string,): boolean {
      return line !== '';
    },);
  /**
   Joined single-line standard error text.
   */
  const joined = lines.join(' ',);
  return joined.length <= GH_STDERR_EXCERPT_CHARACTERS
    ? joined
    : `${joined.slice(
      0,
      GH_STDERR_EXCERPT_CHARACTERS,
    )}${EXCERPT_TRUNCATION_SUFFIX}`;
}

/**
 Build one notice replacing binary gh output that cannot render as text.
 
 @param url - fetched URL
 
 @param byteLength - captured binary output size
 
 @returns notice text
 
 @example
 ```ts
 binaryContentNotice({ url: 'https://github.com/cli/cli/blob/trunk/logo.png', byteLength: 76424 });
 ```
 */
function binaryContentNotice(
  {
    url,
    byteLength,
  }: {
    readonly url: string;
    readonly byteLength: number;
  },
): string {
  return `[gh returned ${String(byteLength,)} bytes of non-UTF-8 binary content for ${url}; text rendering omitted.]`;
}

//endregion Failure and notice text

export {
  createGhClient,
  GhFetchError,
};
