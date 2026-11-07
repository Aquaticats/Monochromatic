/**
 Types shared by the repository link check, the identity writer and the bins.

 @module
 */

/**
 The two phases the identity file records: `before` while the repository is still a real folder on the NVMe, `after`
 once the old path is a link to the 860.
 */
export type LinkPhase = 'after' | 'before';

/**
 The identity the check enforces, recorded once by the cut-over step.
 */
export type LinkIdentity = {
  /**
   The old repository path, for example `/var/home/user/Monochromatic`.
   */
  readonly linkPath: string;

  /**
   The exact text of the link, for example `/var/mnt/encrypted/repository/Monochromatic`; empty in phase `before`.
   */
  readonly linkText: string;

  /**
   The approved canonical root the link must resolve to.
   */
  readonly canonicalRoot: string;

  /**
   The 860's mount point, for example `/var/mnt/encrypted`.
   */
  readonly mountPoint: string;

  /**
   Filesystem UUID of the 860 (phase `after`).
   */
  readonly driveUuid: string;

  /**
   Filesystem UUID of the NVMe that holds the repository (phase `before`).
   */
  readonly nvmeUuid: string;

  /**
   First commit hash of `main`, also written to `.git/monochromatic-root-commit` (phase `after`).
   */
  readonly rootCommit: string;

  /**
   The phase the identity describes.
   */
  readonly phase: LinkPhase;
};

/**
 What a probe learns about one path without following a final link.
 */
export type EntryInfo = {
  readonly exists: boolean;
  readonly kind: 'directory' | 'file' | 'link' | 'missing' | 'other';
};

/**
 The filesystem questions the check asks. The real probe answers them from the system; tests answer them from
 fixtures, and a test of the deadline answers none.
 */
export type Probe = {
  /**
   Looks at a path without following a final link.
   */
  readonly lstat: (path: string,) => Promise<EntryInfo>;

  /**
   Reads the text of a link.
   */
  readonly readlink: (path: string,) => Promise<string>;

  /**
   Resolves every link in a path; an empty string when the path does not resolve.
   */
  readonly realpath: (path: string,) => Promise<string>;

  /**
   Reads a small text file; an empty string when it cannot be read.
   */
  readonly readText: (path: string,) => Promise<string>;

  /**
   Filesystem UUID of the filesystem that holds a path; an empty string when it has none or cannot be found.
   */
  readonly uuidOf: (path: string,) => Promise<string>;

  /**
   Tells whether a path is a mount point.
   */
  readonly isMountPoint: (path: string,) => Promise<boolean>;
};

/**
 Every answer the check can give.
 */
export type AnswerKind =
  | 'drive-did-not-answer'
    | 'drive-not-mounted'
    | 'identity-missing'
    | 'link-missing'
    | 'mounted-folder-missing'
    | 'not-a-repository'
    | 'ready-after'
    | 'ready-before'
    | 'real-folder-replaced-link'
    | 'wrong-filesystem'
    | 'wrong-repository';

/**
 The check's answer: a kind, whether the repository is ready, and the plain message for a person.
 */
export type LinkAnswer = {
  readonly kind: AnswerKind;
  readonly ready: boolean;
  readonly message: string;
};
