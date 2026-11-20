# Provenance of the adapted codec and layout code

This crate's name-value list codec and its command-struct mirror are adapted
from another project,
as the design's "Requests and input ownership" asks
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`).
The design names the original as `Ukko-Ylijumala/zfsdev-rs` at commit
`4559ffc711`;
this record names what came from it,
under which license,
and
what changed.

## The original

- Repository:
  `Ukko-Ylijumala/zfsdev-rs`
- Revision:
  `4559ffc71191ab9326275160ab6d2c461e507bda`
  (2026-11-10T00:21:12+02:00,
  "Cargo.toml:
  version bump")
- Author and copyright holder:
  Mikko Tanner
- License of the original:
  MIT or Apache-2.0,
  at the holder's choice

The files this crate adapted:

- `src/nvlist.rs`:
  the native name-value list codec.
  Everything under
  `src/nvlist/` in this crate comes from it,
  with the changes below.
- `src/ioctl.rs`:
  the `zfs_cmd_t` mirror.
  `src/layout/zfs_cmd_t.rs` follows its
  field order and its device-free pinning approach,
  with the changes below.
- `fuzz/fuzz_targets/nvlist.rs`:
  the codec fuzz target.
  This crate's
  `package/zfs/request-to-openzfs-kernel-device.fuzz/fuzz_targets/nvlist_codec.rs`
  follows it,
  with the changes below.

Not carried over,
as the design requires:

- the original's vendored CDDL reference material (`doc/reference/`),
  which is not part of this crate and is not compiled into it;
- the original's live-pool test suite (`tests/live_zfs.rs`);
- every module this crate does not ship:
  `enums`,
  `props`,
  `stats`,
  `vdev`,
  `kstat`,
  `mount` and `wrapkey`.

## Changes in this crate

The originality is not in dispute,
and neither is the license:
the adapted
files keep the original's copyright line,
its `MIT OR Apache-2.0` SPDX
identifier,
and the MIT and Apache-2.0 texts in `LICENSES/`.
This crate as a whole is `LGPL-3.0-or-later`,
which the MIT and Apache-2.0
grants permit;
the notices below are what those grants require.

### The codec (`src/nvlist/`)

- Native encoding only.
  The original decodes both the native and XDR encodings
  and encodes the native one;
  this crate decodes and encodes only native,
  because `/dev/zfs` accepts only native input and its replies are native.
  The XDR decoder is therefore absent,
  not disabled.
- The scalar decoder is split from the recursive pair loop
  (`decode_scalar_value.rs` and `decode_pairs.rs`),
  and the scalar writer from
  the recursive pair writer,
  so no two files of this crate's folder use each
  other in a loop (rule `LYF`).
- The `thiserror` derive is replaced by hand-written `Display` and
  `std::error::Error` implementations,
  so the codec has no dependency of its
  own.
- The `Cursor` follows the same checked reads,
  in `stream_cursor.rs`.
- Additions:
  `NvData::data_type` (the wire type a value encodes as),
  and the nesting bound at level 20,
  the module's own `nvpair_max_recursion` under `_KERNEL`
  (`module/nvpair/nvpair.c:152`),
  where the original refused past 64 (`src/nvlist.rs:44`).
  The original already refused its own bound on both the read and the write
  side (`src/nvlist.rs:409` and `:600`).
- Not carried:
  the `NvList` helpers `get_u64_array`,
  `add_byte_array` and
  `add_uint8_array` (`src/nvlist.rs:308`,
  `:359`,
  `:365`),
  which no request of this crate builds,
  and the original's reference-encoder test
  (`pack_matches_reference_encoder`,
  `src/nvlist.rs:1259-1279`),
  replaced by the golden vectors in `src/nvlist/pack_tests.rs`.
- An unknown `data_type_t` is kept rather than refused:
  `NvData::Unknown` carries the type number,
  the pair's element count and its value bytes,
  so a decode and re-pack is a fixed point,
  where OpenZFS refuses such a pair
  (`i_get_value_size`'s `default: return (-1)`,
  `module/nvpair/nvpair.c:1153-1156`).

### The layout (`src/layout/zfs_cmd_t.rs`)

- The original mirrors OpenZFS 2.2.2,
  where `zc_inject_record` is a plain
  field of 352 bytes.
  This crate mirrors OpenZFS `zfs-2.4.4`,
  where
  `zinject_record_t` grew to 368 bytes and OpenZFS put the three control fields
  in a union with it,
  so that `zc_defer_destroy`,
  `zc_flags` and
  `zc_action_handle` sit at 13648,
  13652 and 13656 and `zc_cleanup_fd` starts at
  13664.
  It also carries `dds_flags`,
  the byte `zfs-2.4.4` added to `dmu_objset_stats_t`
  (`include/sys/dmu.h:1041`).
  The size assertions follow the module's BTF for `zfs-2.4.4`.
- The buffer that owns the zeroed words,
  its size bounds and its fixed-width
  address are in `src/layout/command_buffer.rs`,
  adapted from the original's
  `CmdBuf`.
- The original's name setting (`set_name`,
  `set_value`) lives in this crate's
  `src/names/`,
  because the check belongs with the field bound it checks.
- Not carried:
  the ioctl call itself,
  the kernel version probe,
  the handle
  options and every request method.
  The live call is step 2 of the
  implementation plan;
  this crate builds and decodes only.

### The fuzz target

- The original's target also walks the decoded property,
  stat and vdev
  decoders,
  which this crate does not ship.
  This crate's target walks the lists
  nested inside the decoded list and asserts the same fixed point over each.
