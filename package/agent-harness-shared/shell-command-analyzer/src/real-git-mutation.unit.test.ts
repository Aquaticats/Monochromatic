import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  invokesMutatingGitThroughRealGitPath,
  REAL_GIT_MUTATION_BAN_REASON,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Absolute real-Git spelling used by string fixtures,
 so the predicate is exercised without touching this host's installation.
 */
const FIXTURE_REAL_GIT_PATH = '/usr/bin/git';

/**
 Executable name that marks a path word as the real Git binary.
 */
const FIXTURE_REAL_GIT_BASENAME = 'git';

/**
 Injected executable names that keep fixture commands independent of the host.
 */
const FIXTURE_OPTIONS = { realGitBasenames: [FIXTURE_REAL_GIT_BASENAME,], };

/**
 Repository root passed to the `-C` remedy in fixture commands.
 */
const FIXTURE_REPOSITORY_ROOT = '/repo';

/**
 Reads every read-only subcommand that must stay allowed through an absolute real-Git path.
 */
const READ_ONLY_SUBCOMMANDS = [
  'for-each-ref',
  'rev-parse',
  'log',
  'show',
  'status',
  'ls-remote',
  'diff',
  'cat-file',
  'ls-files',
  'merge-base',
] as const;

//endregion Fixtures

await describe({
  name: 'real-Git mutation guard helpers',
  children: [
    describe({
      name: invokesMutatingGitThroughRealGitPath.name,
      children: [
        describe({
          name: 'mutations through an absolute real-Git path',
          children: [
            it({
              name: 'denies a direct commit',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a quoted real-Git path',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `"${FIXTURE_REAL_GIT_PATH}" commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a commit behind an env assignment prefix',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `env X=1 ${FIXTURE_REAL_GIT_PATH} commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a reset behind sudo',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `sudo ${FIXTURE_REAL_GIT_PATH} reset --hard`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a reset behind a sudo option with a value',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `sudo -u root ${FIXTURE_REAL_GIT_PATH} reset --hard`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies an add chained after another command',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `cd x && ${FIXTURE_REAL_GIT_PATH} add .`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a commit inside a shell -c script string',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `bash -c '${FIXTURE_REAL_GIT_PATH} commit -m x'`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a commit inside a shell flag cluster',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `bash -lc "${FIXTURE_REAL_GIT_PATH} commit -m x"`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a commit inside a shell started by env',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `env X=1 bash -c '${FIXTURE_REAL_GIT_PATH} commit -m x'`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a commit inside a function body',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `f(){ ${FIXTURE_REAL_GIT_PATH} push; }; f`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a mutation after Git global options',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} -C ${FIXTURE_REPOSITORY_ROOT} commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a mutation after a config override',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} -c user.name=x commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies an unqualified stash save',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} stash`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies branch creation',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} branch new-branch`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies branch deletion',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} branch -D old-branch`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies tag creation',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} tag v1.0.0`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
            it({
              name: 'denies a subcommand whose effect is unknown',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} frobnicate`,
                  FIXTURE_OPTIONS,
                ),).toBe(true,);
              },
            },),
          ],
        },),
        describe({
          name: 'read-only probes through an absolute real-Git path',
          children: [
            ...READ_ONLY_SUBCOMMANDS.map(function readOnlySubcommandCase(subcommand,) {
              return it({
                name: `allows ${subcommand}`,
                fn: async ({ expect: e, },) => {
                  e(await invokesMutatingGitThroughRealGitPath(
                    `${FIXTURE_REAL_GIT_PATH} ${subcommand}`,
                    FIXTURE_OPTIONS,
                  ),).toBe(false,);
                },
              },);
            },),
            it({
              name: 'allows version output without a subcommand',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} --version`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a read-only probe after Git global options',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} -C ${FIXTURE_REPOSITORY_ROOT} status --short`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows branch listing',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} branch --list`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows branch listing filtered by a contained commit',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} branch --contains HEAD`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows tag listing',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} tag --list 'v*'`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows stash listing',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} stash list`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows remote listing',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} remote -v`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a config read',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} config --get user.name`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a worktree listing',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `${FIXTURE_REAL_GIT_PATH} worktree list`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
          ],
        },),
        describe({
          name: 'commands that are not a real-Git path invocation',
          children: [
            it({
              name: 'allows the wrapper selected from PATH',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `git commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows the wrapper with the root passed through -C',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `git -C ${FIXTURE_REPOSITORY_ROOT} commit -m x`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a document path that only contains the directory word git',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  'cat /usr/share/doc/git/RelNotes',
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a script stored in a directory named git',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  'bash /home/user/git/check-notes.sh',
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a differently named binary in the same directory',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  '/usr/bin/gitk --version',
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows the real-Git path quoted inside prose',
              fn: async ({ expect: e, },) => {
                e(await invokesMutatingGitThroughRealGitPath(
                  `echo "${FIXTURE_REAL_GIT_PATH} commit -m x"`,
                  FIXTURE_OPTIONS,
                ),).toBe(false,);
              },
            },),
            it({
              name: 'allows a read-only probe through the resolver executable',
              fn: async ({ expect: e, },) => {
                /**
                 Real Git this host exposes, quoted so spaced install paths stay one word.
                 */
                const realGit = JSON.stringify(await resolveRealGit(),);
                e(await invokesMutatingGitThroughRealGitPath(`${realGit} rev-parse HEAD`,),)
                  .toBe(false,);
              },
            },),
          ],
        },),
        describe({
          name: 'resolver-derived executable names',
          children: [
            it({
              name: 'denies a mutation through the executable the resolver returns',
              fn: async ({ expect: e, },) => {
                /**
                 Real Git this host exposes, quoted so spaced install paths stay one word.
                 */
                const realGit = JSON.stringify(await resolveRealGit(),);
                e(await invokesMutatingGitThroughRealGitPath(`${realGit} commit -m x`,),)
                  .toBe(true,);
              },
            },),
          ],
        },),
      ],
    },),
    describe({
      name: 'REAL_GIT_MUTATION_BAN_REASON',
      children: [
        it({
          name: 'names the wrapper invocation to run instead',
          fn: async ({ expect: e, },) => {
            e(REAL_GIT_MUTATION_BAN_REASON.includes('git -C <repository-root>',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'names the read-only probes that stay allowed',
          fn: async ({ expect: e, },) => {
            e(REAL_GIT_MUTATION_BAN_REASON.includes('rev-parse',),).toBe(true,);
            e(REAL_GIT_MUTATION_BAN_REASON.includes('for-each-ref',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
