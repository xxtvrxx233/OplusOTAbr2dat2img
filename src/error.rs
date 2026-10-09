//! The error type shared by every stage of a conversion.

use std::io;
use std::path::{Path, PathBuf};

/// Convenience alias for the results this crate returns.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong while turning a block-OTA payload into a raw
/// partition image.
///
/// The variants carry enough context (file name, line number, mismatch) to be
/// shown to a user as-is.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An I/O operation failed. `context` names both the file and what was
    /// being done to it, for example ``reading `system.new.dat.br` ``.
    #[error("{context}: {source}")]
    Io {
        /// What was being done, including the file name.
        context: String,
        /// The error reported by the operating system.
        #[source]
        source: io::Error,
    },

    /// The transfer list does not follow the `*.transfer.list` grammar.
    #[error("{}:{line}: {message}", path.display())]
    Parse {
        /// The transfer list being read.
        path: PathBuf,
        /// 1-based line number the problem is on.
        line: usize,
        /// What is wrong with that line.
        message: String,
    },

    /// The transfer list asks for a command that a block image cannot contain,
    /// or that this tool does not implement.
    #[error(
        "{}:{line}: unsupported command `{command}` (a full `.new.dat` transfer list \
         only uses `new`, `zero` and `erase`)",
        path.display()
    )]
    UnsupportedCommand {
        /// The transfer list being read.
        path: PathBuf,
        /// 1-based line number the command is on.
        line: usize,
        /// The command we do not know how to apply.
        command: String,
    },

    /// A consistency check between the header, the commands and the payload
    /// failed. These are the checks that catch a truncated download or a
    /// transfer list that belongs to a different payload.
    #[error("{}: {message}", path.display())]
    Verify {
        /// The transfer list the check was run against.
        path: PathBuf,
        /// Which check failed.
        message: String,
    },

    /// The payload ended before every `new` block had been read.
    #[error(
        "payload ends after {found} of the {expected} blocks the transfer list \
         needs (truncated or corrupt `.new.dat`?)"
    )]
    TruncatedPayload {
        /// Blocks that were available.
        found: u64,
        /// Blocks the transfer list asked for.
        expected: u64,
    },

    /// No transfer list next to the payload, and none was named on the command line.
    #[error("no transfer list at `{path}`; pass `--transfer-list` to choose one")]
    MissingTransferList {
        /// Where we looked for it.
        path: PathBuf,
    },

    /// The output image is already there and overwriting was not requested.
    #[error("`{path}` already exists; pass `--force` to overwrite it")]
    OutputExists {
        /// The file we refused to clobber.
        path: PathBuf,
    },

    /// A transfer list name cannot be derived from the payload file name.
    #[error("cannot derive a transfer list name from `{path}`; pass `--transfer-list`")]
    UnnamedPayload {
        /// The payload whose name we could not make sense of.
        path: PathBuf,
    },
}

impl Error {
    /// An I/O failure while reading `path`.
    pub(crate) fn reading(path: &Path, source: io::Error) -> Self {
        Self::io(format!("reading `{}`", path.display()), source)
    }

    /// An I/O failure while creating `path`.
    pub(crate) fn creating(path: &Path, source: io::Error) -> Self {
        Self::io(format!("creating `{}`", path.display()), source)
    }

    /// An I/O failure on a stream that has no file name.
    pub(crate) fn stream(context: &str, source: io::Error) -> Self {
        Self::io(context.to_owned(), source)
    }

    fn io(context: String, source: io::Error) -> Self {
        Self::Io { context, source }
    }

    /// A malformed transfer list, pointing at `line`.
    pub(crate) fn parse(path: &Path, line: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            path: path.to_path_buf(),
            line,
            message: message.into(),
        }
    }
}
