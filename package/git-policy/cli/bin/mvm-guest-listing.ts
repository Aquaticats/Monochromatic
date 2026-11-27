/**
 What a guest driver reads out of `mvm list`, kept where a unit test can hold it.

 A driver that creates a guest refuses a name `mvm list` already reports, because `mvm create` on a name another run
 holds would either refuse or, worse, adopt that run's guest and destroy it at this run's end. The listing pads each
 name to a fixed column and follows it with the guest's state, so the name is the line's first word.
 */

/**
 Whether a guest of one name is already listed.

 The line's own leading whitespace is skipped, and only the start of the line is compared: `mvm list` writes
 `<name><padding><state>`, so a name is listed when a line begins with it. A name that is a prefix of another guest's
 name reads as listed, which refuses the run rather than risking another run's guest.

 @param listing - what `mvm list` printed, one guest per line

 @param name - the guest name the run would create

 @returns true when a line of the listing begins with that name

 @example
 ```ts
 holdsGuestName({ listing: 'git-policy-lock   running\n', name: 'git-policy-lock' }); // true
 ```
 */
export function holdsGuestName({
  listing,
  name,
}: {
  readonly listing: string;
  readonly name: string;
},): boolean {
  for (const line of listing.split('\n',)) {
    if (line.trimStart()
      .startsWith(name,))
      return true;
  }
  return false;
}
