/**
 Fork-only invariants: call-shape fidelity against an independent settlement
 model, and input totality for `pify`'s wrap boundary.
 
 @module
 */

import {
  anything,
  assert,
  asyncProperty,
  property,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pify, } from '@monochromatic-dev/upstream-fork-pify/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  flagSlotArb,
  type FunctionBehavior,
  type OptionsSpec,
  wrappedFunctionSpecArb,
} from './pify-arbitrary.ts';
import {
  buildOptions,
  buildWrappedFunction,
} from './spec-fixtures.ts';
import {
  describeSettlement,
  type Settlement,
} from './settlement.ts';

//region Model

/**
 Computes the settlement upstream `pify`'s documented semantics promise for
 one behavior under one options record: the option slots' presence (including
 explicit `undefined`) decides the effective flags exactly like upstream's
 spread merge, then the callback payload is interpreted by those flags.
 
 @param behavior - Callback payload or throw message the fixture delivers.
 
 @param optionsSpec - Option slots the caller supplies.
 
 @returns Settlement the implementation must produce.
 */
function expectedSettlement(
  {
    behavior,
    optionsSpec,
  }: {
    readonly behavior: FunctionBehavior;
    readonly optionsSpec: OptionsSpec;
  },
): Settlement {
  /**
   Effective flags: the supplied value when the key is present (explicit
   `undefined` included, which stays falsy), the upstream default otherwise.
   */
  /**
   Whether each flag slot is present and truthy: the presence test keeps the
   explicit-`undefined` quirk observable (`undefined` stays falsy while a
   supplied `true` wins over the default).
   */
  const multiArgsProvided = optionsSpec.multiArgs.provided;
  const multiArgs = multiArgsProvided ? Boolean(optionsSpec.multiArgs.value,) : false;
  /**
   Whether the error-first flag is present and truthy, defaulted to upstream's
   `true` when the key is absent.
   */
  const errorFirstProvided = optionsSpec.errorFirst.provided;
  const errorFirst = errorFirstProvided ? Boolean(optionsSpec.errorFirst.value,) : true;

  if (behavior.kind === 'throwSync')
    return {
      tag: 'rejected',
      value: {
        errorName: 'Error',
        message: String(behavior.payload[0],),
      },
    };

  if (multiArgs) {
    if (errorFirst) {
      /**
       Whether the callback reported a truthy leading error argument.
       */
      const errorReported = Boolean(behavior.payload[0],);
      if (errorReported)
        return {
          tag: 'rejected',
          value: behavior.payload,
        };
      return {
        tag: 'resolved',
        value: behavior.payload.slice(1,),
      };
    }
    return {
      tag: 'resolved',
      value: behavior.payload,
    };
  }

  if (errorFirst) {
    /**
     Whether the callback reported a truthy leading error argument.
     */
    const errorReported = Boolean(behavior.payload[0],);
    if (errorReported)
      return {
        tag: 'rejected',
        value: behavior.payload[0],
      };
    return {
      tag: 'resolved',
      value: behavior.payload[1],
    };
  }
  return {
    tag: 'resolved',
    value: behavior.payload[0],
  };
}

//endregion Model

await describe({
  name: 'pify invariants',
  children: [
    //region Call fidelity

    it({
      name: 'forwards the args tuple before the callback and settles like the model',
      fn: async () => {
        await assert(
          asyncProperty(
            wrappedFunctionSpecArb,
            flagSlotArb,
            flagSlotArb,
            string(),
            async function faithfulCall(functionSpec, multiArgsSlot, errorFirstSlot, callerArgument,) {
              /**
               Options record limited to the two settlement-relevant slots.
               */
              const optionsSpec: OptionsSpec = {
                multiArgs: multiArgsSlot,
                errorFirst: errorFirstSlot,
                excludeMain: {
                  provided: false,
                  value: undefined,
                },
                include: {
                  provided: false,
                  value: undefined,
                },
                exclude: {
                  provided: false,
                  value: undefined,
                },
              };
              /**
               Fresh fixture and the fork's view of it.
               */
              const built = buildWrappedFunction({
                spec: functionSpec,
              },);
              const pified = pify({
                input: built.fn as (...call: readonly unknown[]) => unknown,
                options: buildOptions({
                  spec: optionsSpec,
                },) as never,
              });
              /**
               Caller arguments matching the fixture's declared arity.
               */
              const args = Array.from({
                length: functionSpec.declaredArgumentCount,
              }, function byIndex(index: number,): string {
                return `${callerArgument}-${index}`;
              },);
              /**
               Settlement observed through the fork.
               */
              /**
               Fork view called through its `{ args }` tuple shape.
               */
              const callView = pified as unknown as (
                call: { readonly args: readonly unknown[]; },
              ) => Promise<unknown>;
              const settlement = await describeSettlement(callView({
                args,
              },),);
              if (functionSpec.behavior.kind === 'callback') {
                expect(JSON.stringify(settlement,),).toBe(
                  JSON.stringify(expectedSettlement({
                    behavior: functionSpec.behavior,
                    optionsSpec,
                  },),),
                );
              }
              else {
                expect(settlement.tag,).toBe('rejected',);
              }
              expect(built.receivedArguments,).toEqual([
                args,
              ],);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    //endregion Call fidelity

    //region Totality

    it({
      name: 'pify wraps any object or function and rejects anything else with one error',
      fn: async () => {
        await assert(
          property(
            anything(),
            function totalWrap(input: unknown,) {
              try {
                /**
                 Wrap result over the generated input.
                 */
                const view = pify({
                  input: input as object,
                },);
                expect(((typeof input) === 'object') || ((typeof input) === 'function'),).toBe(true,);
                expect(((typeof view) === 'object') || ((typeof view) === 'function'),).toBe(true,);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(TypeError,);
                expect(
                  (error as Error).message,
                ).toBe(`Expected \`input\` to be a \`Function\` or \`Object\`, got \`${input === null ? 'null' : typeof input}\``,);
              }
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    //endregion Totality
  ],
},);
