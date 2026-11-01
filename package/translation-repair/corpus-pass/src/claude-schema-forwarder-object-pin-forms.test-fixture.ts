import {
  objectCase,
  type ObjectMember,
} from './claude-schema-forwarder-object-support.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A forwarder written as a member of an object literal that no file reads
// No file can call such a member without being listed (a call by name) or reported (a read of the object or of the
// member where it is no callee), so the member is silent, and each form of it is a pin
// (`claude-schema-forwarder-object-support.test-fixture.ts`).

/**
 Forms of a member that nothing reads.
 */
const UNREAD_FORMS: readonly {
  readonly member: ObjectMember;
  readonly form: string;
}[] = [
  {
    member: 'arrow',
    form: 'A PROPERTY HOLDING AN ARROW THAT MAPS A LIST',
  },
  {
    member: 'method',
    form: 'A METHOD THAT MAPS A LIST',
  },
  {
    member: 'expression',
    form: 'A PROPERTY HOLDING A FUNCTION EXPRESSION THAT MAPS A LIST',
  },
  {
    member: 'each',
    form: 'A METHOD THAT CALLS A SENDER IN A CALLBACK OF forEach',
  },
  {
    member: 'for',
    form: 'A METHOD THAT CALLS A SENDER IN A LOOP',
  },
  {
    member: 'direct',
    form: 'A METHOD THAT SENDS DIRECTLY',
  },
];

/**
 Cases of a member of an object literal that no file reads, which are silent by design.
 */
export const UNREAD_OBJECT_CASES: readonly ShapeCase[] = UNREAD_FORMS
  .map(function caseOfForm({
    member,
    form,
  },): ShapeCase {
    return objectCase({
      name: `ACCEPTS ${form} THAT NO FILE READS, since no file can call it without being listed or reported (a pin)`,
      member,
      users: [],
      listed: [],
      spots: [],
    },);
  },);

//endregion A forwarder written as a member of an object literal that no file reads
