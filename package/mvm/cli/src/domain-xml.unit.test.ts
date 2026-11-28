/**
 Unit tests for the libvirt domain definition `mvm` writes: every guest family
 gets one virtio memory balloon that reports the pages the guest frees to the
 host, inside the domain's device list.
 No virtual machine is started.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_MEMORY_MIB,
  DEFAULT_VCPUS,
  domainXml,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 The balloon element every domain carries, as
 `package/mvm/cli/src/domain-xml-builders.ts` renders it.
 */
const REPORTING_BALLOON = '<memballoon model="virtio" freePageReporting="on" />';

/**
 Guest families the libvirt backend creates, each with the arguments that
 decide its domain: a Linux guest gets the cloud-init seed, a Windows guest
 none.
 */
const GUEST_FAMILIES = [
  {
    osFamily: 'linux',
    seedIsoPath: '/vms/dev/seed.iso',
  },
  {
    osFamily: 'windows',
  },
] as const;

/**
 Text of the failure a definition without a device list earns in these tests.
 */
const NO_DEVICE_LIST = 'the domain definition has no device list';

/**
 Virtual CPUs the size tests ask a guest for; deliberately different from the
 default so a definition that ignored the ask would fail them.
 */
const ASKED_VCPUS = 2;

/**
 Memory the size tests ask a guest for, in mebibytes; deliberately different
 from the default so a definition that ignored the ask would fail them.
 */
const ASKED_MEMORY_MIB = 4_096;

/**
 Reads the text one element carries from a domain definition.
 
 @param xml - domain definition to read
 
 @param tag - element name whose text is wanted, such as `vcpu`
 
 @returns the element's own text
 
 @throws Error when the definition carries no such element
 
 @example
 ```ts
 textOf({ xml, tag: 'vcpu' }); // '4'
 ```
 */
function textOf({
  tag,
  xml,
}: {
  readonly tag: string;
  readonly xml: string;
},): string {
  /**
   Opening tag, or -1 when the definition carries none.
   */
  const open = xml.indexOf(`<${tag}`,);
  if (open === (-1))
    throw new Error(`the domain definition has no ${tag} element`,);
  /**
   End of the opening tag's own attributes and start of its text.
   */
  const textStart = xml.indexOf('>', open,) + 1;
  /**
   Closing tag, which ends the element's text.
   */
  const close = xml.indexOf(`</${tag}>`, textStart,);
  if (close === (-1))
    throw new Error(`the ${tag} element is not closed`,);
  return xml.slice(
    textStart,
    close,
  );
}

/**
 Counts the non-overlapping occurrences of one text inside another.
 
 @param haystack - text to search
 
 @param needle - text to count
 
 @returns number of occurrences
 */
function countOf(haystack: string, needle: string,): number {
  return haystack.split(needle,).length - 1;
}

/**
 Reads the domain's device list, the only place libvirt accepts a balloon.
 
 @param xml - complete domain definition
 
 @returns the text between the `<devices>` open and close tags
 
 @throws Error when the definition has no device list
 */
function devicesOf(xml: string,): string {
  /**
   Index of the opening tag, or -1 when the domain carries no device list.
   */
  const start = xml.indexOf('<devices>',);
  /**
   Index of the closing tag after the opening one, or -1 when it is missing.
   */
  const end = xml.indexOf('</devices>',);
  if (start === (-1))
    throw new Error(NO_DEVICE_LIST,);
  if (end === (-1))
    throw new Error(NO_DEVICE_LIST,);
  return xml.slice(
    start,
    end,
  );
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: domainXml.name,
      children: GUEST_FAMILIES.map(function mapFamily(family,) {
        return it({
          name: `gives a ${family.osFamily} guest one reporting memory balloon`,
          fn: async () => {
            /**
             The definition under test, built the way `create` builds one.
             */
            const xml = domainXml({
              diskPath: '/vms/dev/disk.qcow2',
              name: 'dev',
              ...family,
            },);
            /**
             The domain's devices, where libvirt reads the balloon from.
             */
            const devices = devicesOf(xml,);

            expect(countOf(devices, REPORTING_BALLOON,),).toBe(1,);
          },
        },);
      },),
    },),

    //region The size a create asks for

    describe({
      name: 'the guest size',
      children: [
        it({
          name: 'gives a guest the default numbers when the create asked for none',
          fn: async () => {
            /**
             The definition under test, built the way `create` builds one.
             */
            const xml = domainXml({
              diskPath: '/vms/dev/disk.qcow2',
              name: 'dev',
            },);
            expect(textOf({
              tag: 'vcpu',
              xml,
            },),).toBe(String(DEFAULT_VCPUS,),);
            expect(textOf({
              tag: 'memory',
              xml,
            },),).toBe(String(DEFAULT_MEMORY_MIB,),);
            expect(xml,).toContain('unit="MiB"',);
          },
        },),
        it({
          name: 'gives a guest exactly the numbers the create asked for',
          fn: async () => {
            /**
             The definition under test, carrying the asked numbers.
             */
            const xml = domainXml({
              diskPath: '/vms/dev/disk.qcow2',
              memoryMiB: ASKED_MEMORY_MIB,
              name: 'dev',
              vcpus: ASKED_VCPUS,
            },);
            expect(textOf({
              tag: 'vcpu',
              xml,
            },),).toBe(String(ASKED_VCPUS,),);
            expect(textOf({
              tag: 'memory',
              xml,
            },),).toBe(String(ASKED_MEMORY_MIB,),);
          },
        },),
      ],
    },),

    //endregion The size a create asks for
  ],
},);
