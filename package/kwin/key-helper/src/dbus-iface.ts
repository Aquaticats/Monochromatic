/**
 D-Bus interface the KWin script drives. Each method is a thin adapter onto the
 key/nvim/launch/state modules; D-Bus method arguments are positional strings
 dictated by `@homebridge/dbus-native`, so they are exempt from the
 named-parameter convention.
 
 @module
 */

import type {
  DbusExportedObject,
  DbusInterfaceDescriptor,
} from '@homebridge/dbus-native';

import { DBUS_IFACE } from './constants.ts';
import { sendKeys } from './keys.ts';
import { launchNewInstance } from './launch.ts';
import { sendNvimInput } from './nvim.ts';
import { setActiveWindowClass } from './state.ts';

//region Method implementation

/**
 Implementation bridging KWin script commands to actuation the KWin script
 cannot perform (spawning processes, injecting keys, RPC to Neovim). Method
 names match {@link keyHelperInterfaceDescriptor}; `@homebridge/dbus-native`
 invokes each with the incoming call's positional string arguments.
 
 @example
 ```ts
 bus.exportInterface(keyHelperInterface, DBUS_PATH, keyHelperInterfaceDescriptor);
 ```
 */
export const keyHelperInterface: DbusExportedObject = {
  /**
   Record focused window's class on every focus change (double-shift scope).
   
   @param windowClass - Resource class reported by KWin script
   */
  SetActiveWindow(windowClass: string): void {
    setActiveWindowClass(windowClass.toLowerCase());
  },
  /**
   Send F20 to Neovim, manual equivalent of a detected double-shift.
   */
  SendF20(): void {
    void sendNvimInput('<F20>');
  },
  /**
   Send an arbitrary Neovim key sequence over RPC.
   
   @param keys - Neovim key notation, e.g. `<F16>`, `<C-w>`, `<Esc>`
   */
  SendNvimKeys(keys: string): void {
    void sendNvimInput(keys);
  },
  /**
   Inject a key combo into the focused app via ydotool.
   
   @param keys - `+`-joined combo, e.g. `ctrl+w`
   */
  SendKeys(keys: string): void {
    void sendKeys(keys);
  },
  /**
   Launch another instance of the focused app (Meta+N).
   
   @param desktopFileName - Focused window's `desktopFileName`, may be empty
   
   @param resourceClass - Focused window's `resourceClass`, may be empty
   */
  LaunchNewInstance(
    desktopFileName: string,
    resourceClass: string
  ): void {
    void launchNewInstance({
      desktopFileName,
      resourceClass,
    });
  },
};

//endregion Method implementation

//region D-Bus signatures

/**
 D-Bus signatures for {@link keyHelperInterface}, passed to `exportInterface`.
 Each entry is `[inSignature, outSignature, inArgNames, outArgNames]`; every
 method returns nothing, hence the empty output signature.
 
 @example
 ```ts
 bus.exportInterface(keyHelperInterface, DBUS_PATH, keyHelperInterfaceDescriptor);
 ```
 */
export const keyHelperInterfaceDescriptor: DbusInterfaceDescriptor = {
  name: DBUS_IFACE,
  methods: {
    SetActiveWindow: [
      's',
      '',
      ['windowClass'],
      []
    ],
    SendF20: [
      '',
      '',
      [],
      []
    ],
    SendNvimKeys: [
      's',
      '',
      ['keys'],
      []
    ],
    SendKeys: [
      's',
      '',
      ['keys'],
      []
    ],
    LaunchNewInstance: [
      'ss',
      '',
      [
        'desktopFileName',
        'resourceClass'
      ],
      []
    ],
  },
};

//endregion D-Bus signatures
