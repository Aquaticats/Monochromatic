import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';

import { buildCommand, } from '../dist/final/node/launch.mjs';

//region Configured terminal arguments

await describe({
  name: buildCommand.name,
  children: ['--gtk-single-instance=false', '--gtk-single-instance=true', '--gtk-single-instance',
    '--gtk-single-instance-extra=kept', '--unrelated=value with spaces',].map(function configuredFlag(flag,) {
      return it({
        name: `preserves configured Exec token ${flag}`,
        fn: async () => {
          expect(buildCommand({
            terminal: {
              entryId: 'fixture.desktop',
              execTokens: ['/fixture/terminal', flag,],
              execArg: '-e',
              appIdArg: '',
              titleArg: '--title=',
              dirArg: '--working-directory=',
              holdArg: '',
            },
            options: {
              appId: '',
              title: 'Question',
              dir: '/fixture/with spaces',
              hold: false,
              command: ['node', '--eval', 'a; b',],
            },
          },),).toEqual([
            '/fixture/terminal', flag, '--title=Question', '--working-directory=/fixture/with spaces',
            '-e', 'node', '--eval', 'a; b',
          ],);
        },
      },);
    },),
},);

//endregion Configured terminal arguments
