/**
 What systemd says about a transient controls unit: whether the invocation a campaign deferred to still runs.
 */
import { runCaptured } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/run-captured.ts';

/**
 A unit as systemd reports it.
 */
export type UnitState = {
  /**
   `ActiveState`, or the empty text for a unit systemd does not know.
   */
  readonly state: string;
  /**
   `InvocationID`, or the empty text.
   */
  readonly invocation: string;
};

/**
 Active states in which a unit can still do its work.
 */
const runningStates: ReadonlySet<string> = new Set([
  'active',
  'activating',
  'deactivating',
]);

/**
 Ask systemd about a transient unit.

 @param unit - unit name without the `.service` suffix

 @returns its `ActiveState` and `InvocationID`; both empty for a unit that ended and was unloaded (`--collect`)

 @example
 ```ts
 await inspectUnit('cli-git-mutpar-w5-planted-aaaaaaaaaaaa');
 ```
 */
export async function inspectUnit(unit: string): Promise<UnitState> {
  /**
   What `systemctl show` printed, one `Name=value` line per property.
   */
  const shown = await runCaptured({
    program: 'systemctl',
    args: [
      '--user',
      'show',
      '--property=ActiveState',
      '--property=InvocationID',
      unit,
    ],
    capture: true,
  });
  /**
   Value of a property, empty when absent.

   @param name - property name

   @returns its value
   */
  function valueOf(name: string): string {
    return shown.stdout
      .split('\n')
      .filter(function isProperty(line) {
        return line.startsWith(`${name}=`);
      })
      .map(function value(line) {
        return line.slice(name.length + 1)
          .trim();
      })
      .at(0)
      ?? '';
  }
  return {
    state: valueOf('ActiveState'),
    invocation: valueOf('InvocationID'),
  };
}

/**
 Tell whether a unit state is the invocation that can still leave the record.

 @param state - what systemd reports now

 @param invocation - the invocation the campaign deferred to; an empty one is never running

 @returns true when the unit is in a running state and is that very invocation

 @example
 ```ts
 isInvocationRunning({ state: { state: 'active', invocation: 'abc' }, invocation: 'abc' });
 ```
 */
export function isInvocationRunning({
  state,
  invocation,
}: {
  readonly state: UnitState;
  readonly invocation: string;
}): boolean {
  return (invocation !== '') && (state.invocation === invocation)
    && runningStates.has(state.state);
}
