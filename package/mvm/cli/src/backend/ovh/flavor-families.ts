/**
 Which families mvm may pick a machine size from without being told one.

 A flavor's family is the catalogue's own naming: the token before the first
 hyphen, so `d2-2` and `c3-4-flex` are family `d2` and `c3`. mvm keeps a family
 only when OVHcloud documents it as a general-purpose virtual machine sold with
 CPU and memory, and leaves every other family alone, whether or not this build
 has heard of it: a size from a family added after this code was written is then
 refused rather than booted unseen, and a GPU machine is never picked even when it
 is the only size in stock that would fit.

 Kept, with the catalogue's own names as OVHcloud's worker-node guide spells them
 ([GENERAL PURPOSE, COMPUTE OPTIMIZED, MEMORY OPTIMIZED and DISCOVERY][kube-flavors]):

 - `b2`, `b3` for general purpose;
 - `c2`, `c3` for compute optimized;
 - `r2`, `r3` for memory optimized;
 - `d2` for discovery, the entry-level shared-resource family.

 Left out:

 - GPU and accelerator families, which the same guide names as
   `GPU (T2-*, L4-*, L40S-*)` ([its GPU row][kube-gpu]), and the `t1` family the
   catalogue files under `ovh.raid-nvme.t1`. The region's image list also
   carries one image whose `flavorType` restricts it to
   `g1,g2,g3,t1,t2,i1,a100,l4,rtx5000,l40s` (read from a live `BHS5` list on
   2026-11-28, which recorded the value and not which image carries it);
 - the storage shaped family `i1`, which OVHcloud's own concepts page calls
   [Storage Optimized][storage-optimized] and sells for NVMe input/output rather
   than for memory;
 - bare metal sizes, named `bm-*` in the catalogue and called
   [Metal Instances][metal-instances] by OVHcloud;
 - anything whose flavor targets another operating system, which is what a
   `win-*` name and a `windows` `osType` mark.

 OVHcloud's concepts page also names these families as instance types at all
 ([its types table][types-table]). Which families a family token belongs to is
 read from the catalogue, not written here: a name this list does not carry is
 left out.

 [kube-flavors]: https://docs.us.ovhcloud.com/en/public-cloud/managed-kubernetes/datacenters-nodes-storage-flavors#:~:text=GENERAL%20PURPOSE%20(B2%2D*%2C%20B3%2D*)
 [kube-gpu]: https://docs.us.ovhcloud.com/en/public-cloud/managed-kubernetes/datacenters-nodes-storage-flavors#:~:text=GPU%20(T2%2D*%2C%20L4%2D*%2C%20L40S%2D*)
 [storage-optimized]: https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts#:~:text=Storage%20Optimized
 [metal-instances]: https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts#:~:text=Metal%20Instances
 [types-table]: https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts#:~:text=General%20Purpose

 @module
 */

//region Names

/**
 The `osType` a flavor that can boot a Linux image carries. The schema types the
 field as a bare string, so this one value is taken from OVHcloud's public catalog
 as the Linux family rather than from an enum.
 */
export const LINUX_OS_TYPE = 'linux';

/**
 Separator between the family token and the rest of a flavor's name.
 */
const FAMILY_SEPARATOR = '-';

/**
 Families mvm may pick a size from, by their catalogue token, in the order the
 families are documented.
 */
export const KEPT_FLAVOR_FAMILIES: readonly string[] = Object.freeze([
  'b2',
  'b3',
  'c2',
  'c3',
  'd2',
  'r2',
  'r3',
],);

//endregion Names

//region Reading a family

/**
 Reads the family token out of a flavor's name.
 
 @param name - the flavor's own name, such as `c3-4-flex`
 
 @returns the part before the first hyphen, lower-cased, or the whole name when it has no hyphen
 
 @example
 ```ts
 flavorFamilyOf({ name: 'c3-4-flex' }); // 'c3'
 ```
 */
export function flavorFamilyOf({
  name,
}: {
  readonly name: string;
},): string {
  /**
   Where the family token ends, or the whole name when it carries no separator.
   */
  const end = name.indexOf(FAMILY_SEPARATOR,);
  return ((end === (-1)) ? name : name.slice(
    0,
    end,
  )).toLowerCase();
}

/**
 Whether mvm may pick this flavor without being told a size.
 
 @param flavor - flavor the region offers
 
 @returns whether the flavor is a general-purpose Linux machine mvm knows
 
 @example
 ```ts
 isKeptFlavor({ flavor: { name: 'c3-4', osType: 'linux', ... } }); // true
 isKeptFlavor({ flavor: { name: 't1-45', osType: 'linux', ... } }); // false
 ```
 */
export function isKeptFlavor({
  flavor,
}: {
  readonly flavor: {
    readonly name: string;
    readonly osType: string;
  };
},): boolean {
  if (flavor.osType !== LINUX_OS_TYPE) {
    return false;
  }
  return KEPT_FLAVOR_FAMILIES.includes(flavorFamilyOf({ name: flavor.name, },),);
}

//endregion Reading a family
