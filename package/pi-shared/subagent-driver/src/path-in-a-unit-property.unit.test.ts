/**
 One path written the way systemd reads it in a `ReadOnlyPaths=` or `ReadWritePaths=` property: quoted, with only the backslash and
 the double quote escaped, and with the dash prefix for a path that may be absent. A path systemd cannot take is refused here,
 naming the path.

 The encodings this test holds are the ones measured against systemd 262 on this machine, where a value of `--property=` is read
 through the same quoting and escape rules as a unit-file assignment.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  PathThatIsNotAbsoluteError,
  pathInAUnitProperty,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: pathInAUnitProperty.name,
      children: [
        it({
          name: 'quotes a plain path, so that nothing in it is read as the end of an item',
          fn: async () => {
            expect(pathInAUnitProperty({ path: '/repo', },),).toBe('"/repo"',);
          },
        },),
        it({
          name: 'keeps a space, a single quote and a line break inside one item, which systemd then reads as one path',
          fn: async () => {
            expect(pathInAUnitProperty({ path: '/data/my repo', },),).toBe('"/data/my repo"',);
            expect(pathInAUnitProperty({ path: "/data/a'b", },),).toBe(`"/data/a'b"`,);
            expect(pathInAUnitProperty({ path: '/data/a\nb', },),).toBe('"/data/a\nb"',);
          },
        },),
        it({
          name: 'escapes the backslash and the double quote the quoting itself reads, so the path comes through as text',
          fn: async () => {
            expect(pathInAUnitProperty({ path: String.raw`/data/a"b`, },),).toBe(String.raw`"/data/a\"b"`,);
            expect(pathInAUnitProperty({ path: String.raw`/data/a\b`, },),).toBe(String.raw`"/data/a\\b"`,);
            expect(pathInAUnitProperty({ path: String.raw`/data/a\"b`, },),).toBe(String.raw`"/data/a\\\"b"`,);
          },
        },),
        it({
          name: 'leaves a percent sign, a colon and a dash inside a name alone, which systemd reads as they are',
          fn: async () => {
            expect(pathInAUnitProperty({ path: '/data/a%b', },),).toBe('"/data/a%b"',);
            expect(pathInAUnitProperty({ path: '/data/a%h', },),).toBe('"/data/a%h"',);
            expect(pathInAUnitProperty({ path: '/data/a:b', },),).toBe('"/data/a:b"',);
            expect(pathInAUnitProperty({ path: '/data/-b', },),).toBe('"/data/-b"',);
          },
        },),
        it({
          name: 'puts the dash that ignores a path which is not there outside the quotes, where systemd reads it as its prefix',
          fn: async () => {
            expect(pathInAUnitProperty({
              path: '/data/my repo/.cache',
              ignoredWhenMissing: true,
            },),).toBe('-"/data/my repo/.cache"',);
            expect(pathInAUnitProperty({
              path: '/repo/.cache',
              ignoredWhenMissing: true,
            },),).toBe('-"/repo/.cache"',);
          },
        },),
        it({
          name: 'refuses a path systemd cannot take, naming the path and what to write instead',
          fn: async () => {
            for (const path of ['sub/repo', './repo', '', '-sub/repo',]) {
              try {
                pathInAUnitProperty({ path, },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(PathThatIsNotAbsoluteError,);
                expect(String(error,),).toContain(`'${path}'`,);
                expect(String(error,),).toContain('not absolute',);
                expect(String(error,),).toContain('Invalid ReadOnlyPaths',);
                continue;
              }
              throw new Error(`a path systemd cannot take, ${JSON.stringify(path,)}, was written as an item`,);
            }
          },
        },),
      ],
    },),
  ],
},);
