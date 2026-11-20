//! What:
//!  A writer of the type-information bytes a test compares against:
//! a header,
//! a type
//!  section and a string section,
//! built from the kinds and members a test names.
//! Why:
//!  The deviceless tests need files that declare a layout,
//! and no copy of a kernel
//!  file belongs in the repository.
//!  This writer states a module's layout as data a reader can check
//!  against the design's `pahole` output,
//! so a test that refuses a changed module is not asking
//!  the same table twice.

/// The entry point the file-backed checks call.
use super::check_module_layout::{Compatible, LayoutRequest, check_module_layout};
/// The refusal those checks assert on.
use super::refusal::Refusal;
/// The build directory the fixture files live under.
use std::path::PathBuf;

/// The type ids the writer hands out,
/// and the bytes it wrote for each.
pub struct Blob {
    /// The string section,
    /// which starts with the empty name at offset zero.
    strings: Vec<u8>,
    /// The offset a base file's strings add to every name,
    /// which a module's file carries past its base's.
    string_shift: u32,
    /// Each type entry's bytes,
    /// in id order.
    entries: Vec<Vec<u8>>,
}

/// What:
///  One member a test writes into a structure or a union.
/// Why:
///   A member is its name,
/// the id of its type and the bits it sits at,
/// which is
///  everything the comparison reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenMember = { name: string; typeId: number; bitOffset: number };
/// ```
pub struct WrittenMember {
    /// The member's name,
    /// and the empty name for an anonymous container.
    pub name: &'static str,
    /// The member's type id.
    pub type_id: u32,
    /// Bits from the container's first bit.
    pub bit_offset: u32,
}

/// An empty file,
/// which every test starts from.
impl Default for Blob {
    /// An empty file,
    /// whose first type id is one.
    fn default() -> Self {
        return Self::new();
    }
}

/// The type ids one file hands out,
/// and the bytes it wrote for each.
impl Blob {
    /// An empty file,
    /// whose first type id is one.
    pub fn new() -> Self {
        return Blob {
            strings: vec![0],
            string_shift: 0,
            entries: Vec::<Vec<u8>>::new(),
        };
    }

    /// An empty module file over a base whose string section is `shift` bytes,
    /// so its names start where a split file's do.
    pub fn over(shift: u32) -> Self {
        let mut blob: Blob = Blob::new();
        blob.string_shift = shift;
        return blob;
    }

    /// The bytes this file's string section holds.
    pub fn string_bytes(&self) -> u32 {
        return self.strings.len() as u32;
    }

    /// One base integer type,
    /// whose size is stated in bytes.
    pub fn int(&mut self, name: &str, bytes: u32) -> u32 {
        let entry: Vec<u8> = self.entry(name, 1, 0, bytes, &[0, 0, 0, 0]);
        return self.push(entry);
    }

    /// One pointer,
    /// whose width the format states nowhere.
    pub fn pointer(&mut self, name: &str) -> u32 {
        let entry: Vec<u8> = self.entry(name, 2, 0, 0, &[]);
        return self.push(entry);
    }

    /// One typedef of another type.
    pub fn alias(&mut self, name: &str, target: u32) -> u32 {
        let entry: Vec<u8> = self.entry(name, 8, 0, target, &[]);
        return self.push(entry);
    }

    /// One fixed array of `count` elements of `element`.
    pub fn array(&mut self, element: u32, count: u32) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        extra.extend_from_slice(&element.to_le_bytes());
        extra.extend_from_slice(&0_u32.to_le_bytes());
        extra.extend_from_slice(&count.to_le_bytes());
        let entry: Vec<u8> = self.entry("", 3, 0, 0, &extra);
        return self.push(entry);
    }

    /// One structure.
    pub fn structure(&mut self, name: &str, bytes: u32, members: &[WrittenMember]) -> u32 {
        return self.record(name, 4, bytes, members);
    }

    /// One union.
    pub fn union(&mut self, name: &str, bytes: u32, members: &[WrittenMember]) -> u32 {
        return self.record(name, 5, bytes, members);
    }

    /// One enumeration of 32-bit values.
    pub fn enumeration(&mut self, name: &str, bytes: u32, values: &[(&str, i32)]) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        for (value_name, value) in values {
            let offset: u32 = self.name(value_name);
            extra.extend_from_slice(&offset.to_le_bytes());
            extra.extend_from_slice(&value.to_le_bytes());
        }
        let entry: Vec<u8> = self.entry(name, 6, values.len() as u32, bytes, &extra);
        return self.push(entry);
    }

    /// One structure or union with its members.
    fn record(&mut self, name: &str, kind: u32, bytes: u32, members: &[WrittenMember]) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        for member in members {
            let offset: u32 = self.name(member.name);
            extra.extend_from_slice(&offset.to_le_bytes());
            extra.extend_from_slice(&member.type_id.to_le_bytes());
            extra.extend_from_slice(&member.bit_offset.to_le_bytes());
        }
        let entry: Vec<u8> = self.entry(name, kind, members.len() as u32, bytes, &extra);
        return self.push(entry);
    }

    /// The file with the 32-byte header a current tool writes.
    pub fn bytes(&self) -> Vec<u8> {
        return self.bytes_with_header(32);
    }

    /// The file with a header of `header_bytes`,
    /// which is 24 for the oldest header.
    pub fn bytes_with_header(&self, header_bytes: usize) -> Vec<u8> {
        let types: Vec<u8> = self.entries.concat();
        let mut file: Vec<u8> = Vec::<u8>::new();
        file.extend_from_slice(&0xeb9f_u16.to_le_bytes());
        file.push(1);
        file.push(0);
        file.extend_from_slice(&(header_bytes as u32).to_le_bytes());
        file.extend_from_slice(&0_u32.to_le_bytes());
        file.extend_from_slice(&(types.len() as u32).to_le_bytes());
        file.extend_from_slice(&(types.len() as u32).to_le_bytes());
        file.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());
        if header_bytes >= 32 {
            file.extend_from_slice(&(types.len() as u32).to_le_bytes());
            file.extend_from_slice(&0_u32.to_le_bytes());
        }
        file.extend_from_slice(&types);
        file.extend_from_slice(&self.strings);
        return file;
    }

    /// The same file with its bytes changed at `at`,
    /// so a test can move one field.
    pub fn with_byte(mut file: Vec<u8>, at: usize, byte: u8) -> Vec<u8> {
        if let Some(slot) = file.get_mut(at) {
            *slot = byte;
        }
        return file;
    }

    /// One type entry's bytes.
    fn entry(&mut self, name: &str, kind: u32, vlen: u32, size: u32, extra: &[u8]) -> Vec<u8> {
        let name_offset: u32 = self.name(name);
        let info: u32 = (kind << 24) | (vlen & 0x00ff_ffff);
        let mut entry: Vec<u8> = Vec::<u8>::new();
        entry.extend_from_slice(&name_offset.to_le_bytes());
        entry.extend_from_slice(&info.to_le_bytes());
        entry.extend_from_slice(&size.to_le_bytes());
        entry.extend_from_slice(extra);
        return entry;
    }

    /// One name's offset in the string section.
    fn name(&mut self, name: &str) -> u32 {
        let offset: u32 = self.string_shift.saturating_add(self.strings.len() as u32);
        self.strings.extend_from_slice(name.as_bytes());
        self.strings.push(0);
        return offset;
    }

    /// One entry's id.
    fn push(&mut self, entry: Vec<u8>) -> u32 {
        self.entries.push(entry);
        return self.entries.len() as u32;
    }
}

/// The directory one check writes its two files into,
/// under the build directory rather than a system temporary one.
pub fn fixture_directory(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-fixtures")
        .join(name);
    std::fs::create_dir_all(&directory).expect("fixture directory");
    return directory;
}

/// Write the two files one check reads,
/// and answer the gate's verdict.
pub fn check_pair(name: &str, base: &Blob, module: &Blob) -> Result<Compatible, Refusal> {
    return check_raw(name, base.bytes().as_slice(), module.bytes().as_slice());
}

/// Write the two files exactly as given,
/// and answer the gate's verdict.
pub fn check_raw(name: &str, base: &[u8], module: &[u8]) -> Result<Compatible, Refusal> {
    let directory: PathBuf = fixture_directory(name);
    let base_path: PathBuf = directory.join("vmlinux");
    let module_path: PathBuf = directory.join("zfs");
    std::fs::write(&base_path, base).expect("the base file is written");
    std::fs::write(&module_path, module).expect("the module file is written");
    return check_module_layout(&LayoutRequest {
        base_btf: base_path.as_path(),
        module_btf: module_path.as_path(),
    });
}
