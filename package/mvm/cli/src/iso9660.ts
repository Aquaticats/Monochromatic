/**
 Minimal ISO9660 image generator for cloud-init NoCloud seed ISOs.
 Produces a valid ISO containing small text files with a specified volume label.
 Avoids any external dependency on `genisoimage` or `mkisofs`.
 */

import * as L from './iso9660-layout.ts';

//region Directory record writer

/**
 Writes a single ISO9660 directory entry at the given buffer offset.
 
 @param buf - Target byte array
 
 @param view - DataView for target buffer
 
 @param offset - Byte offset within target buffer
 
 @param isDir - Directory flag (sets 0x02 in flags byte)
 
 @param name - File identifier (or `\u0000` for self, `\u0001` for parent)
 
 @param sector - Starting sector (extent location) for this entry
 
 @param size - Data length in bytes for this entry
 
 @returns Number of bytes written (record length)
 
 @example
 ```ts
 const buf = new Uint8Array(L.SECTOR_SIZE);
 const view = new DataView(buf.buffer);
 const written = writeDirEntry({
   buf,
   view,
   offset: 0,
   isDir: true,
   name: '\u0000',
   sector: L.ROOT_DIRECTORY_SECTOR,
   size: L.SECTOR_SIZE,
 });
 ```
 */
function writeDirEntry({
  buf,
  isDir,
  name,
  offset,
  sector,
  size,
  view,
}: {
  readonly buf: Uint8Array;
  readonly isDir: boolean;
  readonly name: string;
  readonly offset: number;
  readonly sector: number;
  readonly size: number;
  readonly view: DataView;
},): number {
  /**
   ISO9660 file identifier length; the special "self" and "parent" entries collapse to a single byte.
   */
  const nameLen = ((name === '\u0000') || (name === '\u0001')) ? 1 : name.length;
  /**
   Record length must be even per ISO9660.
   */
  const recordLen = L.DIR_FIXED_HEADER_SIZE
    + nameLen
    + (((nameLen % 2) === 0) ? 1 : 0);

  buf[offset] = recordLen;
  L.writeBoth32({
    offset: offset + L
      .DIR_EXTENT_OFFSET,
    value: sector,
    view,
  },);
  L.writeBoth32({
    offset: offset + L
      .DIR_SIZE_OFFSET,
    value: size,
    view,
  },);
  L.writeTimestamp7({
    buf,
    offset: offset + L
      .DIR_TIMESTAMP_OFFSET,
  },);
  buf[offset + L
    .DIR_FLAGS_OFFSET] = isDir ? 0x02 : 0x00;
  L.writeBoth16({
    offset: offset + L
      .DIR_VOL_SEQ_OFFSET,
    value: 1,
    view,
  },);
  buf[offset + L
    .DIR_NAME_LEN_OFFSET] = nameLen;

  if (name === '\u0000')
    buf[offset + L
      .DIR_NAME_DATA_OFFSET] = 0;
  else if (name === '\u0001')
    buf[offset + L
      .DIR_NAME_DATA_OFFSET] = 1;
  else {
    L.writeStr({
      buf,
      len: nameLen,
      offset: offset + L
        .DIR_NAME_DATA_OFFSET,
      str: name,
    },);
  }

  return recordLen;
}

//endregion

/**
 Creates a minimal ISO9660 image containing the given files.
 
 Layout: system area (sectors 0-15), PVD (16), VDST (17),
 path table L (18), path table M (19), root directory (20), file data (21+).
 
 File identifiers use lowercase names with ";1" version suffix.
 This is technically non-Level-1-compliant but the Linux kernel reads it correctly,
 which is all cloud-init needs.
 
 @param files - Array of files to include, each with name and data
 
 @param volumeId - Volume identifier string for the ISO
 
 @returns Complete ISO9660 image as a byte array
 
 @example
 ```ts
 const iso = createIso({
   volumeId: 'cidata',
   files: [
     { name: 'user-data', data: new TextEncoder().encode('#cloud-config\n') },
     { name: 'meta-data', data: new TextEncoder().encode('instance-id: test\n') },
   ],
 });
 ```
 */
export function createIso({
  files,
  volumeId,
}: {
  readonly files: readonly {
    readonly data: Uint8Array;
    readonly name: string;
  }[];
  readonly volumeId: string;
},): Uint8Array {
  /**
   Cursor advancing across file-data sectors; assigned to each entry then incremented.
   */
  let nextSector = L.FIRST_FILE_DATA_SECTOR;
  /**
   Per-file layout records carrying the assigned starting sector; iterated below to emit directory entries and payloads.
   */
  const entries = files.map(function mapFileEntry(f,) {
    /**
     Sector this file occupies; reserved before `nextSector` is bumped past the file.
     */
    const sector = nextSector;
    nextSector += Math.ceil(f.data
      .length
      / L
        .SECTOR_SIZE,)
      || 1;
    return {
      data: f.data,
      name: f.name,
      sector,
    };
  },);

  /**
   Total sectors required for system area, descriptors, root dir, and all file data; sizes the output buffer.
   */
  const totalSectors = nextSector;
  /**
   Output ISO image buffer; zero-filled with `totalSectors` worth of bytes.
   */
  const iso = new Uint8Array(totalSectors * L
    .SECTOR_SIZE,);
  /**
   Typed DataView over `iso` for endian-aware writes of multi-byte integers.
   */
  const view = new DataView(iso.buffer,);
  /**
   Root directory occupies exactly one sector; large enough for the entries this tool emits.
   */
  const rootDirSize = L.SECTOR_SIZE;

  //region Primary Volume Descriptor (sector 16)
  /**
   Byte offset of the PVD inside `iso`; PVD lives at sector 16 by spec.
   */
  const pvd = L.PVD_SECTOR
    * L
      .SECTOR_SIZE;
  iso[pvd] = 1;
  L.writeStr({
    buf: iso,
    len: L.CD001_LENGTH,
    offset: pvd + 1,
    str: 'CD001',
  },);
  iso[pvd + L
    .PVD_VERSION_OFFSET] = 1;
  L.writeStr({
    buf: iso,
    len: L.ISO_STRING_FIELD_LENGTH,
    offset: pvd + L
      .PVD_SYSTEM_ID_OFFSET,
    str: '',
  },);
  L.writeStr({
    buf: iso,
    len: L.ISO_STRING_FIELD_LENGTH,
    offset: pvd + L
      .PVD_VOLUME_ID_OFFSET,
    str: volumeId,
  },);
  L.writeBoth32({
    offset: pvd + L
      .PVD_VOLUME_SPACE_OFFSET,
    value: totalSectors,
    view,
  },);
  L.writeBoth16({
    offset: pvd + L
      .PVD_VOLUME_SET_SIZE_OFFSET,
    value: 1,
    view,
  },);
  L.writeBoth16({
    offset: pvd + L
      .PVD_VOLUME_SEQ_OFFSET,
    value: 1,
    view,
  },);
  L.writeBoth16({
    offset: pvd + L
      .PVD_BLOCK_SIZE_OFFSET,
    value: L.SECTOR_SIZE,
    view,
  },);
  L.writeBoth32({
    offset: pvd + L
      .PVD_PATH_TABLE_SIZE_OFFSET,
    value: L.PATH_TABLE_SIZE,
    view,
  },);
  view.setUint32(
    pvd + L
      .PVD_PATH_TABLE_L_OFFSET,
    L.PATH_TABLE_LE_SECTOR,
    true,
  );
  view.setUint32(
    pvd + L
      .PVD_PATH_TABLE_M_OFFSET,
    L.PATH_TABLE_BE_SECTOR,
    false,
  );
  writeDirEntry({
    buf: iso,
    isDir: true,
    name: '\u0000',
    offset: pvd + L
      .PVD_ROOT_DIR_RECORD_OFFSET,
    sector: L.ROOT_DIRECTORY_SECTOR,
    size: rootDirSize,
    view,
  },);
  L.writeTimestamp17({
    buf: iso,
    offset: pvd + L
      .PVD_CREATION_TIMESTAMP_OFFSET,
  },);
  L.writeTimestamp17({
    buf: iso,
    offset: pvd + L
      .PVD_MODIFICATION_TIMESTAMP_OFFSET,
  },);
  L.writeTimestamp17({
    buf: iso,
    offset: pvd + L
      .PVD_EXPIRATION_TIMESTAMP_OFFSET,
  },);
  L.writeTimestamp17({
    buf: iso,
    offset: pvd + L
      .PVD_EFFECTIVE_TIMESTAMP_OFFSET,
  },);
  iso[pvd + L
    .PVD_FILE_STRUCTURE_VERSION_OFFSET] = 1;
  //endregion

  //region Volume Descriptor Set Terminator (sector 17)
  /**
   Byte offset of the VDST inside `iso`; VDST follows the PVD at sector 17 by spec.
   */
  const vdst = L.VDST_SECTOR
    * L
      .SECTOR_SIZE;
  iso[vdst] = 255;
  L.writeStr({
    buf: iso,
    len: L.CD001_LENGTH,
    offset: vdst + 1,
    str: 'CD001',
  },);
  iso[vdst + L
    .PVD_VERSION_OFFSET] = 1;
  //endregion

  //region Path tables (sectors 18 LE, 19 BE)
  for (const [ptSector, le,] of [
    [
      L.PATH_TABLE_LE_SECTOR,
      true,
    ],
    [
      L.PATH_TABLE_BE_SECTOR,
      false,
    ],
  ] as const) {
    /**
     Byte offset of the current path table inside `iso`; emitted once in LE then once in BE.
     */
    const pt = ptSector * L
      .SECTOR_SIZE;
    iso[pt] = 1;
    view.setUint32(
      pt + 2,
      L.ROOT_DIRECTORY_SECTOR,
      le,
    );
    view.setUint16(
      pt + L
        .PATH_TABLE_PARENT_DIR_OFFSET,
      1,
      le,
    );
  }
  //endregion

  //region Root directory (sector 20)
  /**
   Write cursor inside the root directory sector; bumped by each emitted dir entry.
   */
  let pos = L.ROOT_DIRECTORY_SECTOR
    * L
      .SECTOR_SIZE;
  pos += writeDirEntry({
    buf: iso,
    isDir: true,
    name: '\u0000',
    offset: pos,
    sector: L.ROOT_DIRECTORY_SECTOR,
    size: rootDirSize,
    view,
  },);
  pos += writeDirEntry({
    buf: iso,
    isDir: true,
    name: '\u0001',
    offset: pos,
    sector: L.ROOT_DIRECTORY_SECTOR,
    size: rootDirSize,
    view,
  },);

  for (const entry of entries) {
    pos += writeDirEntry({
      buf: iso,
      isDir: false,
      name: `${entry.name};1`,
      offset: pos,
      sector: entry.sector,
      size: entry.data
        .length,
      view,
    },);
  }
  //endregion

  //region File data
  for (const entry of entries) {
    iso.set(
      entry.data,
      entry.sector
        * L
          .SECTOR_SIZE,
    );
  }
  //endregion

  return iso;
}
