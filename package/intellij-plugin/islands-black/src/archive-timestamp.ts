/**
 Year the ZIP format's epoch starts: the earliest date its headers can encode.
 */
const ARCHIVE_EPOCH_YEAR = 1_980;

/**
 The one modification time stamped on every member of the plugin archive.

 Fixed at the ZIP format's own epoch (1980-01-01) so a member never records when
 the build ran. A member timestamp is build output rather than input, and this
 archive is committed: a clock that reaches the file turns every build into a
 byte diff the committer has to discard.

 @example
 ```ts
 const zipWriter = new ZipWriter({ modifiedAt: ARCHIVE_MODIFIED_AT, });
 ```
 */
export const ARCHIVE_MODIFIED_AT: Date = new Date(Date.UTC(
  ARCHIVE_EPOCH_YEAR,
  0,
  1,
),);
