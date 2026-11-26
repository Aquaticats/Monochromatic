/**
 What a worktree launch's run takes, or the refusal this command starts nothing for.

 The type sits in a file of its own because two modules build one: the one that places a run on a single pool, and the one that
 places it on a virtual selection. A type each of them answers with, kept beside either, would have them use each other in a loop.

 @module
 */

/**
 What the launch's run takes, or why this command starts none.
 */
export type WorktreeLaunchPlacement =
  | {
    /**
     Whether the run takes its pair and not a refusal.
     */
    readonly took: 'a-run';

    /**
     The provider the run's unit and driver are given.
     */
    readonly provider: string;

    /**
     The model the run's unit and driver are given.
     */
    readonly model: string;

    /**
     How long the run may last, in seconds: the caller's own wall clock, or that clock stretched for a pool that answers more
     slowly.
     */
    readonly wallSeconds: number;

    /**
     The lines this command prints about the pool it chose, which are none when the command line named the pair the run takes.
     */
    readonly lines: readonly string[];
  }
    | {
    /**
     Whether the run takes its pair and not a refusal.
     */
      readonly took: 'no-run';

    /**
     The status this command exits with, which is the pool gate's own for a closed pool and for no room.
     */
      readonly status: number;

    /**
     What the run is refused with, one line per thing a reader needs.
     */
      readonly message: string;
    };
