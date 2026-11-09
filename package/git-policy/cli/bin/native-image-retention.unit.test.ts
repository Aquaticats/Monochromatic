import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  imageNames,
  imagesToRemove,
  isUnnamedImage,
} from './native-image-retention.ts';

/**
 Listing in the shape `podman images --format '{{.ID}} {{.Repository}}:{{.Tag}}'` prints it: one line per name, an
 image nothing names under the untagged marker, and a line that carries no name at all.
 */
const listing = [
  'a1b2c3d4e5f6 localhost/git-policy-native-test:development',
  'a1b2c3d4e5f6 localhost/git-policy-native-test:coordinator-tokio-062',
  '0f0f0f0f0f0f <none>:<none>',
  '999999999999 localhost/git-policy-native-test-base:9c2c03d16a57-97221c29742b',
  '',
  'no-name-here',
].join('\n',);

/**
 Names of the store, read once for the decisions below.
 */
const names = imageNames({ output: listing, },);

/**
 Names of the store after a build wrote the development tag over a different image.
 */
const afterReplacingDevelopment = imageNames({
  output: [
    'b2c3d4e5f6a1 localhost/git-policy-native-test:development',
    '0f0f0f0f0f0f <none>:<none>',
  ].join('\n',),
});

await describe({
  name: 'the image a gate build replaced',
  children: [
    describe({
      name: imageNames.name,
      children: [
        it({
          name: 'reads one name per line, keeps the untagged marker, and drops a line it cannot read',
          fn: async () => {
            expect(names.map(function describeOne(entry) {
              return `${entry.id} ${entry.repository}:${entry.tag}`;
            },),)
              .toEqual([
                'a1b2c3d4e5f6 localhost/git-policy-native-test:development',
                'a1b2c3d4e5f6 localhost/git-policy-native-test:coordinator-tokio-062',
                '0f0f0f0f0f0f <none>:<none>',
                '999999999999 localhost/git-policy-native-test-base:9c2c03d16a57-97221c29742b',
              ],);
          },
        }),
      ],
    }),
    describe({
      name: isUnnamedImage.name,
      children: [
        it({
          name: 'holds for an image only the untagged marker names',
          fn: async () => {
            expect(isUnnamedImage({
              names,
              id: '0f0f0f0f0f0f',
            },),)
              .toBe(true,);
          },
        }),
        it({
          name: 'fails while another tag still names the image, so a worktree\'s own image is never taken from it',
          fn: async () => {
            expect(isUnnamedImage({
              names,
              id: 'a1b2c3d4e5f6',
            },),)
              .toBe(false,);
          },
        }),
        it({
          name: 'fails for an ID the store no longer lists, so nothing is removed twice',
          fn: async () => {
            expect(isUnnamedImage({
              names,
              id: 'ffffffffffff',
            },),)
              .toBe(false,);
          },
        }),
      ],
    }),
    describe({
      name: imagesToRemove.name,
      children: [
        it({
          name: 'removes the image the build left untagged behind its tag',
          fn: async () => {
            expect(imagesToRemove({
              before: imageNames({
                output: '0f0f0f0f0f0f localhost/git-policy-native-test:development\n',
              },),
              after: afterReplacingDevelopment,
              name: 'localhost/git-policy-native-test:development',
            },),)
              .toEqual(['0f0f0f0f0f0f',],);
          },
        }),
        it({
          name: 'keeps an image another tag still names',
          fn: async () => {
            expect(imagesToRemove({
              before: imageNames({
                output: 'a1b2c3d4e5f6 localhost/git-policy-native-test:development\n',
              },),
              after: imageNames({
                output: [
                  'b2c3d4e5f6a1 localhost/git-policy-native-test:development',
                  'a1b2c3d4e5f6 localhost/git-policy-native-test:coordinator-tokio-062',
                ].join('\n',),
              },),
              name: 'localhost/git-policy-native-test:development',
            },),)
              .toEqual([],);
          },
        }),
        it({
          name: 'keeps the image when the build reused the one the tag already carried',
          fn: async () => {
            expect(imagesToRemove({
              before: imageNames({
                output: 'a1b2c3d4e5f6 localhost/git-policy-native-test:development\n',
              },),
              after: imageNames({
                output: 'a1b2c3d4e5f6 localhost/git-policy-native-test:development\n',
              },),
              name: 'localhost/git-policy-native-test:development',
            },),)
              .toEqual([],);
          },
        }),
        it({
          name: 'removes nothing when no image carried the tag before the build',
          fn: async () => {
            expect(imagesToRemove({
              before: [],
              after: afterReplacingDevelopment,
              name: 'localhost/git-policy-native-test:development',
            },),)
              .toEqual([],);
          },
        }),
      ],
    }),
  ],
});
