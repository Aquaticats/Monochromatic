import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readonlyTypeOriginEvidenceFromResolution,
  type ReadonlyTypeOriginResolution,
} from '../dist/final/node/index.mjs';

/**
 One origin accepted by pure resolution classifier.
 */
type Origin = ReadonlyTypeOriginResolution['origins'][number];

/**
 Builds callable origin for branch controls.
 
 @param identity - Full source identity retained for deduplication.
 
 @param name - Reader-facing callable name.
 
 @returns callable origin at shared display line.
 
 @example
 ```ts
 callableOrigin({ identity: '/repo/a.ts:1', name: 'toA' });
 ```
 */
function callableOrigin({
  identity,
  name,
}: {
  readonly identity: string;
  readonly name: string;
},): Origin {
  return {
    identity,
    kind: 'callable',
    name,
    location: 'src/shared.ts:1',
  };
}

await describe({
  name: readonlyTypeOriginEvidenceFromResolution.name,
  children: [
    it({
      name: 'keeps authored syntax ahead of origin resolution',
      fn: async () => {
        expect(readonlyTypeOriginEvidenceFromResolution({
          authored: true,
          resolution: {
            origins: [],
            resolutionIncomplete: true,
          },
        },),).toEqual({ kind: 'authored', },);
      },
    },),
    it({
      name: 'reports no origin when complete resolution finds none',
      fn: async () => {
        expect(readonlyTypeOriginEvidenceFromResolution({
          authored: false,
          resolution: {
            origins: [],
            resolutionIncomplete: false,
          },
        },),).toEqual({ kind: 'none', },);
      },
    },),
    it({
      name: 'withholds unique advice when any declaration resolution is incomplete',
      fn: async () => {
        expect(readonlyTypeOriginEvidenceFromResolution({
          authored: false,
          resolution: {
            origins: [callableOrigin({
              identity: '/repo/src/shared.ts:1',
              name: 'resolved',
            },),],
            resolutionIncomplete: true,
          },
        },),).toEqual({ kind: 'uncertain', },);
      },
    },),
    it({
      name: 'returns sole completely resolved origin',
      fn: async () => {
        /**
         Sole origin expected by exact identity.
         */
        const origin = callableOrigin({
          identity: '/repo/src/shared.ts:1',
          name: 'only',
        },);
        expect(readonlyTypeOriginEvidenceFromResolution({
          authored: false,
          resolution: {
            origins: [origin,],
            resolutionIncomplete: false,
          },
        },),).toEqual({
          kind: 'unique',
          origin,
        },);
      },
    },),
    it({
      name: 'keeps same-line distinct identities non-unique',
      fn: async () => {
        expect(readonlyTypeOriginEvidenceFromResolution({
          authored: false,
          resolution: {
            origins: [
              callableOrigin({
                identity: '/repo/src/shared.ts:1',
                name: 'left',
              },),
              callableOrigin({
                identity: '/repo/src/shared.ts:40',
                name: 'right',
              },),
            ],
            resolutionIncomplete: false,
          },
        },),).toEqual({ kind: 'multiple', },);
      },
    },),
  ],
},);
