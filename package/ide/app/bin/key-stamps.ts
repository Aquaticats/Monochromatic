// The stamps the IDE writes for the keys it handles.
// What: the log record that accounts for one handled key, and the times of those records in a session's log text.
// Why: a stalled window is counted twice, from the compositor's picture and from the pause between two keys the window
//      handled. The second account reads the application's own log, so the record it reads has one named owner here, and
//      a module that moves its logging cannot silently leave the reading counting nothing.

/**
 The log record the application writes for each key it handles.

 What: the module path the record carries, which is `src/native/caret_keys.rs` since caret movement moved out of
       `src/native/input.rs`.
 Why: this is the only record the pause between two handled keys is read from, and a module that moves takes its
      records with it: the check counted nothing while this named the module the keys had left.
 */
export const KEY_STAMP_RECORD = 'monochromatic_ide::native::caret_keys:';

/**
 What: the time of every record in `text` that accounts for a handled key.
 Why: the longest pause between two of them is the window's own account of how long it could not handle input.

 @param text - the session's log, with its colour codes already removed
 @returns the record times in milliseconds, in the order the log holds them
 */
export function keyStamps(text: string): number[] {
  return text.split('\n').filter(line => line.includes(KEY_STAMP_RECORD)).map(line => Date.parse(line.slice(0, 27)));
}
