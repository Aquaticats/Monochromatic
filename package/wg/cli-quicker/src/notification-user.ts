/**
 The user whose desktop receives notifications.
 */
export type NotificationUser = {
  /**
   Numeric user id.
   */
  readonly uid: number;

  /**
   Numeric primary group id used for the privilege drop.
   */
  readonly gid: number;
};

/**
 The user to notify,
 or why none can be named.
 */
export type NotificationUserLookup =
  | {
    readonly found: true;
    readonly user: NotificationUser
  }
    | {
      readonly found: false;
      readonly reason: string
    };

/**
 The environment values that name the user who started the tunnel through `sudo`.
 */
export type SudoEnvironment = {
  /**
   Numeric user id of the caller.
   */
  readonly SUDO_UID?: string;

  /**
   Numeric primary group id of the caller.
   */
  readonly SUDO_GID?: string;
};

/**
 Reports whether text holds a character that is not an ASCII decimal digit.

 @param value - Text to check.

 @returns Whether any character is not a digit.

 @example
 ```ts
 hasNonDigit({ value: '12a' }); // true
 ```
 */
export function hasNonDigit({ value, }: { readonly value: string; },): boolean {
  for (const character of value)
    if ((character < '0') || (character > '9'))
      return true;
  return false;
}

/**
 Reads one decimal id out of the environment of a sudo-started process.

 @param name - Variable name,
 for the message.

 @param value - Variable value.

 @returns The id,
 or the reason it is unusable.

 @example
 ```ts
 readSudoId({ name: 'SUDO_UID', value: '1000' }); // { ok: true, id: 1000 }
 ```
 */
function readSudoId(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value?: string;
  },
): {
  readonly ok: true;
  readonly id: number
} | {
  readonly ok: false;
  readonly reason: string
} {
  if (value === undefined)
    return {
      ok: false,
      reason: `${name} is not set`,
    };
  /**
   Numeric value after the syntax check.
   */
  const id = Math.trunc(Number(value,),);
  if ((value === '') || hasNonDigit({ value, },)
    || (!Number.isSafeInteger(id,)))
    return {
      ok: false,
      reason: `${name} is not a decimal id: ${value}`,
    };
  return {
    ok: true,
    id,
  };
}

/**
 Names the user who receives desktop notifications:
 the sudo caller,
 the same user the process watch follows.

 @param environment - Environment of the watcher,
 where sudo recorded the caller.

 @returns The user,
 or the reason no user can be named.

 @example
 ```ts
 resolveNotificationUser({ environment: { SUDO_UID: '1000', SUDO_GID: '1000' } });
 ```
 */
export function resolveNotificationUser(
  { environment, }: { readonly environment: SudoEnvironment; },
): NotificationUserLookup {
  /**
   Caller user id.
   */
  const uid = readSudoId({
    name: 'SUDO_UID',
    ...(environment.SUDO_UID === undefined ? {} : { value: environment.SUDO_UID, }),
  },);
  if (!uid.ok)
    return {
      found: false,
      reason: `no user to notify: ${uid.reason}`,
    };
  /**
   Caller group id.
   */
  const gid = readSudoId({
    name: 'SUDO_GID',
    ...(environment.SUDO_GID === undefined ? {} : { value: environment.SUDO_GID, }),
  },);
  if (!gid.ok)
    return {
      found: false,
      reason: `no user to notify: ${gid.reason}`,
    };
  return {
    found: true,
    user: {
      uid: uid.id,
      gid: gid.id,
    },
  };
}
