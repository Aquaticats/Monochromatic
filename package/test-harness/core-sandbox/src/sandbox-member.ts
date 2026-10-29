/**
 Discover only Sinon functions introduced into collection descriptors. @module
 */
import { isSandboxTarget, } from './sandbox-value.ts';

/**
 Finds new data-method and accessor fakes without invoking application getters.

 @param value - whole-object or descriptor collection returned by Sinon
 
 @param previous - descriptors preceding the operation, empty for a fresh instance
 
 @returns introduced fake identities, including function-object static members

 @example
 ```ts
 const fakes = collectionFakes({ value, previous });
 ```
 */
export function collectionFakes({
  value,
  previous = {},
}: {
  readonly value: unknown;
  readonly previous?: PropertyDescriptorMap;
},): readonly object[] {
  if (!isSandboxTarget(value,))
    return [];
  return Reflect.ownKeys(value,)
    .flatMap(function collectPropertyFakes(property: PropertyKey,): object[] {
    /**
     Reading descriptors does not execute application accessors.
     */
      const descriptor = Object.getOwnPropertyDescriptor(
        value,
        property,
      );
      return ([
        'value',
        'get',
        'set',
      ] as const).flatMap(function collectDescriptorFake(field: keyof PropertyDescriptor,): object[] {
      /**
       New members may be data methods or accessor spies.
       */
        const member: unknown = descriptor?.[field];
        if (((typeof member) !== 'function') || (member === previous[property]?.[field])
          || (Reflect.get(
            member,
            'isSinonProxy',
          ) !== true))
          return [];
        return [member,];
      },);
    },);
}
