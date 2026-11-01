//region Forbidden publish retry

/**
 Text npm prints when the registry answers a publish with HTTP 403.
 */
export const NPM_FORBIDDEN_MARKER = 'npm error code E403';

/**
 Pause between forbidden publish attempts.
 pnpr-publish run 34922532212 got `E403` for new names at 02:48:23 to 02:48:29
 and published the next new name at 02:48:43,
 so a few of these pauses cover a Coolify redeploy.
 */
export const FORBIDDEN_RETRY_DELAY_MS = 20_000;

/**
 How long after the run starts forbidden publishes are retried.
 The race only exists while the push that changed `config.yaml` is still redeploying the registry;
 after the window a 403 is a real trust failure and fails without waiting.
 */
export const FORBIDDEN_RETRY_WINDOW_MS = 300_000;

/**
 Reports whether a failed publish carried npm's HTTP 403 error code in its captured output.

 @param error - Value thrown by the publish attempt.

 @returns Whether the failure is a registry authorization refusal.

 @example
 ```ts
 isForbiddenPublishError(Object.assign(new Error('failed'), { output: 'npm error code E403' }));
 // => true
 ```
 */
export function isForbiddenPublishError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('output' in error)
    && ((typeof error.output) === 'string')
    && error.output
      .includes(NPM_FORBIDDEN_MARKER,);
}

/**
 Runs a publish attempt, retrying `E403` refusals with a fixed pause while the run's retry deadline allows.
 Each attempt should request its own token, since workload credentials are short-lived.

 @param attempt - One token request plus publish.

 @param deadline - Epoch milliseconds after which no retry starts.

 @param now - Current epoch milliseconds.

 @param sleep - Pause implementation.

 @param onRetry - Reports each retry before its pause.

 @throws The attempt's error when it is not an `E403` refusal, or when the last allowed attempt still fails.

 @example
 ```ts
 await publishWithForbiddenRetry({ attempt, deadline: Date.now() + FORBIDDEN_RETRY_WINDOW_MS, now: Date.now, sleep: wait, onRetry });
 ```
 */
export async function publishWithForbiddenRetry(
  {
    attempt,
    deadline,
    now,
    sleep,
    onRetry,
  }: {
    readonly attempt: () => Promise<void>;
    readonly deadline: number;
    readonly now: () => number;
    readonly sleep: (milliseconds: number) => Promise<unknown>;
    readonly onRetry: (retry: {
      readonly retryNumber: number;
      readonly delayMs: number
    },) => void;
  },
): Promise<void> {
  /**
   Retries whose pauses fit before the deadline, fixed when the first attempt starts.
   */
  const retryCount = Math.max(
    0,
    Math.floor((deadline - now()) / FORBIDDEN_RETRY_DELAY_MS,),
  );
  for (const retryNumber of Array.from(
    { length: retryCount, },
    function toRetryNumber(
      _unused,
      index,
    ) {
      return index + 1;
    },
  )) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- attempts are sequential by definition; a retry follows a refusal.
      await attempt();
      return;
    }
    catch (error: unknown) {
      if (!isForbiddenPublishError(error,))
        throw error;
      onRetry({
        retryNumber,
        delayMs: FORBIDDEN_RETRY_DELAY_MS,
      },);
      // oxlint-disable-next-line eslint/no-await-in-loop -- the pause must finish before the next attempt.
      await sleep(FORBIDDEN_RETRY_DELAY_MS,);
    }
  }
  await attempt();
}

//endregion Forbidden publish retry
