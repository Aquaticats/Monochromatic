//! What:
//!  One healthy dataset of the route's layout for each role,
//!  with the fields a control changes:
//!  the dataset's name,
//!  its `mountpoint` and the dataset that set it,
//!  `canmount`,
//!  `setuid`,
//!  `devices`,
//!  `snapdir`,
//!  `encryption` with its key's state,
//!  and the object-set type.
//! Why:
//!  The dataset,
//!  the key and the clone-property checks read these fields,
//!  so one builder of a healthy dataset lets each of their controls change
//!  exactly the field it judges,
//!  and the kernel's facts are built in the shape OpenZFS returns them,
//!  with the source property that says which dataset set each value.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const datasets = [dataset(DatasetRole.Checkout, DatasetBuilder.checkout().build()), dataset(DatasetRole.Container, DatasetBuilder.container().build()), dataset(DatasetRole.Clone, DatasetBuilder.clone().build())];
//! ```

use super::route_test_facts_names::{
    CHECKOUT,
    CHECKOUT_DATASET,
    CLONE_DATASET,
    CONTAINER_DATASET,
};
use super::zfs_checks::{
    DatasetFacts,
    DatasetRole,
    Reading,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    PropertyValue,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::path::PathBuf;

/// One healthy dataset of the route's layout,
/// with the fields a control changes.
pub(crate) struct DatasetBuilder {
    /// The dataset's name.
    name: String,
    /// The `mountpoint` property's value.
    mountpoint: String,
    /// The dataset the `mountpoint` value is inherited from,
    /// or nothing when the dataset set it itself.
    mountpoint_source: Option<String>,
    /// The `canmount` index value,
    /// where the role reads it.
    canmount: Option<u64>,
    /// The `setuid` index value.
    setuid: u64,
    /// The `devices` index value.
    devices: u64,
    /// The `encryption` value.
    encryption: String,
    /// The `keystatus` value.
    key_status: u64,
    /// The `snapdir` index value,
    /// where the role reads it.
    snapdir: Option<u64>,
    /// The object-set type.
    object_type: u32,
    /// Whether the kernel called it a snapshot.
    is_snapshot: bool,
}

impl DatasetBuilder {
    /// The design's healthy shape for one dataset.
    pub(crate) fn healthy(
        name: &str,
        mountpoint: &str,
    ) -> Self {
        return DatasetBuilder {
            name: String::from(name),
            mountpoint: String::from(mountpoint),
            mountpoint_source: None,
            canmount: None,
            setuid: 0,
            devices: 0,
            encryption: String::from("off"),
            key_status: 0,
            snapdir: Some(
                crate::health::doctor_zfs_route::snapshot_directory_checks::SNAPDIR_DISABLED,
            ),
            object_type: super::zfs_checks::FILESYSTEM_OBJECT_SET,
            is_snapshot: false,
        };
    }

    /// A container at the design's shape.
    pub(crate) fn container() -> Self {
        let mut builder: DatasetBuilder = DatasetBuilder::healthy(CONTAINER_DATASET, "legacy");
        builder.canmount = Some(super::zfs_checks::CANMOUNT_OFF);
        return builder;
    }

    /// The checkout at the design's shape.
    pub(crate) fn checkout() -> Self {
        return DatasetBuilder::healthy(CHECKOUT_DATASET, CHECKOUT);
    }

    /// A clone at the design's shape.
    pub(crate) fn clone() -> Self {
        return DatasetBuilder::healthy(CLONE_DATASET, "legacy");
    }

    /// The `mountpoint` this dataset reports.
    pub(crate) fn mountpoint(
        mut self,
        mountpoint: &str,
    ) -> Self {
        self.mountpoint = String::from(mountpoint);
        return self;
    }

    /// The dataset this one inherits its `mountpoint` from,
    /// which is the shape of a container whose `legacy` comes from an ancestor.
    pub(crate) fn inherits_mountpoint_from(
        mut self,
        from: &str,
    ) -> Self {
        self.mountpoint_source = Some(String::from(from));
        return self;
    }

    /// The `setuid` index value.
    pub(crate) fn setuid(
        mut self,
        setuid: u64,
    ) -> Self {
        self.setuid = setuid;
        return self;
    }

    /// The `devices` index value.
    pub(crate) fn devices(
        mut self,
        devices: u64,
    ) -> Self {
        self.devices = devices;
        return self;
    }

    /// The `canmount` index value.
    pub(crate) fn canmount(
        mut self,
        canmount: u64,
    ) -> Self {
        self.canmount = Some(canmount);
        return self;
    }

    /// The `snapdir` index value,
    /// or nothing where the reply leaves the default out.
    pub(crate) fn snapdir(
        mut self,
        snapdir: Option<u64>,
    ) -> Self {
        self.snapdir = snapdir;
        return self;
    }

    /// An encrypted dataset whose key is loaded,
    /// or not where the caller says so.
    pub(crate) fn encrypted(
        mut self,
        key_status: u64,
    ) -> Self {
        self.encryption = String::from("aes-256-gcm");
        self.key_status = key_status;
        return self;
    }

    /// The object-set type,
    /// and whether the kernel called it a snapshot.
    pub(crate) fn object_type(
        mut self,
        object_type: u32,
        is_snapshot: bool,
    ) -> Self {
        self.object_type = object_type;
        self.is_snapshot = is_snapshot;
        return self;
    }

    /// The kernel's facts,
    /// with the properties the checks read beside the fields.
    pub(crate) fn build(self) -> CloneObjectFacts {
        let mut properties: NvList = NvList::new();
        // A value the dataset set itself names the dataset as its source,
        // which is what OpenZFS answers; an inherited one names the ancestor that set it.
        let mountpoint_source: String = match &self.mountpoint_source {
            Some(from) => from.clone(),
            None => self.name.clone(),
        };
        add_property(
            &mut properties,
            "mountpoint",
            mountpoint_source.as_str(),
            &self.mountpoint,
        );
        if let Some(canmount) = self.canmount {
            add_number_property(&mut properties, "canmount", self.name.as_str(), canmount);
        }
        if let Some(snapdir) = self.snapdir {
            add_number_property(
                &mut properties,
                crate::health::doctor_zfs_route::snapshot_directory_checks::SNAPDIR_PROPERTY,
                self.name.as_str(),
                snapdir,
            );
        }
        return CloneObjectFacts {
            name: self.name,
            guid: 7,
            object_type: self.object_type,
            is_snapshot: self.is_snapshot,
            origin: String::from(
                "zpcachyos/ROOT/cos/home/monochromatic-git-checkout@git-worktree-clone-origin-1",
            ),
            mountpoint: PropertyValue::Text(self.mountpoint),
            setuid: PropertyValue::Number(self.setuid),
            devices: PropertyValue::Number(self.devices),
            encryption: PropertyValue::Text(self.encryption),
            encryption_root: PropertyValue::Text(String::from(
                "zpcachyos/ROOT/cos/home/monochromatic-git-checkout",
            )),
            key_status: PropertyValue::Number(self.key_status),
            properties,
        };
    }
}

/// What:
///  One property in the shape OpenZFS returns it:
///  a nested list with the value and the source the kernel names.
/// Why:
///  The source is the dataset that set the value,
///  which is the dataset itself for a local value and the nearest ancestor for an
///  inherited one,
///  so a control can read the difference the route's two checks judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// properties[name] = { value: text, source: source };
/// ```
fn add_property(
    properties: &mut NvList,
    name: &str,
    source: &str,
    text: &str,
) {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", text);
    pair.add_str("source", source);
    properties.add_nvlist(name, pair);
}

/// One number property in the shape OpenZFS returns it,
/// whose source is the dataset itself.
fn add_number_property(
    properties: &mut NvList,
    name: &str,
    dataset: &str,
    number: u64,
) {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", number);
    pair.add_str("source", dataset);
    properties.add_nvlist(name, pair);
}

/// One dataset of the route with its reading.
pub(crate) fn dataset(
    role: DatasetRole,
    facts: CloneObjectFacts,
) -> DatasetFacts {
    return DatasetFacts {
        role,
        name: facts.name.clone(),
        reading: Reading::Read(facts),
    };
}

/// The three datasets of the design's healthy route.
pub(crate) fn healthy_datasets() -> Vec<DatasetFacts> {
    return vec![
        dataset(
            DatasetRole::Checkout {
                mount_point: PathBuf::from(CHECKOUT),
            },
            DatasetBuilder::checkout().build(),
        ),
        dataset(DatasetRole::Container, DatasetBuilder::container().build()),
        dataset(DatasetRole::Clone, DatasetBuilder::clone().build()),
    ];
}
