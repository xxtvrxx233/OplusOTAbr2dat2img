//! A small parser for `dynamic_partitions_op_list`, the file that records how a
//! dynamic-partition OTA grows and shrinks each partition.
//!
//! It is only used to cross-check the size of the image we produced:
//!
//! ```text
//! #oplus comment## Add partition my_region.10011000 to group qti_dynamic_partitions
//! #oplus comment#add my_region.10011000 qti_dynamic_partitions
//! # Grow partition my_region.10011000 from 0 to 2760704
//! resize my_region 2760704
//! ```
//!
//! Comments (`#`), blank lines and commands we do not care about are skipped,
//! so that vendor specific footnotes in the file cannot make a conversion fail.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{Error, Result};

/// A few devices record partition sizes in 512-byte sectors rather than bytes.
const SECTOR: u64 = 512;

/// A partition as described by a `dynamic_partitions_op_list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    /// Partition name, e.g. `my_region`.
    pub name: String,
    /// The group it was added to, when the op list says so.
    pub group: Option<String>,
    /// Last `resize` value seen for this partition, in whatever unit the file uses.
    pub size: Option<u64>,
}

/// The partitions a `dynamic_partitions_op_list` mentions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpList {
    partitions: Vec<Partition>,
}

impl OpList {
    /// Read and parse an op list from disk.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| Error::reading(path, e))?;
        Self::parse(BufReader::new(file))
    }

    /// Parse an op list held in memory.
    pub fn parse(reader: impl BufRead) -> Result<Self> {
        let mut list = Self::default();

        for line in reader.lines() {
            let line = line.map_err(|e| Error::stream("reading the op list", e))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens.as_slice() {
                ["add", name, group, ..] => {
                    list.entry(name).group = Some((*group).to_owned());
                }
                ["add", name, ..] => {
                    list.entry(name);
                }
                ["resize", name, size, ..] => {
                    if let Ok(size) = size.parse() {
                        list.entry(name).size = Some(size);
                    }
                }
                ["remove", name, ..] => list.partitions.retain(|p| p.name != *name),
                ["remove_all", ..] => list.partitions.clear(),
                _ => {}
            }
        }

        Ok(list)
    }

    /// The partitions the op list touches, in the order they first appear.
    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }

    /// Look a partition up by its exact name.
    pub fn get(&self, name: &str) -> Option<&Partition> {
        self.partitions.iter().find(|p| p.name == name)
    }

    /// The partition a payload named `stem` belongs to.
    ///
    /// Payloads are named `<partition>.<build id>.new.dat[.br]` while op lists
    /// name the bare partition, so the longest partition name that `stem` starts
    /// with wins: `my_region.10011000` resolves to `my_region`.
    pub fn resolve(&self, stem: &str) -> Option<&Partition> {
        self.partitions
            .iter()
            .filter(|p| stem.starts_with(&p.name))
            .max_by_key(|p| p.name.len())
    }

    /// Compare an image size against the last `resize` recorded for `stem`'s
    /// partition.
    pub fn check_size(&self, stem: &str, image_bytes: u64) -> SizeCheck {
        let Some(partition) = self.resolve(stem) else {
            return SizeCheck::Unknown;
        };
        let Some(size) = partition.size else {
            return SizeCheck::Unknown;
        };

        if size == image_bytes || size.saturating_mul(SECTOR) == image_bytes {
            SizeCheck::Match {
                partition: partition.name.clone(),
                size,
            }
        } else {
            SizeCheck::Mismatch {
                partition: partition.name.clone(),
                expected: size,
                actual: image_bytes,
            }
        }
    }

    fn entry(&mut self, name: &str) -> &mut Partition {
        if let Some(index) = self.partitions.iter().position(|p| p.name == name) {
            return &mut self.partitions[index];
        }
        self.partitions.push(Partition {
            name: name.to_owned(),
            group: None,
            size: None,
        });
        self.partitions.last_mut().expect("just pushed")
    }
}

/// Outcome of cross-checking an image size against a `dynamic_partitions_op_list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeCheck {
    /// The op list and the transfer list agree.
    Match {
        /// The partition that was matched.
        partition: String,
        /// The size the op list records, in its own unit.
        size: u64,
    },
    /// They disagree.
    Mismatch {
        /// The partition that was matched.
        partition: String,
        /// The size the op list records.
        expected: u64,
        /// The size the transfer list produces.
        actual: u64,
    },
    /// No partition in the op list matches the payload name, or it has no `resize`.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
#oplus comment## Add partition my_region.10011000 to group qti_dynamic_partitions
#oplus comment#add my_region.10011000 qti_dynamic_partitions
# Grow partition my_region.10011000 from 0 to 2760704
resize my_region 2760704
";

    fn sample() -> OpList {
        OpList::parse(SAMPLE.as_bytes()).unwrap()
    }

    #[test]
    fn comments_are_skipped() {
        let list = sample();
        assert_eq!(list.partitions().len(), 1);
        assert_eq!(list.partitions()[0].name, "my_region");
    }

    #[test]
    fn resize_is_recorded() {
        assert_eq!(sample().get("my_region").unwrap().size, Some(2_760_704));
        assert!(sample().get("qti_dynamic_partitions").is_none());
    }

    #[test]
    fn a_payload_name_resolves_to_its_partition() {
        assert_eq!(
            sample().resolve("my_region.10011000").unwrap().name,
            "my_region"
        );
        assert!(sample().resolve("system").is_none());
    }

    #[test]
    fn the_longest_matching_partition_wins() {
        let list = OpList::parse(
            "add super qti_dynamic_partitions\nadd system qti_dynamic_partitions\n".as_bytes(),
        )
        .unwrap();
        assert_eq!(list.resolve("system_ext.123").unwrap().name, "system");
    }

    #[test]
    fn sizes_are_compared_in_bytes_and_in_sectors() {
        let list = sample();
        assert_eq!(
            list.check_size("my_region.10011000", 2_760_704),
            SizeCheck::Match {
                partition: "my_region".into(),
                size: 2_760_704
            }
        );
        // 5392 sectors is the same size.
        assert_eq!(
            list.check_size("my_region.10011000", 5_392 * SECTOR),
            SizeCheck::Match {
                partition: "my_region".into(),
                size: 2_760_704
            }
        );
        assert_eq!(
            list.check_size("my_region.10011000", 4096),
            SizeCheck::Mismatch {
                partition: "my_region".into(),
                expected: 2_760_704,
                actual: 4096
            }
        );
        assert_eq!(list.check_size("system.123", 4096), SizeCheck::Unknown);
    }

    #[test]
    fn add_and_remove_are_honoured() {
        let list = OpList::parse(
            "add system qti_dynamic_partitions\nresize system 4096\nremove system\n".as_bytes(),
        )
        .unwrap();
        assert!(list.get("system").is_none());
    }

    #[test]
    fn unknown_commands_are_ignored() {
        // `add_group` takes a group and a maximum size, not a partition name.
        let text = "add_group qti_dynamic_partitions 1024\nwhat is this\n";
        assert!(
            OpList::parse(text.as_bytes())
                .unwrap()
                .partitions()
                .is_empty()
        );
    }
}
