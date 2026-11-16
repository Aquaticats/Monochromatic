/**
 What one `up` made, named one thing at a time as it made it.

 A failed `up` rolls back through this record and removes only what it made,
 so a start that made nothing removes nothing.
 That is what keeps a start that lost a race from taking down the tunnel of the
 start that won it: its rollback has no entry for a link, a policy rule or a
 kill switch, so it has nothing to delete.

 @module
 */

/**
 One thing a start can make, named by its entry in {@link UpMade}.
 */
export type UpMadeThing =
  | 'link'
    | 'endpointAllowance'
    | 'dns'
    | 'policyRules'
    | 'exemptRouting'
    | 'killSwitch'
    | 'applicationWatcher';

/**
 The kernel-visible things a start can make,
 each recorded at the moment the start begins to make it.

 A thing is recorded before the command that makes it,
 because a command that fails partway can still have made part of it,
 and removing a thing that was never made is the safe direction:
 every removal this record drives is written to tolerate something absent.
 */
export type UpMade = {
  /**
   The link this start created with `ip link add`.
   The one entry recorded after its command, because a refused `ip link add`
   created nothing and the link that refused it belongs to another start.
   */
  link: boolean;

  /**
   The OpenSnitch endpoint allowance this start installed.
   */
  endpointAllowance: boolean;

  /**
   The DNS settings this start applied to the link.
   */
  dns: boolean;

  /**
   The policy rules and the policy table this start installed.
   */
  policyRules: boolean;

  /**
   The application-bypass route, its rules and its state that this start installed.
   */
  exemptRouting: boolean;

  /**
   The kill switch this start installed.
   */
  killSwitch: boolean;

  /**
   The application watcher this start asked the companion program to start.
   */
  applicationWatcher: boolean;
};

/**
 Builds the record of a start that has made nothing yet.

 @returns Every entry false.

 @example
 ```ts
 const made = nothingMade();
 made.link = true;
 ```
 */
export function nothingMade(): UpMade {
  return {
    link: false,
    endpointAllowance: false,
    dns: false,
    policyRules: false,
    exemptRouting: false,
    killSwitch: false,
    applicationWatcher: false,
  };
}
