import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { generateDomainXml, } from '../dist/final/node/domain-xml.mjs';

//region Fixtures

/**
 Inputs every case renders, spelled as the entry scripts pass them so a case reads as the shipped document.
 */
const FIXTURE_DOMAIN = {
  name: 'fixture-domain',
  memoryMib: '16384',
  vcpus: '8',
  qcow2Path: '/var/lib/libvirt/images/fixture-domain.qcow2',
};

/**
 Renders {@link FIXTURE_DOMAIN} with the given name and disk path replacing the fixture's.

 @param overrides - name and disk path the case needs

 @returns Domain XML of the merged fixture

 @example
 ```ts
 const xml = render({ name: 'a & b', },);
 ```
 */
function render(overrides: { readonly name?: string; readonly qcow2Path?: string; },): string {
  return generateDomainXml({
    ...FIXTURE_DOMAIN,
    ...overrides,
  },);
}

//endregion Fixtures

await describe({
  name: 'the libvirt domain XML of the dev VM',
  children: [
    describe({
      name: generateDomainXml.name,
      children: [
        //region The document

        it({
          name: 'renders one KVM domain element around every device',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml.startsWith('<domain type="kvm">',),).toBe(true,);
            expect(xml.endsWith('</domain>',),).toBe(true,);
            expect(xml.split('<domain',).length,).toBe(2,);
          },
        },),
        it({
          name: 'names the domain after the argument and nothing else',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain(`<name>${FIXTURE_DOMAIN.name}</name>`,);
            expect(xml.split('<name>',).length,).toBe(2,);
          },
        },),

        //endregion The document

        //region Guest resources

        it({
          name: 'bounds the guest by the memory in MiB and the vCPU count it is given',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain(`<memory unit="MiB">${FIXTURE_DOMAIN.memoryMib}</memory>`,);
            expect(xml,).toContain(`<vcpu>${FIXTURE_DOMAIN.vcpus}</vcpu>`,);
          },
        },),
        it({
          name: 'boots UEFI firmware from the first hard disk',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain('<os firmware="efi">',);
            expect(xml,).toContain('<type arch="x86_64">hvm</type>',);
            expect(xml,).toContain('<boot dev="hd" />',);
          },
        },),
        it({
          name: 'passes the host CPU through and keeps the clock in UTC',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain('<cpu mode="host-passthrough" />',);
            expect(xml,).toContain('<clock offset="utc" />',);
          },
        },),

        //endregion Guest resources

        //region Devices

        it({
          name: 'attaches the qcow2 path as a virtio disk qemu opens as qcow2',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain('<disk type="file" device="disk">',);
            expect(xml,).toContain('<driver name="qemu" type="qcow2" />',);
            expect(xml,).toContain(`<source file="${FIXTURE_DOMAIN.qcow2Path}" />`,);
            expect(xml,).toContain('<target dev="vda" bus="virtio" />',);
          },
        },),
        it({
          name: 'gives the guest SPICE graphics, 3D acceleration and the guest agent channel',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain('<graphics type="spice">',);
            expect(xml,).toContain('<listen type="none" />',);
            expect(xml,).toContain('<gl enable="yes" />',);
            expect(xml,).toContain('<acceleration accel3d="yes" />',);
            expect(xml,).toContain('<channel type="unix">',);
            expect(xml,).toContain('<target type="virtio" name="org.qemu.guest_agent.0" />',);
          },
        },),
        it({
          name: 'renders the user-mode interface the entry scripts ship',
          fn: async () => {
            /**
             Document under test.
             */
            const xml = render({},);
            expect(xml,).toContain('<interface type="user">',);
          },
        },),

        //endregion Devices

        //region Escaping

        it({
          name: 'escapes XML metacharacters in the domain name',
          fn: async () => {
            /**
             Name carrying every character a hand-written template would let through.
             */
            const name = `a & b <c> "d" 'e'`;
            /**
             Document under test.
             */
            const xml = render({ name, },);
            expect(xml,).toContain('<name>a &amp; b &lt;c&gt; &quot;d&quot; &apos;e&apos;</name>',);
            expect(xml,).not.toContain('<c>',);
          },
        },),
        it({
          name: 'escapes XML metacharacters in the disk path',
          fn: async () => {
            /**
             Disk path carrying the characters a directory name may hold.
             */
            const qcow2Path = '/var/lib/libvirt/images/a&b<c>.qcow2';
            /**
             Document under test.
             */
            const xml = render({ qcow2Path, },);
            expect(xml,).toContain('<source file="/var/lib/libvirt/images/a&amp;b&lt;c&gt;.qcow2" />',);
            expect(xml,).not.toContain('a&b',);
          },
        },),

        //endregion Escaping
      ],
    },),
  ],
},);
