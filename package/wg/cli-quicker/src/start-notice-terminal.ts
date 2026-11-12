/**
 Prints one start notice where the person who started the tunnel will see it.

 The notice goes to this program's own standard error and not through a logger
 console sink, because that sink drops `warn` records when its warning switch is
 off and every console record when console logging is off. The decision these
 packages answer to is that a start is always warned about, with no toggle: the
 switch a machine-wide hook sets to keep its protocol streams clean must not be
 able to hide what leaves the tunnel. The two switches keep working for every
 other record, which still go through the logger.

 @param notice - One start notice, as `startNotices` built it.

 @example
 ```ts
 printStartNotice('Config for wg0 has no ExemptMark; …');
 ```
 */
export function printStartNotice(notice: string,): void {
  process.stderr
    .write(`${notice}\n`,);
}
