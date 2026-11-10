/**
 Local type augmentation for the server side of `@homebridge/dbus-native`.
 
 Its shipped `index.d.ts` types only the client path (`getService` /
 `getInterface`) and neither exports `sessionBus` nor types
 `MessageBus.requestName` / `MessageBus.exportInterface`. key-helper is a D-Bus
 service, so it uses exactly those three members; declare them here to match
 the runtime surface in `node_modules/@homebridge/dbus-native/lib/bus.js`.
 
 @module
 */

import type { MessageBus } from '@homebridge/dbus-native';

declare module '@homebridge/dbus-native' {
  //region Declared runtime surface

  /**
   Options for {@link sessionBus}, where `busAddress` overrides what would
   otherwise come from `DBUS_SESSION_BUS_ADDRESS`.
   
   @example
   ```ts
   const bus = sessionBus({ busAddress: 'unix:path=/run/user/1000/bus' });
   ```
   */
  export type SessionBusOptions = {
    readonly busAddress?: string;
  };

  /**
   One method entry in a {@link DbusInterfaceDescriptor}, positionally:
   input signature, output signature, input argument names, output argument
   names.
   
   @example
   ```ts
   const setActiveWindow: DbusMethodDescriptor = ['s', '', ['windowClass'], []];
   ```
   */
  export type DbusMethodDescriptor = readonly [
    inSignature: string,
    outSignature: string,
    inArgNames: readonly string[],
    outArgNames: readonly string[],
  ];

  /**
   Third argument to {@link MessageBus.exportInterface}, mapping each D-Bus
   member name to its {@link DbusMethodDescriptor}.
   
   @example
   ```ts
   const desc: DbusInterfaceDescriptor = { name: 'org.example.Iface', methods: {} };
   ```
   */
  export type DbusInterfaceDescriptor = {
    readonly name: string;
    readonly methods: Readonly<Record<string, DbusMethodDescriptor>>;
  };

  /**
   First argument to {@link MessageBus.exportInterface}. Members are dispatched
   dynamically by name against the descriptor, so each value stays `unknown`.
   
   @example
   ```ts
   const impl: DbusExportedObject = { SayHello: () => 'hi' };
   ```
   */
  export type DbusExportedObject = Readonly<Record<string, unknown>>;

  /**
   Connect to the D-Bus session bus and return its client/service handle.
   
   @param opts - Connection overrides; omit to read `DBUS_SESSION_BUS_ADDRESS`
   
   @example
   ```ts
   const bus = sessionBus();
   ```
   */
  export function sessionBus(opts?: SessionBusOptions): MessageBus;

  interface MessageBus {
    /**
     Publish an interface implementation at an object path so remote callers
     can invoke its declared methods.
     
     @param obj - Implementation whose keys match `iface.methods` entries
     
     @param path - Object path exported at, e.g. `/org/monochromatic/KeyHelper`
     
     @param iface - Descriptor naming interface plus method signatures
     
     @example
     ```ts
     bus.exportInterface(keyHelperInterface, DBUS_PATH, keyHelperInterfaceDescriptor);
     ```
     */
    exportInterface: (
      obj: DbusExportedObject,
      path: string,
      iface: DbusInterfaceDescriptor,
    ) => void;

    /**
     Claim a well-known bus name. Callback receives the numeric `RequestName`
     reply code, where `1` means this connection is now primary owner.
     
     @param name - Well-known name claimed, e.g. `org.monochromatic.KeyHelper`
     
     @param flags - `RequestName` flags bitfield; `0` requests with no queueing
     
     @param callback - Node-style errback yielding reply code; `error` arrives
     as a `{ name, message }` object rather than an `Error` on failure, hence
     `unknown`
     
     @example
     ```ts
     bus.requestName(DBUS_SERVICE, 0, (err, code) => {});
     ```
     */
    requestName: (
      name: string,
      flags: number,
      callback: (
        error: unknown,
        retCode: number,
      ) => void,
    ) => void;
  }

  //endregion Declared runtime surface
}
