/**
 The whole daily update path once, over a disposable repository with a recording stand-in for `gh`: baseline checks,
 the update's commits, the second checks, and the one issue for the crate the update broke.

 No real issue can be opened: the repository has no remote and the stand-in `gh` is first on `PATH`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type RecordedGhCall,
  runWholePath,
} from './daily-update-whole-path-fixture.ts';

/**
 Calls of one `gh` verb pair, such as `issue create`.

 @param calls - every call recorded

 @param verb - the first two arguments joined by a space

 @returns the matching calls
 */
function callsOf({
  calls,
  verb,
}: {
  readonly calls: readonly RecordedGhCall[];
  readonly verb: string;
},): readonly RecordedGhCall[] {
  return calls.filter(function matches(call,) {
    return call.args.slice(0, 2,)
      .join(' ',) === verb;
  },);
}

await describe({
  name: 'the whole daily update path with a stand-in gh',
  children: [
    it({
      name: 'opens one issue for the crate the update broke, none for the crate that was already red, and keeps both commits',
      fn: async () => {
        const result = await runWholePath({ ghExitCode: 0, },);
        expect(result.exitCode,).toBe(0,);
        expect(result.commitSubjects.slice(0, 2,),).toEqual([
          'chore(*): daily update of the pnpm and Cargo dependency locks',
          'chore(toolchain): daily update of the repository toolchain lock',
        ],);
        expect(result.commitSubjects.length,).toBe(3,);
        const creates = callsOf({ calls: result.ghCalls, verb: 'issue create', },);
        expect(creates.length,).toBe(1,);
        const [create,] = creates;
        expect(create?.args,).toContain('needs-triage',);
        expect(create?.args.join(' ',),).toContain('package/a/broken-crate',);
        expect(create?.stdin,).toContain('package/a/broken-crate',);
        expect(create?.stdin,).not.toContain('package/c/red-crate',);
        expect(create?.stdin,).toContain('`zbus` 5.19.0 to 6.2.1',);
        expect(create?.stdin,).toContain('`rust` nightly-1 to nightly-2',);
        expect(create?.stdin,).toContain('error[E0433]',);
        expect(create?.stdin,).toContain('mise run //package/a/broken-crate:lint:clippy',);
        expect(create?.stdin,).toContain('toolchain-daily-update package=package/a/broken-crate task=lint:clippy cause=E0433',);
        expect(result.pendingText,).toBe('',);
        expect(result.notifications.length,).toBe(1,);
        expect(result.notifications[0]?.bus.endsWith('no-session-bus',),).toBe(true,);
        expect(result.notifications[0]?.args.join(' ',),).toContain('issue opened',);
        for (const call of result.ghCalls) {
          expect(call.environment.GIT_ASKPASS,).toBe('',);
          expect(call.environment.GIT_TERMINAL_PROMPT,).toBe('0',);
          expect(call.environment.GH_PROMPT_DISABLED,).toBe('1',);
        }
      },
    },),
    it({
      name: 'keeps the issue for the next run when gh fails, and still exits as the update did',
      fn: async () => {
        const result = await runWholePath({ ghExitCode: 1, },);
        expect(result.exitCode,).toBe(0,);
        expect(result.commitSubjects.length,).toBe(3,);
        expect(callsOf({ calls: result.ghCalls, verb: 'issue create', },).length,).toBe(0,);
        expect(result.pendingText,).toContain('package/a/broken-crate',);
        expect(result.journal,).toContain('error connecting to api.github.com',);
        expect(result.notifications.length,).toBe(1,);
      },
    },),
  ],
},);
