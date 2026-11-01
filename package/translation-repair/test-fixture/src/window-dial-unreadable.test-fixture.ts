//region Window dial unreadable
// POINTS ONE STRAGGLER WINDOW VARIABLE OF THIS PROCESS AT A VALUE NO DIAL CAN
// READ, for the cases showing that a stage handed its windows waits under those
// and reads neither variable: a stage that still read the process's window
// would refuse on it.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. `process.env` is process-wide, so every
// suite around a case calling this runs one case at a time
// (`global-writes-sequenced.unit.test.ts`).

/**
 The two window dials a stage once read for itself.
 */
type WindowVariable = 'TRANSLATION_REPAIR_STRAGGLER_GRACE_MS' | 'TRANSLATION_REPAIR_WRITER_GRACE_MS';

/**
 Points one window variable at `300s`, which `readWindowDial` refuses, until
 the handle's scope ends.

 @param variable - which window the process should hold unreadable

 @returns Disposable restoring the variable as it stood

 @example
 ```ts
 using round = windowDialUnreadable({ variable: 'TRANSLATION_REPAIR_STRAGGLER_GRACE_MS', },);
 ```
 */
export function windowDialUnreadable({ variable, }: { readonly variable: WindowVariable; },): Disposable {
  /**
   Value as the suite's shell left it.
   */
  const before = process.env[variable];
  process.env[variable] = '300s';
  return {
    [Symbol.dispose]: function restoreWindow(): void {
      if (before === undefined) {
        Reflect.deleteProperty(
          process.env,
          variable,
        );
      } else
        process.env[variable] = before;
    },
  };
}

//endregion Window dial unreadable
