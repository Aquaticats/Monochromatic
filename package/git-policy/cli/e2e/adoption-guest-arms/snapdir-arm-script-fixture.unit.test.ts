/**
 Guard: the pattern the `snapdir` arm hands the guest's `grep` takes the checkout's `.zfs` directory and no other name.

 The listing the arm greps is `ls -a` of the checkout, so a real `grep` reads that listing here: what the arm asserts is
 that `.zfs` is one of the lines, and a pattern whose dot matches any character also takes a name that merely ends in
 `zfs`.
 */
import { spawnSync, } from 'node:child_process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { snapdirScript, } from './snapdir-arm-script-fixture.ts';

/**
 The words the line that greps the listing begins its pattern with.
 */
const PATTERN_OPENS_WITH = 'grep --quiet "';

/**
 The names a listing of the checkout may hold beside `.zfs`: one a dot standing for any character would take, and one
 that begins with the directory's own name.
 */
const OTHER_NAMES: readonly string[] = [
  'xzfs',
  '.zfsx',
];

/**
 The pattern the arm hands the guest's `grep`, read out of the script text between the quotes of the line that greps the
 listing.

 @returns the pattern exactly as the guest's shell reads it

 @throws Error when the script holds no line that greps the listing, which is a script this guard cannot judge
 */
function patternOfTheListingLine(): string {
  /**
   The line that reads the listing of the checkout.
   */
  const line = snapdirScript()
    .split('\n',)
    .find(function grepsTheListing(candidate,) {
      return candidate.includes('snapdir-list.log',) && candidate.includes(PATTERN_OPENS_WITH,);
    },);
  if (line === undefined)
    throw new Error('the snapdir script holds no line that greps the listing of the checkout',);
  /**
   Where the pattern begins and where its closing quote stands.
   */
  const start = line.indexOf(PATTERN_OPENS_WITH,) + PATTERN_OPENS_WITH.length;
  /**
   The offset of the quote that closes the pattern, absent when the line closes none.
   */
  const end = line.indexOf(
    '"',
    start,
  );
  if (end < 0)
    throw new Error(`the line that greps the listing closes no pattern: ${line}`,);
  return line.slice(
    start,
    end,
  );
}

/**
 Whether a real `grep` started with one pattern finds a line in one listing.

 @param pattern - the pattern the arm hands the guest's `grep`

 @param listing - the lines the listing holds, one per line

 @returns true when `grep --quiet` exits zero on those lines

 @throws Error when `grep` could not be started at all, which is neither a match nor a miss
 */
function grepFinds({
  pattern,
  listing,
}: {
  readonly pattern: string;
  readonly listing: string;
},): boolean {
  /**
   The `grep` run, whose status is the answer.
   */
  const read = spawnSync(
    'grep',
    [
      '--quiet',
      pattern,
    ],
    { input: listing, },
  );
  if (read.error !== undefined)
    throw read.error;
  return read.status === 0;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'patternOfTheListingLine',
      children: [
        it({
          name: 'finds the .zfs directory in the listing of the checkout',
          fn: async () => {
            expect(grepFinds({
              pattern: patternOfTheListingLine(),
              listing: [
                '.',
                '..',
                '.zfs',
                'xzfs',
                '.zfsx',
              ].join('\n',),
            },),).toBe(true,);
          },
        },),
        ...OTHER_NAMES.map(function mapOtherName(name,) {
          return it({
            name: `does not find ${name} in a listing that holds it and nothing else`,
            fn: async () => {
              expect(grepFinds({
                pattern: patternOfTheListingLine(),
                listing: `${name}\n`,
              },),).toBe(false,);
            },
          },);
        },),
      ],
    },),
  ],
},);
