/**
 What one guest-driving script's run names on the host side: the guest it drives
 and the folder its own logs go in.

 A driver's own options type is wider than this; a command that only needs the
 guest's name or the log folder takes one of these, so it stays usable by every
 script without knowing what else that script's command line carries.

 @example
 ```ts
 const folder: GuestRunFolder = { logDir: 'target/adoption-guest-logs' };
 ```
 */

/**
 The folder one run writes its host-side logs in.
 */
export type GuestRunFolder = {
  /**
   The folder's path, which every host command's log lands in.
   */
  readonly logDir: string;
};

/**
 One run on the host side: the guest it drives and where its logs go.
 */
export type GuestRun = GuestRunFolder & {
  /**
   The guest's `mvm` name, which every command names so a run never reaches
   another run's guest.
   */
  readonly name: string;
};
