/**
 IPC channel names shared by the main process and the preload bridge.
 
 @example
 ```ts
 console.log(LIST_DIRECTORY_CHANNEL);
 ```
 
 @packageDocumentation
 */

/**
 Invoke channel resolving the directory the first root pane lists.
 
 @example
 ```ts
 console.log(INITIAL_ROOT_CHANNEL);
 ```
 */
export const INITIAL_ROOT_CHANNEL = 'file-manager-electron:initial-root';

/**
 Invoke channel listing one directory into sorted bridge entries.
 
 @example
 ```ts
 console.log(LIST_DIRECTORY_CHANNEL);
 ```
 */
export const LIST_DIRECTORY_CHANNEL = 'file-manager-electron:list-directory';

/**
 Send channel mirroring renderer state to the main process state file.
 
 @example
 ```ts
 console.log(REPORT_STATE_CHANNEL);
 ```
 */
export const REPORT_STATE_CHANNEL = 'file-manager-electron:report-state';
