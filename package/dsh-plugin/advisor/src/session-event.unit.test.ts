/**
 Unit tests for the Advisor session-event accounting sink.
 */

import {
  pluginRecordOf,
  Session,
  SessionId,
} from '@deepseek-ai/dsh-session';
import { validateStoredEvents, } from '@deepseek-ai/dsh-session-persistence';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { appendAdvisorOperationRecord, } from '../dist/final/node/index.mjs';

/**
 Argument shape of the session-event accounting sink.
 */
type AppendRecordOptions = Parameters<typeof appendAdvisorOperationRecord>[0];

await describe({
  name: appendAdvisorOperationRecord.name,
  children: [
    it({
      name: 'appends the operation as an ignorable plugin record every reader may skip',
      fn: async () => {
        const session = Session
          .create(
          SessionId('session-advisor-operation-record-test'),
          );
        const operation = {
          id: 'op-1',
        } as unknown as AppendRecordOptions['operation'];
        appendAdvisorOperationRecord({
          session,
          operation,
        },);
        const events = session
          // oxlint-disable-next-line typescript/no-deprecated -- the upstream deprecation note allows test files to inspect emitted events through the three Session readers; committed plugin records have no other accessor
          .snapshotEvents();

        let caught: unknown;
        try {
          validateStoredEvents(
            session.header,
            [...events,],
          );
        } catch (error) {
          caught = error;
        }
        expect(caught,).toBeUndefined();

        expect(events,).toHaveLength(1,);
        const [event] = events;
        expect(event
          ?.type,).toBe('plugin:advisor/operation',);
        expect(event
          ?.ignorable,).toBe(true,);

        const record = event === undefined
          ? undefined
          : pluginRecordOf(event,);
        expect(record
          ?.type,).toBe('plugin:advisor/operation',);
        expect(record
          ?.data,).toEqual(operation,);
      },
    },),
  ],
},);
