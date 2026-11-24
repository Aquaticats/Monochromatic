/**
 Tests for the collapsed-target reader.
 
 Every shape the owner named gets its own case,
 and every literal path that stays allowed gets one too.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  collapsedTargetReason,
  isEmptySegmentName,
  isTopLevelFolder,
  NOT_COLLAPSED,
  withoutTrailingSeparators,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Home folder every case is judged against, so no case depends on this account.
 */
const FIXTURE_HOME = '/home/user';

/**
 Targets that collapsed to a broader path, with the reading that fires.
 */
const COLLAPSED_TARGETS: readonly { readonly value: string; readonly reason: string; }[] = [
  { value: '/', reason: 'the filesystem root', },
  { value: '//', reason: 'a doubled separator', },
  { value: '/home', reason: 'a top-level folder', },
  { value: '/usr', reason: 'a top-level folder', },
  { value: '/home/', reason: 'a top-level folder', },
  { value: '/home/user', reason: 'the account home folder', },
  { value: '/home/user/', reason: 'the account home folder', },
  { value: '//home/user/x', reason: 'a doubled separator', },
  { value: '/home/user//x', reason: 'a doubled separator', },
  { value: '/home/user/undefined/x', reason: 'a segment named undefined or null', },
  { value: '/home/user/null', reason: 'a segment named undefined or null', },
  { value: 'undefined', reason: 'a segment named undefined or null', },
  { value: 'null/x', reason: 'a segment named undefined or null', },
];

/**
 Targets that name one readable path.
 */
const READABLE_TARGETS: readonly string[] = [
  '/home/user/temp/agent/x',
  '/home/user/temp/agent/x/',
  '/home/userx',
  '/home/user2/x',
  '/usr/local/bin/x',
  './build.log',
  'build/*',
  '~/temp/agent/scratch',
  'undefined.txt',
  'nullable/x',
  String.raw`file\ name.txt`,
  '-',
  '',
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: withoutTrailingSeparators.name,
      children: [
        it({
          name: 'drops the separators after the last named segment',
          fn: async function testDropsSeparators() {
            expect(withoutTrailingSeparators('/home/user/',),).toBe('/home/user',);
          },
        },),
        it({
          name: 'keeps a bare root',
          fn: async function testKeepsRoot() {
            expect(withoutTrailingSeparators('/',),).toBe('/',);
          },
        },),
        it({
          name: 'keeps a path that ends in a named segment',
          fn: async function testKeepsNamedEnd() {
            expect(withoutTrailingSeparators('/home/user',),).toBe('/home/user',);
          },
        },),
      ],
    },),
    describe({
      name: isTopLevelFolder.name,
      children: [
        it({
          name: 'reads one segment under the root',
          fn: async function testOneSegment() {
            expect(isTopLevelFolder('/home',),).toBe(true,);
          },
        },),
        it({
          name: 'reads a deeper folder as no top-level folder',
          fn: async function testDeeperFolder() {
            expect(isTopLevelFolder('/home/user',),).toBe(false,);
          },
        },),
        it({
          name: 'reads a relative path as no top-level folder',
          fn: async function testRelativePath() {
            expect(isTopLevelFolder('home',),).toBe(false,);
          },
        },),
        it({
          name: 'reads the bare root as no top-level folder',
          fn: async function testRoot() {
            expect(isTopLevelFolder('/',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isEmptySegmentName.name,
      children: [
        it({
          name: 'reads the two values a shell writes for an undefined one',
          fn: async function testEmptyNames() {
            expect(isEmptySegmentName('undefined',),).toBe(true,);
            expect(isEmptySegmentName('null',),).toBe(true,);
          },
        },),
        it({
          name: 'reads a name that only starts the same way as a real one',
          fn: async function testSimilarNames() {
            expect(isEmptySegmentName('undefined.txt',),).toBe(false,);
            expect(isEmptySegmentName('nullable',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: collapsedTargetReason.name,
      children: [
        //region Refused shapes
        describe({
          name: 'a target that collapsed',
          children: COLLAPSED_TARGETS
            .map(function collapsedTarget(fixture,) {
              return it({
                name: `reads \`${fixture.value}\` as ${fixture.reason}`,
                fn: async function testCollapsedTarget() {
                  expect(
                    collapsedTargetReason({
                      value: fixture.value,
                      homeFolder: FIXTURE_HOME,
                    },),
                  ).toBe(fixture.reason,);
                },
              },);
            },),
        },),
        //endregion Refused shapes

        //region Readable shapes
        describe({
          name: 'a target that names one path',
          children: READABLE_TARGETS
            .map(function readableTarget(value,) {
              return it({
                name: `reads \`${value}\` as one readable path`,
                fn: async function testReadableTarget() {
                  expect(
                    collapsedTargetReason({
                      value,
                      homeFolder: FIXTURE_HOME,
                    },),
                  ).toBe(NOT_COLLAPSED,);
                },
              },);
            },),
        },),
        //endregion Readable shapes

        //region The injected home
        describe({
          name: 'the home folder it is given',
          children: [
            it({
              name: 'refuses the folder the caller names as home',
              fn: async function testInjectedHome() {
                expect(
                  collapsedTargetReason({
                    value: '/Users/someone',
                    homeFolder: '/Users/someone',
                  },),
                ).toBe('the account home folder',);
              },
            },),
            it({
              name: 'reads another account folder as a path below the root',
              fn: async function testAnotherAccount() {
                expect(
                  collapsedTargetReason({
                    value: '/home/other',
                    homeFolder: FIXTURE_HOME,
                  },),
                ).toBe(NOT_COLLAPSED,);
              },
            },),
            it({
              name: 'reads an empty home folder as no home folder at all',
              fn: async function testEmptyHome() {
                expect(
                  collapsedTargetReason({
                    value: '/home/user/temp/agent/x',
                    homeFolder: '',
                  },),
                ).toBe(NOT_COLLAPSED,);
              },
            },),
          ],
        },),
        //endregion The injected home
      ],
    },),
  ],
},);
