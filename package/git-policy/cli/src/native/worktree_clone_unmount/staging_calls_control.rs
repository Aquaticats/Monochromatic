//! What:
//!  A recording implementation of the staging calls,
//!  for the controls that prove the identity comparisons and the order of the
//!  unmount.
//! Why:
//!  A control on a person's machine cannot mount or unmount,
//!  and the guest proves the calls themselves;
//!  here every call is recorded,
//!  every answer is scripted,
//!  and an arm asserts that no move ran before an identity refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const calls = new RecordingCalls(); calls.mountId = 41; expect(calls.calls).toEqual(['mount-id']);
//! ```

use super::staging_calls::{
    StagingCall,
    StagingCallRefusal,
    StagingCalls,
};
use crate::fs_mount_options::MountFacts;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::path::PathBuf;

/// One call as an arm reads it back.
#[allow(dead_code, reason = "an arm reads the order and a detail it needs")]
pub(crate) struct Recorded {
    /// The call's word.
    pub call: &'static str,
    /// What the arm passed,
    /// such as a descriptor or a name.
    pub detail: String,
}

/// The calls an arm scripts and the order they were made in.
pub(crate) struct RecordingCalls {
    /// Every call,
    /// in order.
    pub made: RefCell<Vec<Recorded>>,
    /// The mount number every descriptor reports,
    /// unless `mount_ids` gives one for the next read.
    pub mount_id: u64,
    /// The mount numbers the next reads answer,
    /// in order,
    /// so a control can tell a parent's mount from a clone's.
    pub mount_ids: RefCell<VecDeque<u64>>,
    /// The mount table the path test and the report read.
    pub table: RefCell<Vec<MountFacts>>,
    /// The facts `mount_facts` answers,
    /// and whether it refuses.
    pub facts: RefCell<Option<MountFacts>>,
    /// The facts `mount_facts` answers for named mounts,
    /// consulted before `facts`.
    pub facts_by_id: RefCell<Vec<(u64, MountFacts)>>,
    /// The path every descriptor names.
    pub observed: PathBuf,
    /// What the handle open answers.
    pub open_handle: RefCell<Option<Result<std::fs::File, StagingCallRefusal>>>,
    /// What the bind answers.
    pub bind: RefCell<Option<Result<std::fs::File, StagingCallRefusal>>>,
    /// What each by-name open answers,
    /// in order.
    pub open_by_name: RefCell<VecDeque<Result<std::fs::File, StagingCallRefusal>>>,
    /// What the move answers.
    pub move_result: RefCell<Option<StagingCallRefusal>>,
    /// What each unmount answers,
    /// in order.
    pub unmount_results: RefCell<VecDeque<Option<StagingCallRefusal>>>,
    /// What `name_handle` answers.
    pub name_handle: RefCell<Option<Result<(i32, Vec<u8>), StagingCallRefusal>>>,
    /// The namespace pair.
    pub namespace: (u64, u64),
    /// This boot's identifier.
    pub boot: String,
}

impl Default for RecordingCalls {
    fn default() -> RecordingCalls {
        return RecordingCalls {
            made: RefCell::new(Vec::new()),
            mount_id: 41,
            mount_ids: RefCell::new(VecDeque::new()),
            table: RefCell::new(Vec::new()),
            facts: RefCell::new(None),
            facts_by_id: RefCell::new(Vec::new()),
            observed: PathBuf::from("/fixture/slot"),
            open_handle: RefCell::new(None),
            bind: RefCell::new(None),
            open_by_name: RefCell::new(VecDeque::new()),
            move_result: RefCell::new(None),
            unmount_results: RefCell::new(VecDeque::new()),
            name_handle: RefCell::new(Some(Ok((1, vec![1, 2, 3])))),
            namespace: (7, 9),
            boot: String::from("boot-of-the-control"),
        };
    }
}

impl RecordingCalls {
    /// Record one call.
    fn note(
        &self,
        call: StagingCall,
        detail: String,
    ) {
        self.made.borrow_mut().push(Recorded {
            call: call.word(),
            detail,
        });
    }

    /// The call words,
    /// in order.
    #[allow(dead_code, reason = "the unmount arms read the order back")]
    pub fn words(&self) -> Vec<&'static str> {
        return self
            .made
            .borrow()
            .iter()
            .map(|made| return made.call)
            .collect();
    }

    /// Whether a call ran.
    pub fn ran(
        &self,
        call: StagingCall,
    ) -> bool {
        return self
            .made
            .borrow()
            .iter()
            .any(|made| return made.call == call.word());
    }
}

impl StagingCalls for RecordingCalls {
    fn bind_the_folder(
        &self,
        _folder: &std::fs::File,
        _cache: &std::fs::File,
        name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        self.note(StagingCall::Bind, name.to_string_lossy().to_string());
        return self.bind.borrow_mut().take().unwrap_or_else(|| {
            return Err(StagingCallRefusal {
                call: StagingCall::Bind,
                errno: libc::EIO,
                detail: String::from("the control scripted no bind"),
            });
        });
    }

    fn make_private(
        &self,
        _bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        self.note(StagingCall::MakePrivate, String::new());
        return Ok(());
    }

    fn move_clone(
        &self,
        _clone: &std::fs::File,
        _bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        self.note(StagingCall::Move, String::new());
        return match self.move_result.borrow().as_ref() {
            Some(refusal) => Err(refusal.clone()),
            None => Ok(()),
        };
    }

    fn unmount_top(
        &self,
        held: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        self.note(
            StagingCall::Unmount,
            super::staging_calls::descriptor_path(held)
                .to_string_lossy()
                .to_string(),
        );
        let mut scripted = self.unmount_results.borrow_mut();
        return match scripted.pop_front().flatten() {
            Some(refusal) => Err(refusal),
            None => Ok(()),
        };
    }

    fn name_handle(
        &self,
        _folder: &std::fs::File,
    ) -> Result<(i32, Vec<u8>), StagingCallRefusal> {
        self.note(StagingCall::NameHandle, String::new());
        return self.name_handle.borrow_mut().take().unwrap_or_else(|| {
            return Err(StagingCallRefusal {
                call: StagingCall::NameHandle,
                errno: libc::EIO,
                detail: String::from("the control scripted no handle"),
            });
        });
    }

    fn open_from_handle(
        &self,
        _cache: &std::fs::File,
        _handle_type: i32,
        _handle: &[u8],
    ) -> Result<std::fs::File, StagingCallRefusal> {
        self.note(StagingCall::OpenHandle, String::new());
        return self.open_handle.borrow_mut().take().unwrap_or_else(|| {
            return Err(StagingCallRefusal {
                call: StagingCall::OpenHandle,
                errno: libc::ESTALE,
                detail: String::from("the control scripted a stale handle"),
            });
        });
    }

    fn open_by_name(
        &self,
        _cache: &std::fs::File,
        name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        self.note(StagingCall::OpenBind, name.to_string_lossy().to_string());
        return self
            .open_by_name
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| {
                return Err(StagingCallRefusal {
                    call: StagingCall::OpenBind,
                    errno: libc::EIO,
                    detail: String::from("the control scripted no by-name open"),
                });
            });
    }

    fn mount_id(
        &self,
        _file: &std::fs::File,
    ) -> Result<u64, StagingCallRefusal> {
        self.note(StagingCall::MountId, String::new());
        return Ok(self
            .mount_ids
            .borrow_mut()
            .pop_front()
            .unwrap_or(self.mount_id));
    }

    fn observed_path(
        &self,
        _file: &std::fs::File,
    ) -> Result<PathBuf, StagingCallRefusal> {
        self.note(StagingCall::MountId, String::new());
        return Ok(self.observed.clone());
    }

    fn mount_facts(
        &self,
        mount_id: u64,
    ) -> Result<MountFacts, StagingCallRefusal> {
        self.note(StagingCall::MountTable, String::new());
        let named = self
            .facts_by_id
            .borrow()
            .iter()
            .find(|(id, _)| return *id == mount_id)
            .map(|(_, facts)| return facts.clone());
        if let Some(facts) = named {
            return Ok(facts);
        }
        return match self.facts.borrow().clone() {
            Some(facts) => Ok(facts),
            None => Err(StagingCallRefusal {
                call: StagingCall::MountTable,
                errno: libc::ENOENT,
                detail: String::from("the control scripted no mount facts"),
            }),
        };
    }

    fn mount_table(&self) -> Result<Vec<MountFacts>, StagingCallRefusal> {
        self.note(StagingCall::MountTable, String::new());
        return Ok(self.table.borrow().clone());
    }

    fn namespace_pair(&self) -> Result<(u64, u64), StagingCallRefusal> {
        self.note(StagingCall::Namespace, String::new());
        return Ok(self.namespace);
    }

    fn boot(&self) -> Result<String, StagingCallRefusal> {
        self.note(StagingCall::Boot, String::new());
        return Ok(self.boot.clone());
    }
}
