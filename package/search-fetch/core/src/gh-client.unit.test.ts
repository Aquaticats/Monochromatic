/**
 Unit tests for the gh-backed fetch client.
 
 @module
 */

import { setImmediate as nextTurn, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createGhClient,
  GhFetchError,
  type GhClientFetchOptions,
  type GhCommandOutcome,
  type GhCommandRan,
  type GhCommandRequest,
  type GhCommandRunner,
  type GhFetchAttempt,
  type GhInvocation,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Fetched URL used across gh client cases.
 */
const FETCH_URL = 'https://github.com/cli/cli/issues/13118';

/**
 Required invocation argument vector for a body read.
 */
const BODY_ARGS: readonly string[] = [
  'issue',
  'view',
  FETCH_URL,
];

/**
 Optional invocation argument vector for a comment read.
 */
const COMMENT_ARGS: readonly string[] = [
  'issue',
  'view',
  FETCH_URL,
  '--comments',
];

/**
 Captured output byte count used for the binary content notice.
 */
const BINARY_BYTE_LENGTH = 76_424;

/**
 Standard error excerpt length ceiling mirrored from the client.
 */
const STDERR_EXCERPT_CHARACTERS = 400;

/**
 Completed run fixture fields.
 */
type RanFixture = {
  /**
   Captured standard output text.
   */
  readonly stdout?: string;
  /**
   Captured standard error text.
   */
  readonly stderr?: string;
  /**
   Numeric child exit code.
   */
  readonly exitCode?: number;
  /**
   Whether standard output is safe to render as text.
   */
  readonly stdoutIsText?: boolean;
  /**
   Captured standard output byte count.
   */
  readonly stdoutByteLength?: number;
};

/**
 Recording runner harness.
 */
type RunnerHarness = {
  /**
   Runner injected into the gh client.
   */
  readonly runner: GhCommandRunner;
  /**
   Requests received in call order.
   */
  readonly calls: GhCommandRequest[];
};

/**
 Build one completed run outcome.
 
 @param fixture - completed run fields
 
 @returns completed run outcome
 */
function ranOutcome(fixture: RanFixture = {},): GhCommandRan {
  /**
   Captured standard output text.
   */
  const stdout = fixture.stdout ?? '';
  return {
    ran: true,
    exitCode: fixture.exitCode ?? 0,
    stdout,
    stdoutIsText: fixture.stdoutIsText ?? true,
    stdoutByteLength: fixture.stdoutByteLength ?? stdout.length,
    stderr: fixture.stderr ?? '',
  };
}

/**
 Build one required invocation.
 
 @param args - gh argument vector
 
 @returns required invocation
 */
function requiredInvocation(args: readonly string[],): GhInvocation {
  return {
    args,
    required: true,
  };
}

/**
 Build one optional invocation.
 
 @param args - gh argument vector
 
 @returns optional invocation
 */
function optionalInvocation(args: readonly string[],): GhInvocation {
  return {
    args,
    required: false,
  };
}

/**
 Build one attempt.
 
 @param label - attempt label
 
 @param invocations - attempt invocations
 
 @returns planned attempt
 */
function attempt(
  {
    label,
    invocations,
  }: {
    readonly label: string;
    readonly invocations: readonly GhInvocation[];
  },
): GhFetchAttempt {
  return {
    label,
    invocations,
  };
}

/**
 Build one issue thread attempt whose comment read is optional.
 
 @returns planned thread attempt
 */
function threadAttempt(): GhFetchAttempt {
  return attempt({
    label: 'issue cli/cli#13118 body and comments',
    invocations: [
      requiredInvocation(BODY_ARGS,),
      optionalInvocation(COMMENT_ARGS,),
    ],
  },);
}

/**
 Build one gh client request.
 
 @param attempts - planned attempts
 
 @param signal - optional caller cancellation signal
 
 @returns gh client fetch options
 */
function fetchOptions(
  {
    attempts,
    signal,
  }: {
    readonly attempts: readonly GhFetchAttempt[];
    readonly signal?: AbortSignal;
  },
): GhClientFetchOptions {
  return {
    url: FETCH_URL,
    kind: 'issue-thread',
    attempts,
    ...(signal === undefined ? {} : { signal, }),
  };
}

/**
 Build a runner returning scripted outcomes in call order.
 
 @param outcomes - outcomes returned in call order
 
 @returns runner harness recording every request
 */
function scriptedRunner(outcomes: readonly GhCommandOutcome[],): RunnerHarness {
  /**
   Recorded requests.
   */
  const calls: GhCommandRequest[] = [];
  return {
    calls,
    runner: async function respond(request: GhCommandRequest,): Promise<GhCommandOutcome> {
      calls.push(request,);
      /**
       Scripted outcome for this call index.
       */
      const scripted = outcomes[calls.length - 1];
      if (scripted === undefined)
        throw new Error(`no scripted gh outcome for call ${String(calls.length,)}`);
      return scripted;
    },
  };
}

/**
 Capture one rejection without `.rejects` indirection.
 
 @param pending - gh client fetch promise
 
 @returns caught rejection value
 */
async function caughtRejection(pending: Promise<unknown>,): Promise<unknown> {
  try {
    await pending;
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('expected the gh client to reject',);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: createGhClient.name,
      children: [
        //region Attempt ordering

        it({
          name: 'returns the first successful attempt output as markdown',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'name:\tcli/cli', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                attempt({
                  label: 'repository cli/cli overview and README',
                  invocations: [
                    requiredInvocation([
                      'repo',
                      'view',
                      'cli/cli',
                    ],),
                  ],
                },),
              ],
            },),);

            expect(response,).toEqual({ markdown: 'name:\tcli/cli', },);
            expect(harness.calls,).toHaveLength(1,);
            expect(harness.calls[0]?.args,).toEqual([
              'repo',
              'view',
              'cli/cli',
            ],);
          },
        },),
        it({
          name: 'runs the next attempt after a required invocation exits with the ordinary failure code',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                exitCode: 1,
                stderr: 'gh: No commit found for the ref 8761 (HTTP 404)',
              },),
              ranOutcome({ stdout: 'module github.com/cli/cli/v2', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                attempt({
                  label: 'file content attempt 1 of 2 at ref 8761 path allow-items/go.mod',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/allow-items/go.mod?ref=8761',
                    ],),
                  ],
                },),
                attempt({
                  label: 'file content attempt 2 of 2 at ref 8761/allow-items path go.mod',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/go.mod?ref=8761/allow-items',
                    ],),
                  ],
                },),
              ],
            },),);

            expect(response,).toEqual({ markdown: 'module github.com/cli/cli/v2', },);
            expect(harness.calls,).toHaveLength(2,);
            expect(harness.calls[1]?.args,).toEqual([
              'api',
              '/repos/cli/cli/contents/go.mod?ref=8761/allow-items',
            ],);
          },
        },),
        it({
          name: 'stops after a required invocation does not run',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              {
                ran: false,
                reason: 'gh executable gh was not found on PATH',
              },
              ranOutcome({ stdout: '# GitHub CLI', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const pendingOptions = fetchOptions({
              attempts: [
                attempt({
                  label: 'repository cli/cli overview and README',
                  invocations: [
                    requiredInvocation([
                      'repo',
                      'view',
                      'cli/cli',
                    ],),
                  ],
                },),
                attempt({
                  label: 'repository cli/cli overview and README retry',
                  invocations: [
                    requiredInvocation([
                      'repo',
                      'view',
                      'cli/cli',
                    ],),
                  ],
                },),
              ],
            },);
            /**
             Local value for caught.
             */
            const caught = await caughtRejection(client.fetch(pendingOptions,),);

            expect(caught,).toBeInstanceOf(GhFetchError,);
            expect((caught as GhFetchError).attemptReasons,).toHaveLength(1,);
            expect(harness.calls,).toHaveLength(1,);
          },
        },),
        it({
          name: 'stops after a required invocation reports the authentication exit code',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                exitCode: 4,
                stderr: 'To get started with GitHub CLI, please run:  gh auth login',
              },),
              ranOutcome({ stdout: 'module github.com/cli/cli/v2', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const pendingOptions = fetchOptions({
              attempts: [
                attempt({
                  label: 'file content attempt 1 of 2 at ref 8761 path allow-items/go.mod',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/allow-items/go.mod?ref=8761',
                    ],),
                  ],
                },),
                attempt({
                  label: 'file content attempt 2 of 2 at ref 8761/allow-items path go.mod',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/go.mod?ref=8761/allow-items',
                    ],),
                  ],
                },),
              ],
            },);
            /**
             Local value for caught.
             */
            const caught = await caughtRejection(client.fetch(pendingOptions,),);

            expect(caught,).toBeInstanceOf(GhFetchError,);
            expect((caught as GhFetchError).attemptReasons,).toHaveLength(1,);
            expect(harness.calls,).toHaveLength(1,);
          },
        },),
        it({
          name: 'throws after every attempt fails and names the URL, kind, and every attempt reason',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                exitCode: 1,
                stderr: 'gh: No commit found for the ref 8761 (HTTP 404)',
              },),
              ranOutcome({
                exitCode: 1,
                stderr: 'gh: Not Found (HTTP 404)',
              },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  attempt({
                    label: 'first split',
                    invocations: [
                      requiredInvocation([
                        'api',
                        '/repos/cli/cli/contents/allow-items/go.mod?ref=8761',
                      ],),
                    ],
                  },),
                  attempt({
                    label: 'second split',
                    invocations: [
                      requiredInvocation([
                        'api',
                        '/repos/cli/cli/contents/go.mod?ref=8761/allow-items',
                      ],),
                    ],
                  },),
                ],
              },),),
            );

            expect(caught,).toBeInstanceOf(GhFetchError,);
            /**
             Local value for failure.
             */
            const failure = caught as GhFetchError;
            expect(failure.url,).toBe(FETCH_URL,);
            expect(failure.kind,).toBe('issue-thread',);
            expect(failure.attemptReasons,).toHaveLength(2,);
            expect(failure.attemptReasons[0],).toContain('first split',);
            expect(failure.attemptReasons[0],).toContain('No commit found for the ref 8761',);
            expect(failure.attemptReasons[1],).toContain('second split',);
            expect(failure.attemptReasons[1],).toContain('Not Found (HTTP 404)',);
            expect(failure.message,).toContain(FETCH_URL,);
            expect(failure.message,).toContain('issue-thread',);
          },
        },),
        it({
          name: 'throws when a plan carries no attempts',
          fn: async () => {
            /**
             Local value for client.
             */
            const client = createGhClient({
              runner: scriptedRunner([],).runner,
            },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({ attempts: [], },),),
            );

            expect(caught,).toBeInstanceOf(GhFetchError,);
            expect((caught as GhFetchError).attemptReasons,).toHaveLength(0,);
          },
        },),

        //endregion Attempt ordering

        //region Optional invocations

        it({
          name: 'appends a visible notice when an optional comment read fails',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'title:\tbrew install gh', },),
              ranOutcome({
                exitCode: 1,
                stderr: 'GraphQL: Could not resolve to an issue',
              },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
            },),);
            /**
             Local value for markdown.
             */
            const {
              markdown,
            } = response as { readonly markdown: string; };

            expect(markdown.startsWith('title:\tbrew install gh\n\n[optional gh read failed',),).toBe(true,);
            expect(markdown,).toContain('its output is omitted',);
            expect(markdown,).toContain('Could not resolve to an issue',);
          },
        },),
        it({
          name: 'appends a visible notice when an optional comment read does not run',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'title:\tbrew install gh', },),
              {
                ran: false,
                reason: 'gh was terminated after exceeding the 60000ms deadline',
              },
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
            },),);
            /**
             Local value for markdown.
             */
            const {
              markdown,
            } = response as { readonly markdown: string; };

            expect(markdown.startsWith('title:\tbrew install gh\n\n[optional gh read failed',),).toBe(true,);
            expect(markdown,).toContain('60000ms deadline',);
          },
        },),
        it({
          name: 'joins required and optional output with one blank line',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'title:\tbrew install gh', },),
              ranOutcome({ stdout: 'author:\tgithub-actions', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
            },),);

            expect(response,).toEqual({ markdown: 'title:\tbrew install gh\n\nauthor:\tgithub-actions', },);
          },
        },),
        it({
          name: 'skips an empty optional output without adding a separator',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'title:\tbrew install gh', },),
              ranOutcome({ stdout: '', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
            },),);

            expect(response,).toEqual({ markdown: 'title:\tbrew install gh', },);
          },
        },),
        it({
          name: 'returns empty markdown when a required invocation succeeds with no output',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: '', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                attempt({
                  label: 'file content attempt 1 of 1 at ref trunk path .empty',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/.empty?ref=trunk',
                    ],),
                  ],
                },),
              ],
            },),);

            expect(response,).toEqual({ markdown: '', },);
          },
        },),
        it({
          name: 'preserves whitespace-only file content exactly',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: '  \n\t\n', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                attempt({
                  label: 'file content attempt 1 of 1 at ref trunk path .keep',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/.keep?ref=trunk',
                    ],),
                  ],
                },),
              ],
            },),);

            expect(response,).toEqual({ markdown: '  \n\t\n', },);
          },
        },),

        //endregion Optional invocations

        //region Failure and binary text

        it({
          name: 'replaces non-UTF-8 output with a byte-count notice',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                stdout: 'wOF2\uFFFD\uFFFD',
                stdoutIsText: false,
                stdoutByteLength: BINARY_BYTE_LENGTH,
              },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for response.
             */
            const response = await client.fetch(fetchOptions({
              attempts: [
                attempt({
                  label: 'file content attempt 1 of 1 at ref main path font.woff2',
                  invocations: [
                    requiredInvocation([
                      'api',
                      '/repos/cli/cli/contents/font.woff2?ref=main',
                    ],),
                  ],
                },),
              ],
            },),);
            /**
             Local value for markdown.
             */
            const {markdown} = (response as { readonly markdown: string; });

            expect(markdown,).toContain(`${String(BINARY_BYTE_LENGTH,)} bytes of non-UTF-8 binary content`,);
            expect(markdown,).toContain(FETCH_URL,);
            expect(markdown,).toContain('text rendering omitted',);
          },
        },),
        it({
          name: 'names authentication for gh exit code 4',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                exitCode: 4,
                stderr: 'To get started with GitHub CLI, please run:  gh auth login',
              },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  attempt({
                    label: 'repository cli/cli overview and README',
                    invocations: [
                      requiredInvocation([
                        'repo',
                        'view',
                        'cli/cli',
                      ],),
                    ],
                  },),
                ],
              },),),
            );
            /**
             Local value for reason.
             */
            const reason = (caught as GhFetchError).attemptReasons[0] ?? '';

            expect(reason,).toContain('needs authentication',);
            expect(reason,).toContain('gh auth login',);
            expect(reason,).toContain('exit 4',);
          },
        },),
        it({
          name: 'names cancellation for gh exit code 2',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ exitCode: 2, },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  attempt({
                    label: 'repository cli/cli overview and README',
                    invocations: [
                      requiredInvocation([
                        'repo',
                        'view',
                        'cli/cli',
                      ],),
                    ],
                  },),
                ],
              },),),
            );

            expect((caught as GhFetchError).attemptReasons[0],).toContain('cancelled the request',);
          },
        },),
        it({
          name: 'names the exit code alone when standard error is empty',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ exitCode: 1, },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  attempt({
                    label: 'repository cli/cli overview and README',
                    invocations: [
                      requiredInvocation([
                        'repo',
                        'view',
                        'cli/cli',
                      ],),
                    ],
                  },),
                ],
              },),),
            );

            expect((caught as GhFetchError).attemptReasons[0],).toBe(
              'repository cli/cli overview and README: gh exited with code 1',
            );
          },
        },),
        it({
          name: 'collapses multiline standard error into one bounded excerpt',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({
                exitCode: 1,
                stderr: `${'x'.repeat(STDERR_EXCERPT_CHARACTERS + 100,)}\nsecond line\n`,
              },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  attempt({
                    label: 'repository cli/cli overview and README',
                    invocations: [
                      requiredInvocation([
                        'repo',
                        'view',
                        'cli/cli',
                      ],),
                    ],
                  },),
                ],
              },),),
            );
            /**
             Local value for reason.
             */
            const reason = (caught as GhFetchError).attemptReasons[0] ?? '';

            expect(reason,).toContain('...',);
            expect(reason,).toContain(('x'.repeat(STDERR_EXCERPT_CHARACTERS,)),);
            expect(reason.includes('second line',),).toBe(false,);
          },
        },),

        //endregion Failure and binary text

        //region Runner contract

        it({
          name: 'propagates a runner rejection instead of falling back',
          fn: async () => {
            /**
             Local value for cancellation.
             */
            const cancellation = new Error('The operation was aborted',);
            /**
             Local value for client.
             */
            const client = createGhClient({
              runner: async function reject(): Promise<GhCommandOutcome> {
                throw cancellation;
              },
            },);

            /**
             Local value for caught.
             */
            const caught = await caughtRejection(
              client.fetch(fetchOptions({
                attempts: [
                  threadAttempt(),
                ],
              },),),
            );

            expect(caught,).toBe(cancellation,);
          },
        },),
        it({
          name: 'starts every invocation of one attempt before any of them settles',
          fn: async () => {
            /**
             Local value for activeCount.
             */
            let activeCount = 0;
            /**
             Local value for maxActiveCount.
             */
            let maxActiveCount = 0;
            /**
             Local value for client.
             */
            const client = createGhClient({
              runner: async function overlap(): Promise<GhCommandOutcome> {
                activeCount += 1;
                maxActiveCount = Math.max(
                  maxActiveCount,
                  activeCount,
                );
                await nextTurn();
                activeCount -= 1;
                return ranOutcome({ stdout: 'part', },);
              },
            },);

            await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
            },),);

            expect(maxActiveCount,).toBe(2,);
            expect(activeCount,).toBe(0,);
          },
        },),
        it({
          name: 'forwards the caller cancellation signal to every invocation',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            /**
             Local value for harness.
             */
            const harness = scriptedRunner([
              ranOutcome({ stdout: 'title:\tbrew install gh', },),
              ranOutcome({ stdout: '', },),
            ],);
            /**
             Local value for client.
             */
            const client = createGhClient({ runner: harness.runner, },);

            await client.fetch(fetchOptions({
              attempts: [
                threadAttempt(),
              ],
              signal: controller.signal,
            },),);

            expect(harness.calls[0]?.signal,).toBe(controller.signal,);
            expect(harness.calls[1]?.signal,).toBe(controller.signal,);
          },
        },),

        //endregion Runner contract
      ],
    },),
  ],
},);
