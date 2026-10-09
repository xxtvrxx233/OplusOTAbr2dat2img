//! Parser for the `*.transfer.list` files that ship next to every block-based
//! OTA payload.
//!
//! The file describes a list of commands, each of which names the target blocks
//! it applies to:
//!
//! ```text
//! 4                                     <- version
//! 674                                   <- blocks the list is expected to cover
//! 0                                     <- stash slots      (version >= 2)
//! 0                                     <- stashed blocks   (version >= 2)
//! new 10,2,6,7,15,519,520,641,655,673,674
//! zero 2,655,673
//! ```
//!
//! Every range set starts with the number of block numbers that follow it, so
//! `10,2,6,…` means ten numbers, i.e. five `start,end` pairs.

use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Granularity of every offset in a transfer list: 4 KiB, the block size used
/// by all Android block-based OTAs.
pub const BLOCK_SIZE: u64 = 4096;

/// Name used in error messages for a list parsed from memory.
const UNNAMED: &str = "<transfer list>";

/// A half-open range of 4 KiB blocks, `start..end`.
///
/// Note that, unlike the numbers in a transfer list, `end` is *not* part of the
/// range: the pair `2,6` covers blocks 2, 3, 4 and 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockRange {
    /// First block of the range.
    pub start: u64,
    /// One past the last block of the range.
    pub end: u64,
}

impl BlockRange {
    /// Number of blocks covered.
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range covers no blocks at all.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Offset of the first block inside the image.
    pub fn byte_offset(&self) -> u64 {
        self.start * BLOCK_SIZE
    }

    /// Number of bytes covered.
    pub fn byte_len(&self) -> u64 {
        self.len() * BLOCK_SIZE
    }
}

/// The commands that occur in a full block-image transfer list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    /// Copy the next blocks of the payload into the target blocks.
    New,
    /// Make the target blocks zero.
    Zero,
    /// Same as [`CommandKind::Zero`], kept for older lists.
    Erase,
}

impl CommandKind {
    /// The token this command is spelled with in a transfer list.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Zero => "zero",
            Self::Erase => "erase",
        }
    }

    /// Whether the command consumes blocks from the payload. Only `new` does;
    /// `zero` and `erase` leave their blocks untouched.
    pub const fn reads_payload(self) -> bool {
        matches!(self, Self::New)
    }

    fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "new" => Self::New,
            "zero" => Self::Zero,
            "erase" => Self::Erase,
            _ => return None,
        })
    }
}

/// One line of a transfer list: a command and the target blocks it applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// What to do with the blocks.
    pub kind: CommandKind,
    /// Blocks the command applies to, in the order they appear in the file.
    pub ranges: Vec<BlockRange>,
}

impl Command {
    /// Total number of blocks this command covers.
    pub fn blocks(&self) -> u64 {
        self.ranges.iter().map(BlockRange::len).sum()
    }

    /// The individual ranges of this command, in file order.
    pub fn ranges(&self) -> impl Iterator<Item = &BlockRange> {
        self.ranges.iter()
    }
}

impl fmt::Display for Command {
    /// Renders the command exactly as it appears in a transfer list.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.kind.as_str(), self.ranges.len() * 2)?;
        for range in &self.ranges {
            write!(f, ",{},{}", range.start, range.end)?;
        }
        Ok(())
    }
}

/// A parsed `*.transfer.list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferList {
    path: PathBuf,
    version: u32,
    total_blocks: u64,
    stash_slots: u64,
    stash_blocks: u64,
    commands: Vec<Command>,
}

impl TransferList {
    /// Oldest transfer list version we know how to apply.
    pub const MIN_VERSION: u32 = 1;
    /// Newest transfer list version we know how to apply.
    pub const MAX_VERSION: u32 = 4;

    /// Read and parse a transfer list from disk.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| Error::reading(path, e))?;
        Self::parse_named(path, BufReader::new(file))
    }

    /// Parse a transfer list held in memory.
    ///
    /// Errors refer to `<transfer list>` instead of a file name; use
    /// [`TransferList::parse_named`] to name the source.
    pub fn parse(reader: impl BufRead) -> Result<Self> {
        Self::parse_named(Path::new(UNNAMED), reader)
    }

    /// Parse a transfer list, naming `path` in error messages.
    pub fn parse_named(path: &Path, reader: impl BufRead) -> Result<Self> {
        let mut lines = LineReader::new(path, reader);

        let version = lines.number("the transfer list version")?;
        let version = u32::try_from(version)
            .ok()
            .filter(|v| (Self::MIN_VERSION..=Self::MAX_VERSION).contains(v));
        let Some(version) = version else {
            return Err(lines.error(format!(
                "unsupported transfer list version (this build handles {}..={})",
                Self::MIN_VERSION,
                Self::MAX_VERSION
            )));
        };

        let total_blocks = lines.number("the number of blocks the list covers")?;

        // Versions 2 and up record how much scratch space applying the list
        // needs. A full `.new.dat` list never stashes anything, so we only keep
        // the numbers around for reporting.
        let (stash_slots, stash_blocks) = if version >= 2 {
            (
                lines.number("the number of stash slots")?,
                lines.number("the number of stashed blocks")?,
            )
        } else {
            (0, 0)
        };

        let mut commands = Vec::new();
        while let Some(line) = lines.next()? {
            let mut tokens = line.split_whitespace();
            let head = tokens.next().expect("a trimmed line is never empty");

            // Lines that start with a number are not commands. `sdat2img.py`
            // skips them, and so do we, so that a doubled header does not trip
            // us up.
            if head.starts_with(|c: char| c.is_ascii_digit()) {
                continue;
            }

            let Some(kind) = CommandKind::parse(head) else {
                return Err(Error::UnsupportedCommand {
                    path: path.to_path_buf(),
                    line: lines.line_no(),
                    command: head.to_owned(),
                });
            };
            let Some(ranges) = tokens.next() else {
                return Err(lines.error(format!("`{}` is missing its block ranges", kind.as_str())));
            };
            if tokens.next().is_some() {
                return Err(lines.error(format!(
                    "`{}` has trailing data after its block ranges",
                    kind.as_str()
                )));
            }

            let ranges = parse_ranges(path, lines.line_no(), ranges)?;
            commands.push(Command { kind, ranges });
        }

        Ok(Self {
            path: path.to_path_buf(),
            version,
            total_blocks,
            stash_slots,
            stash_blocks,
            commands,
        })
    }

    /// Transfer list format version, 1 to 4.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The Android release this version of the format belongs to, for display.
    pub fn android_release(&self) -> &'static str {
        match self.version {
            1 => "Android 5.0 Lollipop",
            2 => "Android 5.1 Lollipop MR1",
            3 => "Android 6.x Marshmallow",
            4 => "Android 7.x / 8.x Nougat-Oreo",
            _ => "unknown Android release",
        }
    }

    /// Second header line: the number of blocks the list is expected to cover.
    pub fn total_blocks(&self) -> u64 {
        self.total_blocks
    }

    /// Scratch slots the list needs simultaneously (0 before version 2).
    pub fn stash_slots(&self) -> u64 {
        self.stash_slots
    }

    /// Blocks the list stashes at once (0 before version 2).
    pub fn stash_blocks(&self) -> u64 {
        self.stash_blocks
    }

    /// The file this list was read from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The commands, in file order.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Every range of every command, in file order.
    pub fn ranges(&self) -> impl Iterator<Item = &BlockRange> {
        self.commands
            .iter()
            .flat_map(|command| command.ranges.iter())
    }

    /// Blocks that `new` commands pull out of the payload.
    pub fn payload_blocks(&self) -> u64 {
        self.commands
            .iter()
            .filter(|command| command.kind.reads_payload())
            .map(Command::blocks)
            .sum()
    }

    /// Blocks covered by all commands, `new` included.
    pub fn covered_blocks(&self) -> u64 {
        self.commands.iter().map(Command::blocks).sum()
    }

    /// Size of the image the list describes, in blocks: the highest block any
    /// command touches. Blocks below it that no command mentions stay zero.
    pub fn image_blocks(&self) -> u64 {
        self.ranges().map(|range| range.end).max().unwrap_or(0)
    }

    /// Size of the image the list describes, in bytes.
    pub fn image_bytes(&self) -> u64 {
        self.image_blocks() * BLOCK_SIZE
    }

    /// Cross-check the header, the commands and — when it is known ahead of
    /// time — the size of the payload.
    ///
    /// Catches the mistakes that otherwise produce a plausible looking but
    /// wrong image: a header that disagrees with the commands, overlapping
    /// commands (which matters because `zero` is implemented by leaving a hole)
    /// and a payload of the wrong length.
    pub fn verify(&self, payload_bytes: Option<u64>) -> Result<()> {
        if self.commands.is_empty() {
            return self.fail("the transfer list contains no commands");
        }

        let covered = self.covered_blocks();
        if covered != self.total_blocks {
            return self.fail(format!(
                "the header expects {} blocks but the commands cover {covered}",
                self.total_blocks
            ));
        }

        let mut ranges: Vec<BlockRange> = self.ranges().copied().collect();
        ranges.sort_unstable();
        for pair in ranges.windows(2) {
            if pair[1].start < pair[0].end {
                return self.fail(format!(
                    "blocks {},{start} and {},{end} overlap",
                    pair[0].start,
                    pair[0].end,
                    start = pair[1].start,
                    end = pair[1].end,
                ));
            }
        }

        if let Some(bytes) = payload_bytes {
            let expected = self.payload_blocks() * BLOCK_SIZE;
            if bytes != expected {
                return self.fail(format!(
                    "the payload holds {bytes} bytes but the `new` commands need {expected}"
                ));
            }
        }

        Ok(())
    }

    fn fail(&self, message: impl Into<String>) -> Result<()> {
        Err(Error::Verify {
            path: self.path.clone(),
            message: message.into(),
        })
    }
}

/// Parse the `10,2,6,7,…` part of a command: a count followed by that many
/// block numbers, which pair up into `start,end` ranges.
fn parse_ranges(path: &Path, line: usize, spec: &str) -> Result<Vec<BlockRange>> {
    let mut numbers = Vec::new();
    for token in spec.split(',') {
        let token = token.trim();
        let block: u64 = token
            .parse()
            .map_err(|_| Error::parse(path, line, format!("`{token}` is not a block number")))?;
        numbers.push(block);
    }

    let (count, pairs) = numbers.split_first().expect("a split never yields nothing");
    let count = *count;
    if count != pairs.len() as u64 {
        return Err(Error::parse(
            path,
            line,
            format!(
                "the range set declares {count} numbers but carries {}",
                pairs.len()
            ),
        ));
    }
    if count == 0 || count % 2 != 0 {
        return Err(Error::parse(
            path,
            line,
            format!("the range set needs a non-zero, even number of block numbers, found {count}"),
        ));
    }

    let mut ranges = Vec::with_capacity(pairs.len() / 2);
    for pair in pairs.as_chunks::<2>().0 {
        let [start, end] = *pair;
        if start >= end {
            return Err(Error::parse(
                path,
                line,
                format!("`{start},{end}` is not a usable block range"),
            ));
        }
        ranges.push(BlockRange { start, end });
    }

    Ok(ranges)
}

/// A line reader that keeps track of line numbers and normalises line endings,
/// so that both `\n` and `\r\n` files parse the same way.
struct LineReader<'a, R> {
    path: &'a Path,
    lines: Lines<R>,
    line_no: usize,
}

impl<'a, R: BufRead> LineReader<'a, R> {
    fn new(path: &'a Path, reader: R) -> Self {
        Self {
            path,
            lines: reader.lines(),
            line_no: 0,
        }
    }

    /// The 1-based number of the line most recently returned.
    fn line_no(&self) -> usize {
        self.line_no
    }

    /// The next non-blank line, trimmed.
    fn next(&mut self) -> Result<Option<String>> {
        for line in self.lines.by_ref() {
            self.line_no += 1;
            let line = line.map_err(|e| {
                Error::parse(self.path, self.line_no, format!("not valid UTF-8 ({e})"))
            })?;
            let line = line.trim();
            if !line.is_empty() {
                return Ok(Some(line.to_owned()));
            }
        }
        Ok(None)
    }

    /// The next non-blank line, or an error saying `what` was expected instead.
    fn required(&mut self, what: &str) -> Result<String> {
        self.next()?
            .ok_or_else(|| self.error(format!("unexpected end of file, expected {what}")))
    }

    /// The next non-blank line, parsed as a number.
    fn number(&mut self, what: &str) -> Result<u64> {
        let line = self.required(what)?;
        line.parse()
            .map_err(|_| self.error(format!("expected {what}, found `{line}`")))
    }

    fn error(&self, message: impl Into<String>) -> Error {
        Error::parse(self.path, self.line_no, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<TransferList> {
        TransferList::parse(text.as_bytes())
    }

    /// The transfer list from `eg/`, header included.
    const SAMPLE: &str = "\
4
674
0
0
new 10,2,6,7,15,519,520,641,655,673,674
new 2,0,1
new 2,520,641
new 2,30,519
new 2,15,30
new 2,6,7
new 2,1,2
zero 2,655,673
";

    #[test]
    fn parses_the_header_and_commands() {
        let list = parse(SAMPLE).unwrap();
        assert_eq!(list.version(), 4);
        assert_eq!(list.android_release(), "Android 7.x / 8.x Nougat-Oreo");
        assert_eq!(list.total_blocks(), 674);
        assert_eq!(list.stash_slots(), 0);
        assert_eq!(list.stash_blocks(), 0);
        assert_eq!(list.commands().len(), 8);
        assert_eq!(list.commands()[0].kind, CommandKind::New);
        assert_eq!(list.commands()[0].ranges.len(), 5);
        assert_eq!(list.commands()[7].kind, CommandKind::Zero);
        assert_eq!(list.payload_blocks(), 656);
        assert_eq!(list.covered_blocks(), 674);
        assert_eq!(list.image_blocks(), 674);
        assert_eq!(list.image_bytes(), 674 * BLOCK_SIZE);
        assert_eq!(list.ranges().count(), 12);
        list.verify(Some(656 * BLOCK_SIZE)).unwrap();
    }

    #[test]
    fn version_1_has_no_stash_lines() {
        let list = parse("1\n2\nnew 2,0,2\n").unwrap();
        assert_eq!(list.version(), 1);
        assert_eq!(list.stash_slots(), 0);
        assert_eq!(list.commands().len(), 1);
    }

    #[test]
    fn a_command_renders_back_to_its_original_line() {
        for line in ["new 10,2,6,7,15,519,520,641,655,673,674", "zero 2,655,673"] {
            let list = parse(&format!("4\n674\n0\n0\n{line}\n")).unwrap();
            assert_eq!(list.commands()[0].to_string(), line);
        }
    }

    #[test]
    fn every_sample_command_round_trips() {
        let list = parse(SAMPLE).unwrap();
        let rendered: Vec<String> = list.commands().iter().map(Command::to_string).collect();
        let original: Vec<&str> = SAMPLE.lines().skip(4).collect();
        assert_eq!(rendered, original);
    }

    #[test]
    fn tolerates_crlf_blank_lines_and_stray_numeric_lines() {
        let list = parse("4\r\n674\r\n0\r\n0\r\n\r\nnew 2,0,674\r\n674\r\n").unwrap();
        assert_eq!(list.commands().len(), 1);
        assert_eq!(list.image_blocks(), 674);
    }

    #[test]
    fn rejects_unknown_commands() {
        let err = parse("4\n2\n0\n0\nmove 2,0,2\n").unwrap_err();
        assert!(
            matches!(err, Error::UnsupportedCommand { line: 5, ref command, .. } if command == "move"),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_a_range_set_that_miscounts_itself() {
        let err = parse("4\n2\n0\n0\nnew 4,0,2\n").unwrap_err();
        assert!(
            err.to_string().contains("declares 4 numbers but carries 2"),
            "{err}"
        );
    }

    #[test]
    fn rejects_odd_count_range_sets() {
        let err = parse("4\n2\n0\n0\nnew 1,0\n").unwrap_err();
        assert!(err.to_string().contains("even number"), "{err}");
    }

    #[test]
    fn rejects_empty_ranges() {
        let err = parse("4\n2\n0\n0\nnew 2,7,7\n").unwrap_err();
        assert!(
            err.to_string().contains("not a usable block range"),
            "{err}"
        );
    }

    #[test]
    fn rejects_an_unsupported_version() {
        let err = parse("9\n2\nnew 2,0,2\n").unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported transfer list version"),
            "{err}"
        );
    }

    #[test]
    fn reports_the_line_of_a_malformed_header() {
        let err = parse("4\n674\nnot a number\n").unwrap_err();
        assert!(err.to_string().starts_with("<transfer list>:3:"), "{err}");
    }

    #[test]
    fn a_truncated_header_is_an_error() {
        let err = parse("4\n").unwrap_err();
        assert!(err.to_string().contains("unexpected end of file"), "{err}");
    }

    #[test]
    fn verify_rejects_a_header_that_disagrees_with_the_commands() {
        let list = parse("4\n100\n0\n0\nnew 2,0,2\n").unwrap();
        let err = list.verify(None).unwrap_err();
        assert!(
            err.to_string()
                .contains("expects 100 blocks but the commands cover 2"),
            "{err}"
        );
    }

    #[test]
    fn verify_rejects_overlapping_commands() {
        let list = parse("4\n5\n0\n0\nnew 2,0,3\nnew 2,2,4\n").unwrap();
        let err = list.verify(None).unwrap_err();
        assert!(err.to_string().contains("overlap"), "{err}");
    }

    #[test]
    fn verify_rejects_a_payload_of_the_wrong_size() {
        let list = parse(SAMPLE).unwrap();
        let err = list.verify(Some(1)).unwrap_err();
        assert!(
            err.to_string().contains("the payload holds 1 bytes"),
            "{err}"
        );
    }

    #[test]
    fn verify_rejects_an_empty_list() {
        let list = parse("4\n0\n0\n0\n").unwrap();
        let err = list.verify(None).unwrap_err();
        assert!(err.to_string().contains("no commands"), "{err}");
    }
}
