/**
 Generates libvirt domain XML shared by `build-and-import.ts` and `import.ts`.
 Extracted to keep consumer scripts under the max-lines limit.
 */
import { hXml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Generates libvirt domain XML for the dev VM.
 Uses {@link hXml} with SPICE graphics, virtio disk and NIC, UEFI boot,
 and host-passthrough CPU.
 
 @param name - Libvirt domain name
 
 @param memoryMib - VM memory in MiB as a string
 
 @param vcpus - Virtual CPU count as a string
 
 @param qcow2Path - Absolute path to the qcow2 disk image
 
 @returns Complete libvirt domain XML string
 
 @example
 ```ts
 const xml = generateDomainXml({
   name: 'my-vm',
   memoryMib: '16384',
   vcpus: '8',
   qcow2Path: '/var/lib/libvirt/images/my-vm.qcow2',
 });
 ```
 */
export function generateDomainXml(
  {
    name,
    memoryMib,
    vcpus,
    qcow2Path,
  }: {
    readonly name: string;
    readonly memoryMib: string;
    readonly vcpus: string;
    readonly qcow2Path: string;
  },
): string {
  return h({
    tag: 'domain',
    attrs: { type: 'kvm', },
    children: [
      h({
        tag: 'name',
        text: name,
      },),
      h({
        tag: 'memory',
        attrs: { unit: 'MiB', },
        text: memoryMib,
      },),
      h({
        tag: 'vcpu',
        text: vcpus,
      },),
      h({
        tag: 'os',
        attrs: { firmware: 'efi', },
        children: [
          h({
            tag: 'type',
            attrs: { arch: 'x86_64', },
            text: 'hvm',
          },),
          h({
            tag: 'boot',
            attrs: { dev: 'hd', },
          },),
        ],
      },),
      h({
        tag: 'cpu',
        attrs: { mode: 'host-passthrough', },
      },),
      h({
        tag: 'features',
        children: [h({ tag: 'acpi', },),],
      },),
      h({
        tag: 'clock',
        attrs: { offset: 'utc', },
      },),
      h({
        tag: 'devices',
        children: [
          h({
            tag: 'disk',
            attrs: {
              type: 'file',
              device: 'disk',
            },
            children: [
              h({
                tag: 'driver',
                attrs: {
                  name: 'qemu',
                  type: 'qcow2',
                },
              },),
              h({
                tag: 'source',
                attrs: { file: qcow2Path, },
              },),
              h({
                tag: 'target',
                attrs: {
                  dev: 'vda',
                  bus: 'virtio',
                },
              },),
            ],
          },),
          h({
            tag: 'interface',
            attrs: { type: 'user', },
            children: [
              h({
                tag: 'model',
                attrs: { type: 'virtio', },
              },),
            ],
          },),
          h({
            tag: 'graphics',
            attrs: { type: 'spice', },
            children: [
              h({
                tag: 'listen',
                attrs: { type: 'none', },
              },),
              h({
                tag: 'gl',
                attrs: { enable: 'yes', },
              },),
            ],
          },),
          h({
            tag: 'video',
            children: [
              h({
                tag: 'model',
                attrs: {
                  type: 'virtio',
                  heads: '1',
                },
              },),
              h({
                tag: 'acceleration',
                attrs: { accel3d: 'yes', },
              },),
            ],
          },),
          h({
            tag: 'channel',
            attrs: { type: 'unix', },
            children: [
              h({
                tag: 'target',
                attrs: {
                  type: 'virtio',
                  name: 'org.qemu.guest_agent.0',
                },
              },),
            ],
          },),
        ],
      },),
    ],
  },);
}
