//region One-way flag
// A flag that can be raised and never lowered, so a fact met once in a run (a billing error, the first limits line)
// cannot be forgotten by a later call. It sat in `claude-client.ts` until that file passed the package's line limit.

/**
 A flag that can be raised and never lowered, so a fact met once in a run (a billing error, the first limits line)
 cannot be forgotten by a later call.

 CARRIED BY AN ABORT CONTROLLER, whose signal goes aborted once and stays: raising again changes nothing, and nothing
 here grows or rebinds.

 @returns The flag's reader and its raiser

 @example
 ```ts
 const refused = oneWayFlag();
 refused.raise();
 ```
 */
export function oneWayFlag(): {
  readonly isRaised: () => boolean;
  readonly raise: () => void;
} {
  /**
   Aborted once the flag is raised.
   */
  const raised = new AbortController();
  return {
    isRaised: function isRaised(): boolean {
      return raised.signal
        .aborted;
    },
    raise: function raise(): void {
      raised.abort();
    },
  };
}

//endregion One-way flag
