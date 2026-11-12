import type { UpInterruption, } from './interrupt-signal.ts';

/**
 The steps of bringing an interface up,
 each already bound to the config it works on.
 */
export type UpSteps = {
  /**
   Creates the link.
   */
  readonly addLink: () => Promise<void>;

  /**
   Runs the `PreUp` hooks.
   */
  readonly runPreUp: () => Promise<void>;

  /**
   Applies the peer configuration.
   */
  readonly applyPeerConfig: () => Promise<void>;

  /**
   Lets the tunnel's endpoints through the firewall that guards them.
   */
  readonly allowEndpoints: () => Promise<void>;

  /**
   Adds the addresses and brings the link up.
   */
  readonly addAddressesAndBringUp: () => Promise<void>;

  /**
   Sets the DNS servers of the link.
   */
  readonly setDns: () => Promise<void>;

  /**
   Installs the routes and rules.
   */
  readonly setupRoutes: () => Promise<void>;

  /**
   Starts the application exemptions for the configured mark,
   and stops the start when the program that runs them cannot be found.
   */
  readonly startExemptions: (call: { readonly mark: number; },) => Promise<void>;

  /**
   Runs the `PostUp` hooks.
   */
  readonly runPostUp: () => Promise<void>;
};

/**
 Runs the steps of an up in order and rolls back when any of them fails.

 A config without `ExemptMark` skips the exemption step and goes on.
 The owner chose that: "Warn if ExemptMark is unset but continue to up".

 A failing step,
 including the one that stops the start for a missing exemption program,
 runs the rollback once and then rethrows the failure,
 so the interface is not left half configured.

 An interrupt is the same: a signal ends the up at the next step boundary,
 the rollback runs,
 and the error says which signal arrived and what did not run,
 because the default action of a signal would end the process before any rollback.

 @param exemptMark - `ExemptMark` of the config,
 absent when none is set.

 @param steps - The steps,
 replaceable in tests.

 @param rollback - Removes what the steps did.

 @param interruption - Where the up learns that it was interrupted,
 absent when nothing watches for signals.

 @throws What the failing step threw,
 after the rollback.

 @throws {@link UpInterruptedError} when a signal arrived before a step,
 after the rollback.

 @example
 ```ts
 await bringUp({ exemptMark: 100, steps, rollback });
 ```
 */
export async function bringUp(
  {
    exemptMark,
    steps,
    rollback,
    interruption,
  }: {
    readonly exemptMark?: number;
    readonly steps: UpSteps;
    readonly rollback: () => Promise<void>;
    readonly interruption?: UpInterruption;
  },
): Promise<void> {
  /**
   The steps in the order wg-quick runs them,
   each with the words an interrupted up names it by.
   */
  const sequence: readonly {
    readonly does: string;
    readonly step: () => Promise<void>;
  }[] = [
    {
      does: 'add the link',
      step: steps.addLink,
    },
    {
      does: 'run the pre-up hooks',
      step: steps.runPreUp,
    },
    {
      does: 'apply the peer configuration',
      step: steps.applyPeerConfig,
    },
    {
      does: 'let the tunnel endpoints through the firewall',
      step: steps.allowEndpoints,
    },
    {
      does: 'add the addresses and bring the link up',
      step: steps.addAddressesAndBringUp,
    },
    {
      does: 'set the DNS servers',
      step: steps.setDns,
    },
    {
      does: 'install the routes and rules',
      step: steps.setupRoutes,
    },
    ...(exemptMark === undefined ? [] : [
      {
        does: 'start the application exemptions',
        step: function startExemptions(): Promise<void> {
          return steps.startExemptions({ mark: exemptMark, },);
        },
      },
    ]),
    {
      does: 'run the post-up hooks',
      step: steps.runPostUp,
    },
  ];
  /**
   The step the up is on,
   so a failure after an interrupt names it.
   */
  const at = { does: 'start the up', };
  try {
    /* oxlint-disable eslint/no-await-in-loop -- The steps of an up run in order, and each one depends on the ones before it. */
    for (const {
      does,
      step,
    } of sequence) {
      at.does = does;
      interruption?.beforeStep(does,);
      await step();
    }
    /* oxlint-enable eslint/no-await-in-loop */
  }
  catch (error: unknown) {
    await rollback();
    if (interruption === undefined)
      throw error;
    throw interruption.insteadOfStepFailure(
      at.does,
      error,
    );
  }
}
