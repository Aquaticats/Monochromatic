/**
 What one guest run names on the host side: the guest it drives and the folder its own logs go in.

 A driver's own options type is wider than this; a command that only needs the guest's name or the log folder takes one
 of these, so it stays usable by every driver without knowing what else that driver's command line carries. The three
 sets of guest helpers each carried a copy of these two shapes (`GuestFacts`, `GuestRun` and `GuestRunFolder`), and the
 `mvm` steps answer with the one copy.

 @example
 ```ts
 const chosen: GuestFacts = { name: 'git-policy-lock', logDir: 'target/lock-guest-logs' };
 ```
 */

/**
 The folder one run writes its host-side logs in.
 */
export type GuestLogFolder = {
  /**
   The folder's path, which every host command's log lands in.
   */
  readonly logDir: string;
};

/**
 One run on the host side: the guest it drives and where its logs go.
 */
export type GuestFacts = GuestLogFolder & {
  /**
   The guest's `mvm` name, which every command names so a run never reaches another run's guest.
   */
  readonly name: string;
};
