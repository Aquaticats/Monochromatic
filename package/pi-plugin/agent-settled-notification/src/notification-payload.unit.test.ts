/**
 Tests for the notification payload a settled agent sends.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NOTIFICATION_APP_NAME,
  NOTIFICATION_BODY_WITHOUT_FOLDER,
  NOTIFICATION_COMMAND,
  NOTIFICATION_TITLE,
  notificationArguments,
  notificationBody,
  notificationInvocation,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Working folder of one agent, distinct from the folder the next case uses.
 */
const AGENT_FOLDER = '/var/mnt/encrypted/repository/example/.claude/worktrees/agent-7';

/**
 Working folder of a second agent, whose last segment differs from the first.
 */
const OTHER_AGENT_FOLDER = '/var/mnt/encrypted/repository/example/.claude/worktrees/agent-8';

/**
 Folder holding shell syntax that would split or expand if the payload were one argument.
 */
const HOSTILE_FOLDER = '/work/agent with space; $(reboot)';

/**
 Folder Pi reports as whitespace only, which must read as absent.
 */
const BLANK_FOLDER = '   ';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: notificationBody.name,
      children: [
        it({
          name: 'names the working folder of the settled agent',
          fn: async () => {
            expect(notificationBody({ folder: AGENT_FOLDER, },),).toBe(`Idle in ${AGENT_FOLDER}`,);
          },
        },),

        it({
          name: 'tells two agents in different folders apart',
          fn: async () => {
            expect(notificationBody({ folder: AGENT_FOLDER, },),).not.toBe(
              notificationBody({ folder: OTHER_AGENT_FOLDER, },),
            );
          },
        },),
      ],
    },),

    describe({
      name: notificationArguments.name,
      children: [
        it({
          name: 'carries the application name, the shared title and the body',
          fn: async () => {
            /**
             Body this case sends.
             */
            const body = `Idle in ${AGENT_FOLDER}`;

            expect(notificationArguments({ body, },),).toEqual([
              `--app-name=${NOTIFICATION_APP_NAME}`,
              NOTIFICATION_TITLE,
              body,
            ],);
          },
        },),

        it({
          name: 'keeps every value in its own argument, outside shell syntax',
          fn: async () => {
            /**
             Arguments built for a folder holding shell syntax.
             */
            const { args, } = notificationInvocation({ folder: HOSTILE_FOLDER, },);

            expect(args,).toHaveLength(3,);
            expect(args.at(-1,),).toBe(`Idle in ${HOSTILE_FOLDER}`,);
          },
        },),
      ],
    },),

    describe({
      name: notificationInvocation.name,
      children: [
        it({
          name: 'names the desktop executable and the agent folder',
          fn: async () => {
            expect(notificationInvocation({ folder: AGENT_FOLDER, },),).toEqual({
              command: NOTIFICATION_COMMAND,
              args: [
                `--app-name=${NOTIFICATION_APP_NAME}`,
                NOTIFICATION_TITLE,
                `Idle in ${AGENT_FOLDER}`,
              ],
            },);
          },
        },),

        it({
          name: 'invokes the executable that implements the Freedesktop boundary',
          fn: async () => {
            expect(NOTIFICATION_COMMAND,).toBe('notify-send',);
          },
        },),

        it({
          name: 'surrounding whitespace never reaches the body',
          fn: async () => {
            expect(notificationInvocation({ folder: `  ${AGENT_FOLDER}\n`, },),).toEqual(
              notificationInvocation({ folder: AGENT_FOLDER, },),
            );
          },
        },),

        it({
          name: 'falls back to the folderless body when no folder is reported',
          fn: async () => {
            expect(notificationInvocation({},).args.at(-1,),).toBe(NOTIFICATION_BODY_WITHOUT_FOLDER,);
          },
        },),

        it({
          name: 'falls back to the folderless body when the folder is blank',
          fn: async () => {
            expect(notificationInvocation({ folder: BLANK_FOLDER, },).args.at(-1,),).toBe(
              NOTIFICATION_BODY_WITHOUT_FOLDER,
            );
          },
        },),
      ],
    },),
  ],
},);
