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

 @param exemptMark - `ExemptMark` of the config,
 absent when none is set.

 @param steps - The steps,
 replaceable in tests.

 @param rollback - Removes what the steps did.

 @throws What the failing step threw,
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
  }: {
    readonly exemptMark?: number;
    readonly steps: UpSteps;
    readonly rollback: () => Promise<void>;
  },
): Promise<void> {
  try {
    await steps.addLink();
    await steps.runPreUp();
    await steps.applyPeerConfig();
    await steps.allowEndpoints();
    await steps.addAddressesAndBringUp();
    await steps.setDns();
    await steps.setupRoutes();
    if (exemptMark !== undefined)
      await steps.startExemptions({ mark: exemptMark, },);
    await steps.runPostUp();
  }
  catch (error: unknown) {
    await rollback();
    throw error;
  }
}
