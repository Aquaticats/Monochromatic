/**
 The names a run's container gets, and the unit a run's own program finds itself in: the parts of handing a container to the manager
 that are only about names.

 @module
 */

import { randomUUID, } from 'node:crypto';

/**
 How many characters of a fresh identifier a container's name carries, which is enough to tell two of them apart.
 */
const NAME_IDENTIFIER_CHARACTERS = 8;

/**
 The suffix the manager writes a transient unit's name with, which a run's own name does not carry.
 */
const SERVICE_SUFFIX = '.service';

/**
 The suffix the manager writes a scope's name with.
 */
const SCOPE_SUFFIX = '.scope';

/**
 The prefix the launcher gives a run's unit.
 */
const RUN_UNIT_PREFIX = 'pi-';

/**
 Reads the name a unit has without its `.service` or `.scope` suffix, which is the name its other units are built from.

 @param unit - the unit's name as the manager writes it

 @returns the name without the suffix, which is the whole name when it has none

 @example
 ```ts
 const name = unitNameWithoutItsSuffix('pi-copy-route.service');
 ```
 */
export function unitNameWithoutItsSuffix(unit: string,): string {
  for (const suffix of [
    SERVICE_SUFFIX,
    SCOPE_SUFFIX,
  ]) {
    if (unit.endsWith(suffix,)) {
      return unit.slice(
        0,
        -suffix.length,
      );
    }
  }
  return unit;
}

/**
 Reads the unit a program runs in from its own cgroup file.

 @param cgroup - the text of `/proc/self/cgroup`

 @returns the unit's name, and nothing when the text names no unit

 @example
 ```ts
 const unit = unitOfThisProcess('0::/user.slice/user-1000.slice/user@1000.service/app.slice/pi-a.service');
 ```
 */
export function unitOfThisProcess(cgroup: string,): string {
  /**
   The last segment of each of the cgroup's lines, which is where a unit's name is.
   */
  const segments = cgroup.split('\n',)
    .map(function lastSegment(line,) {
      return line.trim()
        .split('/',)
        .at(-1,);
    },);
  /**
   The first segment that is a unit's name.
   */
  const named = segments.find(function isAUnit(segment,) {
    return (segment !== undefined) && (unitNameWithoutItsSuffix(segment,) !== segment);
  },);
  return named ?? '';
}

/**
 Reads the run's own name from the unit its program runs in.

 @param ownUnit - the unit this program runs in

 @returns the run's name, which is the unit's name without the launcher's prefix and without the manager's suffix

 @example
 ```ts
 const runName = runNameOf({ ownUnit: 'pi-copy-route.service', });
 ```
 */
export function runNameOf({ ownUnit }: { readonly ownUnit: string },): string {
  /**
   The unit's name without its suffix.
   */
  const withoutSuffix = unitNameWithoutItsSuffix(ownUnit,);
  return ((withoutSuffix.startsWith(RUN_UNIT_PREFIX,)) ? withoutSuffix.slice(RUN_UNIT_PREFIX.length,) : withoutSuffix) || 'run';
}

/**
 Says whether a command asks for a container that outlives the command.

 @param words - the arguments the run gave

 @returns whether the command asks for a detached container

 @example
 ```ts
 const detached = asksForADetachedContainer(['run', '--rm', '-d', 'alpine', '/bin/sleep', '30']);
 ```
 */
export function asksForADetachedContainer(words: readonly string[],): boolean {
  /**
   The subcommand, which is the first argument `podman` does not read as a global option.
   */
  const subcommand = words.find(function isNotAGlobalOption(word,) {
    return !word.startsWith('-',);
  },);
  if ((subcommand !== 'run') && (subcommand !== 'create')) {
    return false;
  }
  return words.some(function isDetach(word,) {
    return (word === '--detach') || (word === '-d');
  },);
}

/**
 Makes the name a container gets when the run named none, which says which run it belongs to.

 @param ownUnit - the unit this program runs in

 @param words - the arguments the run gave

 @returns a name no other container of this run has

 @example
 ```ts
 const name = containerNameOf({ ownUnit: 'pi-a.service', words: ['run', '-d', 'alpine'], });
 ```
 */
export function containerNameOf({
  ownUnit,
  words,
}: {
  readonly ownUnit: string;
  readonly words: readonly string[];
},): string {
  /**
   The run's own name, which the container's name carries.
   */
  const runName = runNameOf({ ownUnit, },);
  if (!asksForADetachedContainer(words,)) {
    return `${runName}-container`;
  }
  /**
   A fresh identifier, which tells two containers of one run apart.
   */
  const identifier = randomUUID()
    .slice(
    0,
    NAME_IDENTIFIER_CHARACTERS,
    );
  return `${runName}-container-${identifier}`;
}
