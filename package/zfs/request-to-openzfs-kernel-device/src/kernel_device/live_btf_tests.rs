//! What:
//!  The live proof of the ABI gate's first half:
//! the running module's own type
//!  information reports the size,
//! alignment and every field offset the Rust mirror
//!  compiles to,
//! and its pinned header assigns every request the number this
//!  crate sends.
//! Why:
//!  A field at the wrong offset is read as another field,
//! and a wrong request
//!  number sends a destroy as a read;
//! neither can be seen from userspace without
//!  the module.
//! The test reads the type information of the module the kernel loaded from
//!  `/sys/kernel/btf/<module>` with `pahole`,
//! the file and the command the design's ABI gate
//!  names and the start-time checker will read,
//! and the numbers from the release header,
//! so a
//!  release that moved either is refused here before step 6's gate exists.

use super::fixture::Fixture;
use super::fixture_pool::FixturePool;
use super::live_btf_fields::{
    COMMAND_STRUCT_BYTES, DMU_OBJSET_STATS_BYTES, dmu_objset_stats_fields, zfs_cmd_fields,
};
use super::live_btf_nested_fields::{
    DRR_BEGIN_BYTES, ZFS_SHARE_BYTES, ZFS_STAT_BYTES, ZINJECT_RECORD_BYTES, drr_begin_fields,
    zfs_share_fields, zfs_stat_fields, zinject_record_fields,
};
use crate::requests::Operation;
/// The path type the module BTF's location is built from.
use std::path::PathBuf;
/// The process that runs `pahole`.
use std::process::Command;

/// Where the running kernel exposes each loaded module's own type information.
const MODULE_BTF_DIRECTORY: &str = "/sys/kernel/btf";
/// The module whose types this crate mirrors.
const MODULE_NAME: &str = "zfs";
/// The kernel's own type information,
/// which supplies the base types `pahole` needs to read
/// a module's file (`--btf_base`).
const VMLINUX_BTF: &str = "/sys/kernel/btf/vmlinux";
/// Where the guest keeps `pahole`,
/// from the `dwarves` package.
const PAHOLE: &str = "/usr/bin/pahole";

/// One named member of a structure,
/// as the type information reports it.
#[derive(Debug)]
struct Member {
    /// The member's name.
    name: String,
    /// Bytes from the structure's first byte.
    offset: usize,
    /// Bytes the member holds.
    size: usize,
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_running_modules_type_information_matches_the_rust_mirror() {
    let Some(fixture) = Fixture::from_environment(
        "live_btf_tests::the_running_modules_type_information_matches_the_rust_mirror",
    ) else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-layout");
    pool.assert_the_module_release();
    let module: PathBuf = module_btf();
    assert!(
        module.exists(),
        "the running module reports no type information at {}; the recipe makes the guest's \
         module carry it, so a module built without BTF leaves this gate unable to judge the \
         layout",
        module.display()
    );
    let command: String = pahole(&module, "zfs_cmd");
    let (command_size, command_members) = parse_layout(&command);
    assert_eq!(command_size, COMMAND_STRUCT_BYTES);
    assert_eq!(command_size, std::mem::size_of::<crate::layout::ZfsCmd>());
    assert_fields(&command_members, &zfs_cmd_fields());
    let stats: String = pahole(&module, "dmu_objset_stats");
    let (stats_size, stats_members) = parse_layout(&stats);
    assert_eq!(stats_size, DMU_OBJSET_STATS_BYTES);
    assert_fields(&stats_members, &dmu_objset_stats_fields());
    let inject: String = pahole(&module, "zinject_record");
    let (inject_size, inject_members) = parse_layout(&inject);
    assert_eq!(inject_size, ZINJECT_RECORD_BYTES);
    assert_fields(&inject_members, &zinject_record_fields());
    let share: String = pahole(&module, "zfs_share");
    let (share_size, share_members) = parse_layout(&share);
    assert_eq!(share_size, ZFS_SHARE_BYTES);
    assert_fields(&share_members, &zfs_share_fields());
    let legacy_stat: String = pahole(&module, "zfs_stat");
    let (legacy_size, legacy_members) = parse_layout(&legacy_stat);
    assert_eq!(legacy_size, ZFS_STAT_BYTES);
    assert_fields(&legacy_members, &zfs_stat_fields());
    let begin: String = pahole(&module, "drr_begin");
    let (begin_size, begin_members) = parse_layout(&begin);
    assert_eq!(begin_size, DRR_BEGIN_BYTES);
    assert_fields(&begin_members, &drr_begin_fields());
    println!(
        "layout zfs_cmd bytes={command_size} dmu_objset_stats={stats_size} \
         zinject_record={inject_size} drr_begin={begin_size} source={}",
        module.display()
    );
    std::mem::drop(pool);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_release_header_assigns_every_request_the_number_this_crate_sends() {
    let Some(fixture) = Fixture::from_environment(
        "live_btf_tests::the_release_header_assigns_every_request_the_number_this_crate_sends",
    ) else {
        return;
    };
    let source: PathBuf = fixture.source_tree(
        "live_btf_tests::the_release_header_assigns_every_request_the_number_this_crate_sends",
    );
    let meta: String =
        std::fs::read_to_string(source.join("META")).expect("the release META is read");
    let version: &str = meta
        .lines()
        .find_map(|line| {
            return line.strip_prefix("Version:");
        })
        .map(str::trim)
        .expect("the release META names its version");
    assert_eq!(version, "2.4.4", "the release tree is not zfs-2.4.4");
    let header: String = std::fs::read_to_string(source.join("include/sys/fs/zfs.h"))
        .expect("the release header is read");
    for operation in Operation::ALL {
        let constant: String = format!("ZFS_IOC_{},", operation.as_str());
        let Some(line) = header.lines().find(|line| {
            return line.trim_start().starts_with(&constant);
        }) else {
            panic!("the header does not name {constant}");
        };
        let Some(comment) = line.split("/*").nth(1) else {
            panic!("the header line for {constant} carries no number: {line}");
        };
        let digits: String = comment
            .trim()
            .trim_end_matches("*/")
            .trim()
            .trim_start_matches("0x")
            .to_string();
        let number: u64 = u64::from_str_radix(&digits, 16).expect("the header names a hex number");
        assert_eq!(
            number,
            operation.code(),
            "{} is 0x{number:x} in the header and 0x{:x} here",
            operation.as_str(),
            operation.code()
        );
    }
}

#[test]
fn the_layout_parser_reads_a_union_member_at_its_offset() {
    let printed: String = [
        "struct zfs_cmd {",
        "\tuint8_t                     zc_name[4096];        /*     0  4096 */",
        "\tunion {",
        "\t\tzinject_record_t   zc_inject_record;     /* 13296   368 */",
        "\t\tstruct {",
        "\t\t\tchar       zc_pad1[352];         /* 13296   352 */",
        "\t\t\tuint32_t   zc_defer_destroy;     /* 13648     4 */",
        "\t\t};                                       /* 13296   368 */",
        "\t};                                               /* 13296   368 */",
        "\tuint64_t                   zc_zoneid;           /* 13736     8 */",
        "",
        "\t/* size: 13744, cachelines: 215, members: 32 */",
        "};",
    ]
    .join("\n");
    let (size, members) = parse_layout(&printed);
    assert_eq!(size, 13_744);
    assert_eq!(member(&members, "zc_name").offset, 0);
    assert_eq!(member(&members, "zc_name").size, 4_096);
    assert_eq!(member(&members, "zc_inject_record").offset, 13_296);
    assert_eq!(member(&members, "zc_defer_destroy").offset, 13_648);
    assert_eq!(member(&members, "zc_zoneid").offset, 13_736);
    assert!(!members.iter().any(|found| {
        return found.name.is_empty();
    }));
}

/// Where the running kernel exposes the loaded `zfs` module's type information.
fn module_btf() -> PathBuf {
    return PathBuf::from(MODULE_BTF_DIRECTORY).join(MODULE_NAME);
}

/// The type information `pahole` prints for `structure` in `module`,
/// read with the
/// kernel's own types as the base.
fn pahole(module: &std::path::Path, structure: &str) -> String {
    let output = Command::new(PAHOLE)
        .args(["--btf_base", VMLINUX_BTF, "--class_name", structure])
        .arg(module)
        .output()
        .expect("pahole runs in the guest");
    assert!(
        output.status.success(),
        "pahole failed for {structure} in {}: {}",
        module.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// The structure's size and its named members,
/// read from `pahole`'s default
/// layout output.
fn parse_layout(printed: &str) -> (usize, Vec<Member>) {
    let mut size: usize = 0;
    let mut members: Vec<Member> = Vec::new();
    for line in printed.lines() {
        let trimmed: &str = line.trim();
        let Some(start) = trimmed.find("/*") else {
            continue;
        };
        let text: &str = trimmed[..start].trim();
        let comment: &str = trimmed[start + 2..].trim_end_matches("*/").trim();
        if let Some(rest) = comment.strip_prefix("size:") {
            size = rest
                .split(',')
                .next()
                .and_then(|value| {
                    return value.trim().parse().ok();
                })
                .expect("the size comment names the structure's bytes");
            continue;
        }
        let Some(name) = member_name(text) else {
            continue;
        };
        let Some((offset, field_size)) = offset_and_size(comment) else {
            continue;
        };
        members.push(Member {
            name,
            offset,
            size: field_size,
        });
    }
    return (size, members);
}

/// The member a declation line names,
/// or `None` for a brace or a blank line.
fn member_name(text: &str) -> Option<String> {
    let declaration: &str = text.trim().trim_end_matches(';').trim();
    if declaration.is_empty() || declaration.ends_with('}') || declaration.ends_with('{') {
        return None;
    }
    let without_bits: &str = declaration.split(':').next()?;
    let without_extent: &str = without_bits.split('[').next()?;
    let name: &str = without_extent.split_whitespace().next_back()?;
    if name.is_empty() || name.contains('(') || name.contains('*') {
        return None;
    }
    return Some(name.to_string());
}

/// The offset and size a trailing comment states,
/// accepting a bitfield's
/// `offset:bit` form.
fn offset_and_size(comment: &str) -> Option<(usize, usize)> {
    let mut columns = comment.split_whitespace();
    let offset_column: &str = columns.next()?;
    let size_column: &str = columns.next()?;
    let offset: usize = offset_column.split(':').next()?.parse().ok()?;
    let size: usize = size_column.parse().ok()?;
    return Some((offset, size));
}

/// The member named `name`,
/// or a panic naming what was printed instead.
fn member<'a>(members: &'a [Member], name: &str) -> &'a Member {
    return members
        .iter()
        .find(|found| {
            return found.name == name;
        })
        .unwrap_or_else(|| {
            panic!("the type information has no member '{name}'");
        });
}

/// Assert each expected field is reported at its offset and size,
/// and that the
/// module reports no member the mirror lacks.
fn assert_fields(members: &[Member], expected: &[(&str, usize, usize)]) {
    for (name, offset, size) in expected {
        let found: &Member = member(members, name);
        assert_eq!(found.offset, *offset, "the offset of {name}");
        assert_eq!(found.size, *size, "the size of {name}");
    }
    let mut reported: Vec<&str> = members
        .iter()
        .map(|found| {
            return found.name.as_str();
        })
        .collect();
    let mut wanted: Vec<&str> = expected
        .iter()
        .map(|field| {
            return field.0;
        })
        .collect();
    reported.sort_unstable();
    wanted.sort_unstable();
    assert_eq!(reported, wanted);
}
