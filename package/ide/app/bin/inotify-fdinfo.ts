// The watch count of one inotify descriptor, read from the fdinfo text the kernel prints for it.
// What: how many lines of that text open with the marker of one watch.
// Why: the idle-cost measurement counts the watches of a running IDE, and asked for this answer through a global
// pattern with a line-start anchor, which the repository lint bans where one pass over the lines reads the same
// text.
// Tested in inotify-fdinfo.test.ts.
import { isLineTerminator } from './scan-characters.ts';

/** Text that opens the fdinfo line of one inotify watch. */
const watchLineStart = 'inotify wd:';

/**
 Watches one inotify descriptor holds.
 
 @param options - `fdinfo` holds the whole text of the descriptor's fdinfo file.
 
 @returns How many lines of that text open with the watch marker.
 
 @example
 ```ts
 inotifyWatchCount({ fdinfo: 'pos:\t0\ninotify wd:1 ino:1a2b sdev:fd obj:ide\tmask:3ce\n' }); // 1
 ```
 */
export function inotifyWatchCount({ fdinfo }: { fdinfo: string }): number {
  let watches = 0;
  let lineStart = 0;
  // Each line start is visited once, so the text is read in one pass whatever it holds.
  while (lineStart <= fdinfo.length) {
    if (fdinfo.startsWith(watchLineStart, lineStart)) {
      watches++;
    }
    let cursor = lineStart;
    while (cursor < fdinfo.length && !isLineTerminator(fdinfo[cursor] ?? '')) {
      cursor++;
    }
    lineStart = cursor + 1;
  }
  return watches;
}
