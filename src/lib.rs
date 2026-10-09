//! # br2img
//!
//! Turn the pieces of an Android block-based OTA into a raw partition image:
//!
//! ```text
//! system.new.dat.br -brotli-> system.new.dat -transfer.list-> system.img
//! ```
//!
//! A [`Job`] ties the three files together, and defaults to the layout ROM
//! dumps use, so the payload alone is usually enough:
//!
//! ```no_run
//! use br2img::{Job, NoProgress, Options};
//!
//! let job = Job::from_dat("system.new.dat.br")?;
//! let report = job.run(&Options::default(), NoProgress)?;
//! println!("wrote {} bytes to {}", report.image_bytes, job.output.display());
//! # Ok::<(), br2img::Error>(())
//! ```
//!
//! The payload is decompressed as a stream and the image is written sparsely:
//! blocks that no `new` command fills stay holes, so a few hundred megabytes of
//! real data inside a multi-gigabyte image only costs what it needs on disk.
//!
//! Anything that looks wrong — a header that disagrees with the commands, a
//! transfer list that does not cover the image exactly once, a payload of the
//! wrong length — is reported as an [`Error`] instead of being written out as a
//! silently corrupt image. Set [`Options::verify`] to `false` to skip that.
//!
//! # Features
//!
//! This crate is a library by default. The `br2img` command line tool is built
//! only when the `cli` feature is on:
//!
//! ```text
//! cargo build --release --features cli
//! cargo install --path . --features cli
//! ```

#![warn(missing_docs)]

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

mod dat;
mod error;
mod image;
mod op_list;
mod transfer_list;

pub use crate::dat::NewData;
pub use crate::error::{Error, Result};
pub use crate::image::{ImageSink, ImageStats, write_image};
pub use crate::op_list::{OpList, Partition, SizeCheck};
pub use crate::transfer_list::{BLOCK_SIZE, BlockRange, Command, CommandKind, TransferList};

/// Knobs that change what a conversion does.
#[derive(Debug, Clone)]
pub struct Options {
    /// Cross-check the transfer list header, its commands and the payload
    /// against each other, and fail on a mismatch. On by default; this is what
    /// catches a half-finished download or a transfer list that belongs to a
    /// different payload.
    pub verify: bool,

    /// Overwrite the output image if it is already there. Off by default, so a
    /// typo cannot destroy an image that took ten minutes to build.
    pub overwrite: bool,

    /// A `dynamic_partitions_op_list` to cross-check the image size against.
    /// The result ends up in [`Report::size_check`]; a mismatch is a warning,
    /// not a failure, since op lists are not always in bytes.
    pub op_list: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            verify: true,
            overwrite: false,
            op_list: None,
        }
    }
}

/// Told how far along a running [`Job`] is.
///
/// Every method has an empty default body, so an implementation only needs to
/// override what it cares about. [`NoProgress`] ignores everything.
#[allow(unused_variables)]
pub trait Progress {
    /// The conversion touches `total_blocks` blocks: everything the `new`,
    /// `zero` and `erase` commands cover.
    fn begin(&mut self, total_blocks: u64) {}

    /// `done` blocks are finished. The value only ever grows.
    fn update(&mut self, done: u64) {}

    /// The image has been written.
    fn finish(&mut self) {}
}

impl<P: Progress + ?Sized> Progress for &mut P {
    fn begin(&mut self, total_blocks: u64) {
        (**self).begin(total_blocks);
    }

    fn update(&mut self, done: u64) {
        (**self).update(done);
    }

    fn finish(&mut self) {
        (**self).finish();
    }
}

/// A [`Progress`] that ignores everything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoProgress;

impl Progress for NoProgress {}

/// The three files of one conversion.
///
/// The fields are public so a caller that knows better where the files are can
/// simply assign them; [`Job::from_dat`] fills in the conventional names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// The `*.new.dat.br` (or plain `*.new.dat`) payload.
    pub dat: PathBuf,
    /// The matching `*.transfer.list`.
    pub transfer_list: PathBuf,
    /// Where the raw image goes.
    pub output: PathBuf,
}

impl Job {
    /// A job with all three paths spelled out.
    pub fn new(
        dat: impl Into<PathBuf>,
        transfer_list: impl Into<PathBuf>,
        output: impl Into<PathBuf>,
    ) -> Self {
        Self {
            dat: dat.into(),
            transfer_list: transfer_list.into(),
            output: output.into(),
        }
    }

    /// Derive the other two paths from the payload name, following the layout
    /// of a ROM dump, e.g.:
    ///
    /// ```text
    /// dir/my_region.10011000.new.dat.br
    /// dir/my_region.10011000.transfer.list
    /// dir/my_region.10011000.img
    /// ```
    ///
    /// Fails if there is no transfer list next to the payload.
    pub fn from_dat(dat: impl Into<PathBuf>) -> Result<Self> {
        let dat = dat.into();
        let dir = dat.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        let stem = payload_stem(&dat).ok_or_else(|| Error::UnnamedPayload { path: dat.clone() })?;

        let transfer_list = dir.join(format!("{stem}.transfer.list"));
        if !transfer_list.exists() {
            return Err(Error::MissingTransferList {
                path: transfer_list,
            });
        }

        Ok(Self {
            dat,
            transfer_list,
            output: dir.join(format!("{stem}.img")),
        })
    }

    /// The partition this job is about, as spelled by the payload file name:
    /// `my_region` for `my_region.10011000.new.dat.br`.
    pub fn name(&self) -> Option<String> {
        payload_stem(&self.dat)
    }

    /// Read the transfer list, lay the payload out and write the image.
    pub fn run(&self, options: &Options, mut progress: impl Progress) -> Result<Report> {
        let started = Instant::now();

        let list = TransferList::from_path(&self.transfer_list)?;
        let dat = NewData::new(&self.dat);
        let dat_bytes = dat.file_len()?;

        // Refuse before touching the output whenever the payload size is
        // knowable without decompressing it.
        if options.verify {
            list.verify(dat.known_payload_len()?)?;
        }

        let size_check = match &options.op_list {
            Some(path) => {
                let ops = OpList::from_path(path)?;
                let stem = self.name().unwrap_or_default();
                Some(ops.check_size(&stem, list.image_bytes()))
            }
            None => None,
        };

        // Open the payload before creating the output, so that a payload we
        // cannot read never leaves an empty image behind.
        let mut payload = dat.reader()?;
        let mut file = create_output(&self.output, options.overwrite)?;

        progress.begin(list.covered_blocks());
        let stats = match write_image(&list, &mut payload, &mut file, options.verify, |done| {
            progress.update(done)
        }) {
            Ok(stats) => stats,
            Err(error) => {
                // A half-written image looks exactly like a finished one, so
                // take it away again rather than leave a trap behind.
                drop(file);
                let _ = std::fs::remove_file(&self.output);
                return Err(error);
            }
        };
        progress.finish();

        Ok(Report {
            version: list.version(),
            image_blocks: list.image_blocks(),
            image_bytes: stats.image_bytes,
            blocks_copied: stats.blocks_copied,
            blocks_zeroed: stats.blocks_zeroed,
            payload_bytes: stats.payload_bytes,
            dat_bytes,
            size_check,
            elapsed: started.elapsed(),
        })
    }
}

/// What a finished conversion did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Transfer list version that was applied.
    pub version: u32,
    /// Blocks the image holds.
    pub image_blocks: u64,
    /// Size of the image, in bytes.
    pub image_bytes: u64,
    /// Blocks copied out of the payload.
    pub blocks_copied: u64,
    /// Blocks left untouched, which read back as zero.
    pub blocks_zeroed: u64,
    /// Bytes read out of the payload, i.e. its decompressed size.
    pub payload_bytes: u64,
    /// Size of the payload file on disk — the compressed size for a `.br`.
    pub dat_bytes: u64,
    /// Result of the optional `dynamic_partitions_op_list` cross-check.
    pub size_check: Option<SizeCheck>,
    /// Wall-clock duration of the conversion.
    pub elapsed: Duration,
}

/// Extract the partition name and build ID from a payload file path.
/// e.g. `dir/my_region.10011000.new.dat.br` -> `my_region.10011000`.
fn payload_stem(dat: &Path) -> Option<String> {
    let name = dat.file_name()?.to_str()?;
    let name = name.strip_suffix(".br").unwrap_or(name);
    let name = name
        .strip_suffix(".new.dat")
        .or_else(|| name.strip_suffix(".dat"))
        .unwrap_or(name);
    Some(name.to_owned())
}

/// Create the output image, refusing to touch an existing one unless asked.
fn create_output(path: &Path, overwrite: bool) -> Result<File> {
    let mut open = OpenOptions::new();
    open.write(true);
    if overwrite {
        open.create(true).truncate(true);
    } else {
        // `create_new` fails on an existing file without a check-then-create
        // race, which matters when several conversions run side by side.
        open.create_new(true);
    }

    open.open(path).map_err(|e| {
        if e.kind() == io::ErrorKind::AlreadyExists {
            Error::OutputExists {
                path: path.to_path_buf(),
            }
        } else {
            Error::creating(path, e)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_names_are_stripped_down_to_the_partition_and_build_id() {
        let stem = |p: &str| payload_stem(Path::new(p));
        assert_eq!(
            stem("dir/my_region.10011000.new.dat.br").as_deref(),
            Some("my_region.10011000")
        );
        assert_eq!(
            stem("dir/my_region.10011000.new.dat").as_deref(),
            Some("my_region.10011000")
        );
        assert_eq!(stem("system.dat").as_deref(), Some("system"));
        assert_eq!(stem("system.img").as_deref(), Some("system.img"));
        assert_eq!(stem("/"), None);
    }

    #[test]
    fn from_dat_finds_the_sibling_transfer_list() {
        let dir = tempfile::tempdir().unwrap();
        let dat = dir.path().join("my_region.10011000.new.dat.br");
        std::fs::write(&dat, []).unwrap();
        std::fs::write(dir.path().join("my_region.10011000.transfer.list"), []).unwrap();

        let job = Job::from_dat(&dat).unwrap();
        assert_eq!(
            job.transfer_list,
            dir.path().join("my_region.10011000.transfer.list")
        );
        assert_eq!(job.output, dir.path().join("my_region.10011000.img"));
        assert_eq!(job.name().as_deref(), Some("my_region.10011000"));
    }

    #[test]
    fn from_dat_complains_when_the_transfer_list_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let dat = dir.path().join("system.new.dat.br");
        std::fs::write(&dat, []).unwrap();

        let err = Job::from_dat(&dat).unwrap_err();
        assert!(matches!(err, Error::MissingTransferList { .. }), "{err:?}");
        assert!(err.to_string().contains("system.transfer.list"), "{err}");
    }

    #[test]
    fn the_output_is_never_clobbered_by_accident() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("image.img");
        std::fs::write(&out, b"precious").unwrap();

        let err = create_output(&out, false).unwrap_err();
        assert!(matches!(err, Error::OutputExists { .. }), "{err:?}");
        assert_eq!(std::fs::read(&out).unwrap(), b"precious");

        create_output(&out, true).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"");
    }
}
