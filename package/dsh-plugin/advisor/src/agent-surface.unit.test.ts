/**
 Unit tests for the live-agent projection onto the review surface.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { Agent, } from '@deepseek-ai/dsh-agent';

import {
  Session,
  SessionId,
} from '@deepseek-ai/dsh-session';

import type { WorkingDirectoryService, } from '@deepseek-ai/dsh-working-directory';

import { toDshAdvisorAgentSurface, } from '../dist/final/node/index.mjs';

/**
 Options shape of the agent projection.
 */
type ProjectOptions = Parameters<typeof toDshAdvisorAgentSurface>[0];

await describe({
  name: toDshAdvisorAgentSurface.name,
  children: [
    it({
      name: 'projects session, cwd, and route identity onto the surface',
      fn: async () => {
        const session = Session
          .create(
          SessionId('session-agent-surface-append-test'),
          );
        const surface = toDshAdvisorAgentSurface({
          agent: {
            id: SessionId('session-agent-surface-a',),
            session,
            options: {
              provider: 'route',
              model: 'model-id',
            },
          } as unknown as Agent,
          workingDirectory: {
            get: (): string => '/work/dir',
          } as unknown as ProjectOptions['workingDirectory'],
        },);
        expect(surface.sessionId,).toBe('session-agent-surface-a',);
        expect(surface.cwd,).toBe('/work/dir',);
        expect(surface.options,).toEqual({
          provider: 'route',
          model: 'model-id',
        },);
        surface.appendOperationRecord({
          id: 'op-1',
        } as unknown as Parameters<typeof surface.appendOperationRecord>[0],);
        expect(session.seq,).toBe(1,);
      },
    },),
    it({
      name: 'omits absent route identity from the surface options',
      fn: async () => {
        const surface = toDshAdvisorAgentSurface({
          agent: {
            id: SessionId('session-agent-surface-b',),
            session: {
              deriveMessages: () => [],
            },
            options: {},
            whenIdle: () => Promise.resolve(),
          } as unknown as Agent,
          workingDirectory: {
            get: (): string => '/work/dir',
          } as unknown as ProjectOptions['workingDirectory'],
        },);
        expect(surface.options,).toEqual({},);
        await surface.whenIdle();
      },
    },),
    it({
      name: 'carries the agent identity and signal in the assembly context',
      fn: async () => {
        const agent = {
          id: SessionId('session-agent-surface-c',),
          session: {
            deriveMessages: () => [],
          },
          options: {
            provider: 'route',
            model: 'model-id',
          },
        } as unknown as Agent;
        const surface = toDshAdvisorAgentSurface({
          agent,
          workingDirectory: {
            get: (): string => '/work/dir',
          } as unknown as ProjectOptions['workingDirectory'],
        },);
        const controller = new AbortController();
        const context = surface
          .assembleContext(controller.signal,);
        expect(context.agent,).toBe(agent,);
        expect(context.scope,).toBe(agent,);
        expect(context.signal,).toBe(controller.signal,);
        const withoutSignal = surface
          .assembleContext();
        expect(withoutSignal.signal,).toBe(undefined,);
      },
    },),
  ],
},);
