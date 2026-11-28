//! What:
//!  A pool a control scripts:
//!  the device's state,
//!  the layout gate's verdict,
//!  one reading per dataset,
//!  the container's children,
//!  and one answer per dataset's delegation.
//! Why:
//!  No control on this machine may open `/dev/zfs`,
//!  because this machine's home is on a ZFS pool,
//!  so the gather's own seam is what a control drives
//!  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "No test on the host opens `/dev/zfs`").
//!  Every answer a control does not state is the healthy one for the design's
//!  layout,
//!  so a control that judges one fact states that fact and nothing else.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pool = AScriptedPool.aHealthyMachine();
//! pool.datasets.set(CHECKOUT_DATASET, { kind: 'missing', why: 'no such dataset' });
//! ```

/// The seam the gather asks the pool through.
use super::the_machines_pool_answers::TheMachinesPool;
/// One grant the kernel answered.
use crate::doctor_zfs_route::delegation_reply::Grant;
/// The dataset facts builder every control of this crate reads,
/// so a scripted pool answers the kernel's own shape.
use crate::doctor_zfs_route::route_test_facts::DatasetBuilder;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CLONE_DATASET,
    CONTAINER_DATASET,
    POOL,
};
/// The device's state and the layout gate's three answers.
use crate::doctor_zfs_route::zfs_checks::{
    AbiFacts,
    DeviceState,
};
/// What one dataset's read answered.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// The facts one dataset's read answered,
/// and the dataset a control states them for.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts;

/// What:
///  The pool's answers,
///  each one a control may state.
/// Why:
///  One value answers every question the gather asks the kernel,
///  so a control changes the one answer it judges and every other read stays
///  the healthy one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AScriptedPool = { device; abi; datasets: Map<string, Reading<CloneObjectFacts>>; children: string[] | Error; grants: Map<string, Grant[] | Error> };
/// ```
pub(crate) struct AScriptedPool {
    /// What the look at the device and the open answered.
    pub device: DeviceState,
    /// What the layout gate answered.
    pub abi: AbiFacts,
    /// One reading per dataset,
    /// in the order the design's layout names them.
    pub datasets: Vec<(String, Reading<CloneObjectFacts>)>,
    /// The clone container's children,
    /// or the reason the listing failed.
    pub children: Result<Vec<String>, String>,
    /// One answer per dataset's delegation,
    /// or the reason the reply could not be read.
    pub grants: Vec<(String, Result<Vec<Grant>, String>)>,
}

impl AScriptedPool {
    /// What:
    ///  The pool of a machine whose route is the design's own:
    ///  the device opens,
    ///  the layout matches,
    ///  the pool,
    ///  the checkout and the container read,
    ///  one clone hangs below the container,
    ///  and no dataset's delegation has a grant.
    /// Why:
    ///  A control that judges one fact changes that fact,
    ///  so every other answer must be the one a healthy machine gives,
    ///  or the control's own finding would be lost among others.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pool = AScriptedPool.aHealthyMachine(); // device opens, layout compatible, three datasets, one child
    /// ```
    pub(crate) fn a_healthy_machine() -> Self {
        return AScriptedPool {
            device: DeviceState::Opens,
            abi: AbiFacts::Compatible {
                module: Some(String::from("2.4.4-1")),
                details: vec![String::from("base type information 1234 bytes")],
            },
            datasets: vec![
                (
                    String::from(POOL),
                    Reading::Read(DatasetBuilder::healthy(POOL, "/nonexistent").build()),
                ),
                (
                    String::from(CHECKOUT_DATASET),
                    Reading::Read(DatasetBuilder::checkout().build()),
                ),
                (
                    String::from(CONTAINER_DATASET),
                    Reading::Read(DatasetBuilder::container().build()),
                ),
                (
                    String::from(CLONE_DATASET),
                    Reading::Read(DatasetBuilder::clone().build()),
                ),
            ],
            children: Ok(vec![String::from(CLONE_DATASET)]),
            grants: vec![
                (String::from(CHECKOUT_DATASET), Ok(Vec::new())),
                (String::from(CONTAINER_DATASET), Ok(Vec::new())),
            ],
        };
    }

    /// What:
    ///  The same pool with one dataset's read replaced.
    /// Why:
    ///  A control that judges a dataset's absent or unreadable state states
    ///  that one reading and leaves the rest of the machine as it was.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pool = AScriptedPool.a_healthy_machine().answering(CONTAINER_DATASET, { kind: 'unreadable', why: 'EACCES' });
    /// ```
    pub(crate) fn answering(
        mut self,
        dataset: &str,
        reading: Reading<CloneObjectFacts>,
    ) -> Self {
        let mut datasets: Vec<(String, Reading<CloneObjectFacts>)> = Vec::new();
        let mut placed: bool = false;
        for (name, held) in self.datasets {
            if name == dataset {
                datasets.push((String::from(dataset), reading.clone()));
                placed = true;
                continue;
            }
            datasets.push((name, held));
        }
        if !placed {
            datasets.push((String::from(dataset), reading));
        }
        self.datasets = datasets;
        return self;
    }
}

impl TheMachinesPool for AScriptedPool {
    fn the_devices_state(&self) -> DeviceState {
        return self.device.clone();
    }

    fn the_layouts_verdict(&self) -> AbiFacts {
        return self.abi.clone();
    }

    fn one_datasets_facts(
        &self,
        dataset: &str,
    ) -> Reading<CloneObjectFacts> {
        for (name, reading) in &self.datasets {
            if name == dataset {
                return reading.clone();
            }
        }
        return Reading::Missing {
            why: format!("the scripted pool holds no answer for {dataset}"),
        };
    }

    fn the_containers_children(
        &self,
        container: &str,
    ) -> Result<Vec<String>, String> {
        let _container: &str = container;
        return match &self.children {
            Ok(children) => Ok(children.clone()),
            Err(why) => Err(String::from(why.as_str())),
        };
    }

    fn the_grants_at(
        &self,
        dataset: &str,
    ) -> Result<Vec<Grant>, String> {
        for (name, answer) in &self.grants {
            if name == dataset {
                return match answer {
                    Ok(grants) => Ok(grants.clone()),
                    Err(why) => Err(String::from(why.as_str())),
                };
            }
        }
        return Ok(Vec::new());
    }
}
