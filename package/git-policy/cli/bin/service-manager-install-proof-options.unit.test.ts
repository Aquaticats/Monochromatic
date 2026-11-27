import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeVerificationError, } from './native-verification-process.ts';
import { options, } from './service-manager-install-proof-options.ts';

/**
 The two options every run of this proof takes, as the tasks that start it give them.

 @returns the arguments after the program's name
 */
function everyOption(): readonly string[] {
  return [
    '--image',
    'localhost/git-policy-native-test:example',
    '--older-wrapper',
    '/opt/git-0.6.1-alpha.20',
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: options.name,
      children: [
        it({
          name: 'read the image and the older release from the arguments every run takes',
          fn: async () => {
            expect(options({ argv: everyOption(), },),).toEqual({
              image: 'localhost/git-policy-native-test:example',
              olderWrapper: '/opt/git-0.6.1-alpha.20',
            },);
          },
        }),
        it({
          name: 'read the release being installed when the caller names one',
          fn: async () => {
            expect(options({
              argv: [
                ...everyOption(),
                '--candidate-wrapper',
                '/opt/git-0.6.1-alpha.21',
              ],
            },),).toEqual({
              image: 'localhost/git-policy-native-test:example',
              olderWrapper: '/opt/git-0.6.1-alpha.20',
              candidateWrapper: '/opt/git-0.6.1-alpha.21',
            },);
          },
        }),
        it({
          name: "read no release being installed when the caller names none, so the run installs this worktree's build",
          fn: async () => {
            expect(options({ argv: everyOption(), },).candidateWrapper,).toBeUndefined();
          },
        }),
        it({
          name: 'keep the later value of an option a caller named twice',
          fn: async () => {
            expect(options({
              argv: [
                ...everyOption(),
                '--older-wrapper',
                '/opt/git-0.6.1-alpha.21',
              ],
            },).olderWrapper,).toBe('/opt/git-0.6.1-alpha.21',);
          },
        }),
        it({
          name: 'read an empty value for an option a caller gave an empty word',
          fn: async () => {
            expect(options({
              argv: [
                '--image',
                '',
                '--older-wrapper',
                '/opt/git-0.6.1-alpha.20',
              ],
            },),).toEqual({
              image: '',
              olderWrapper: '/opt/git-0.6.1-alpha.20',
            },);
          },
        }),
        ...[[], ['--image', 'localhost/x',], ['--older-wrapper', '/opt/git',], ['--image',],].map(function onlyHalfTheOptions(
          argv,
        ) {
          return it({
            name: `refuse an argument list without both options: ${argv.join(' ',)}`,
            fn: async () => {
              /**
               What reading this argument list threw.
               */
              let caught: unknown;
              try {
                options({ argv, },);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(NativeVerificationError,);
              expect((caught as Error).message,).toBe('Usage: node bin/service-manager-install-proof.ts --image <tag> --older-wrapper <path> [--candidate-wrapper <path>]',);
            },
          },);
        },),
      ],
    },),
  ],
},);
