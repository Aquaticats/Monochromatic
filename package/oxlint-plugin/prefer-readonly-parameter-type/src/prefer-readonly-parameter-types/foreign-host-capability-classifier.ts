/**
 Explicit opaque host capability coverage through parameter bindings.
 
 @module
 */

import type { BindingName, } from 'typescript/ast';
import {
  isBindingElement,
  isIdentifier,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { isForeignHostCapabilityType, } from './foreign-host-capability-identity.ts';

/**
 Tests whether parameter binding contains exact foreign host capability marker.
 
 Destructured parameters retain explicit capability authority when one bound
 field carries marker. Ordinary foreign ownership markers and same-named local
 aliases do not qualify.
 
 @param project - TypeScript project resolving exact marker identity.
 
 @param name - Parameter binding identifier or destructuring pattern.
 
 @returns whether explicit host capability marker reaches binding.
 
 @example
 ```ts
 bindingContainsForeignHostCapability({ project, name: parameter.name });
 ```
 */
export function bindingContainsForeignHostCapability({
  project,
  name,
}: {
  readonly project: Project;
  readonly name: BindingName;
},): boolean {
  if (isIdentifier(name,)) {
    /**
     Semantic type of current bound identifier.
     */
    const type = project.checker
      .getTypeAtLocation(name,);
    return (type !== undefined) && isForeignHostCapabilityType({
      project,
      type,
    },);
  }
  return name.elements
    .some(function nestedBindingContainsCapability(element,): boolean {
      return isBindingElement(element,)
        && (element.name !== undefined)
        && bindingContainsForeignHostCapability({
          project,
          name: element.name,
        },);
    },);
}
