/**
 The permissions a guest driver gives the folders and the logs it writes.

 Every driver gave each folder it made `0o700` and each log it opened `0o600` before this module held the two values. A
 log of a run holds the guest's own output, the pool it built and the names of the fixtures it destroyed, and this
 machine's `/home` is shared, so nothing another account may read is written into one.
 */

/**
 The permissions a folder a driver makes has: the account that runs the driver, and nobody else.
 */
export const FOLDER_MODE = 0o700;

/**
 The permissions a log a driver writes has: the account that runs the driver, and nobody else.
 */
export const LOG_FILE_MODE = 0o600;
