/**
 Unit tests for the directory-listing sort.
 
 Tests import from built `dist/app` so they verify the artifact the main
 process consumes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { BridgeFileEntry, } from '../dist/app/bridge-types.js';
import {
  compareBridgeEntries,
  sortBridgeEntries,
} from '../dist/app/listing-sort.js';

/**
 Shorthand fixture entry builder.
 */
function entry(
  {
    kind,
    name,
  }: {
    readonly kind: BridgeFileEntry['kind'];
    readonly name: string;
  },
): BridgeFileEntry {
  return {
    kind,
    name,
    path: `/fixture/${name}`,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: compareBridgeEntries.name,
      children: [
        it({
          name: 'groups directories before files',
          fn: async () => {
            expect(compareBridgeEntries({
              left: entry({
                kind: 'directory',
                name: 'zeta',
              },),
              right: entry({
                kind: 'file',
                name: 'alpha',
              },),
            },) < 0,).toBe(true,);
          },
        },),
        it({
          name: 'groups symlinks with files, not directories',
          fn: async () => {
            expect(compareBridgeEntries({
              left: entry({
                kind: 'symlink',
                name: 'alpha',
              },),
              right: entry({
                kind: 'directory',
                name: 'zeta',
              },),
            },) > 0,).toBe(true,);
          },
        },),
        it({
          name: 'orders names case-insensitively within a group',
          fn: async () => {
            expect(compareBridgeEntries({
              left: entry({
                kind: 'file',
                name: 'Beta',
              },),
              right: entry({
                kind: 'file',
                name: 'alpha',
              },),
            },) > 0,).toBe(true,);
          },
        },),
        it({
          name: 'treats case-folded equal names as equal',
          fn: async () => {
            expect(compareBridgeEntries({
              left: entry({
                kind: 'file',
                name: 'Same',
              },),
              right: entry({
                kind: 'file',
                name: 'same',
              },),
            },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: sortBridgeEntries.name,
      children: [
        it({
          name: 'sorts directories-first then case-insensitively by name',
          fn: async () => {
            const sorted = sortBridgeEntries({
              entries: [
                entry({
                  kind: 'file',
                  name: 'readme.txt',
                },),
                entry({
                  kind: 'directory',
                  name: 'beta',
                },),
                entry({
                  kind: 'symlink',
                  name: 'Link',
                },),
                entry({
                  kind: 'directory',
                  name: 'Alpha',
                },),
              ],
            },);
            expect(sorted.map(function nameOf(sortedEntry,): string {
              return sortedEntry.name;
            },)
              .join(',',),).toBe('Alpha,beta,Link,readme.txt',);
          },
        },),
      ],
    },),
  ],
},);
