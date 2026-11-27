//! What:
//!  Where Git takes the attributes files that apply whatever the path is:
//! the configuration Git would list,
//! the file `core.attributesFile` names or the default one,
//! and the root file
//! with `info/attributes` and the system file,
//! and the macros among them that set a converting attribute.
//! Why:
//!   The configuration decides which files are read at all,
//! and it is read in process so a decision never depends on starting Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixed = judge.loadFixed(); // the attribute files that apply whatever the path is
//! ```

use crate::repo_read_config_list::list_config;
use crate::repo_read_config_text::{
    ConfigEntry,
    last_entry,
};
use crate::worktree_clean_content::ContentJudge;
use crate::worktree_clean_content::unmodelled;
use crate::worktree_clean_content_attribute_sources::{
    AttributeFile,
    FixedSources,
    SystemAttributes,
    converting_macros,
};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

impl<'a> ContentJudge<'a> {
    /// The configuration Git would list,
    /// read in process.
    pub(super) fn config_entries(&self) -> Result<Vec<ConfigEntry>, String> {
        let Some(listed) = list_config(
            self.common_dir,
            self.git_dir,
            &self.environment.scope,
            self.environment.variables.as_slice(),
        ) else {
            return Err(String::from(
                "the Git configuration cannot be read without running Git, so a global attributes file cannot be ruled out",
            ));
        };
        let mut entries: Vec<ConfigEntry> = Vec::new();
        for item in listed {
            entries.push(item.entry);
        }
        return Ok(entries);
    }

    /// The attributes file `core.attributesFile` names,
    /// or the default one,
    /// from the configuration entries.
    pub(super) fn global_attributes_path(
        &self,
        entries: &[ConfigEntry],
    ) -> Result<Option<PathBuf>, String> {
        if let Some(entry) = last_entry(entries, "core.attributesfile") {
            let Some(value) = entry.value.as_deref() else {
                return Err(String::from("core.attributesFile has no value"));
            };
            if let Some(rest) = value.strip_prefix(b"~/") {
                let Some(home) = self.environment.scope.home.as_deref() else {
                    return Err(String::from(
                        "core.attributesFile starts with ~/ but HOME is not set",
                    ));
                };
                return Ok(Some(home.join(OsStr::from_bytes(rest))));
            }
            let path: PathBuf = PathBuf::from(OsStr::from_bytes(value));
            if !path.is_absolute() {
                return Err(format!(
                    "core.attributesFile is {:?}, which is not an absolute path",
                    String::from_utf8_lossy(value)
                ));
            }
            return Ok(Some(path));
        }
        return Ok(
            match (&self.environment.scope.xdg, &self.environment.scope.home) {
                (Some(directory), _) => Some(directory.join("git/attributes")),
                (None, Some(home)) => Some(home.join(".config/git/attributes")),
                (None, None) => None,
            },
        );
    }

    /// The attributes files that apply whatever the path is,
    /// with the macros among them that set a converting attribute.
    pub(super) fn load_fixed(&self) -> Result<FixedSources, String> {
        if let Some(reason) = self.environment.alternate_attribute_source.as_deref() {
            return Err(unmodelled(reason));
        }
        let entries: Vec<ConfigEntry> = self.config_entries()?;
        if last_entry(entries.as_slice(), "attr.tree").is_some() {
            return Err(unmodelled("attr.tree is set"));
        }
        let mut files: Vec<AttributeFile> = Vec::new();
        if let Some(root_file) =
            self.read_in(self.root, String::from(".gitattributes"), Vec::new())?
        {
            files.push(root_file);
        }
        let mut outside: Vec<(PathBuf, String)> = vec![(
            self.common_dir.join("info/attributes"),
            String::from("info/attributes"),
        )];
        match &self.environment.system_attributes {
            SystemAttributes::Disabled => {}
            SystemAttributes::File(path) => {
                outside.push((path.clone(), path.display().to_string()))
            }
            SystemAttributes::Unknown => {
                return Err(String::from(
                    "the location of the system attributes file cannot be established without running Git, so what it sets cannot be ruled out",
                ));
            }
        }
        if let Some(path) = self.global_attributes_path(entries.as_slice())? {
            outside.push((path.clone(), path.display().to_string()));
        }
        for (path, label) in outside {
            if let Some(file) = self.read_outside(path.as_path(), label)? {
                files.push(file);
            }
        }
        let macros: HashSet<Vec<u8>> = converting_macros(files.as_slice());
        return Ok(FixedSources {
            files,
            macros,
        });
    }
}
