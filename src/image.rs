//! Laying a payload out into a raw partition image.

use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use crate::error::{Error, Result};
use crate::transfer_list::{BLOCK_SIZE, CommandKind, TransferList};

/// Bytes copied per read/write pair. A multiple of [`BLOCK_SIZE`], so copies
/// never straddle a block boundary.
const COPY_CHUNK: usize = 1 << 20;

/// A seekable destination that can be sized up front.
///
/// Sizing first is what makes the output sparse: blocks that no command writes
/// stay holes, and hole reads as zero on every filesystem we care about.
pub trait ImageSink: Write + Seek {
    /// Grow or shrink the destination to exactly `len` bytes.
    fn set_len(&mut self, len: u64) -> io::Result<()>;
}

impl ImageSink for File {
    fn set_len(&mut self, len: u64) -> io::Result<()> {
        File::set_len(self, len)
    }
}

impl ImageSink for Cursor<Vec<u8>> {
    fn set_len(&mut self, len: u64) -> io::Result<()> {
        let len = usize::try_from(len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "image too large for an in-memory sink",
            )
        })?;
        self.get_mut().resize(len, 0);
        Ok(())
    }
}

/// What [`write_image`] did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImageStats {
    /// Blocks copied out of the payload by `new` commands.
    pub blocks_copied: u64,
    /// Blocks `zero` and `erase` commands left untouched.
    pub blocks_zeroed: u64,
    /// Bytes pulled out of the payload, i.e. its decompressed size.
    pub payload_bytes: u64,
    /// Size of the finished image.
    pub image_bytes: u64,
}

/// Write `payload` into `out` following `list`.
///
/// Commands are applied in file order. `new` reads the next blocks out of the
/// payload and writes them at the target position; `zero` and `erase` need no
/// work at all, because `out` is sized beforehand and their blocks were never
/// written — which is only sound while the commands do not overlap, something
/// [`TransferList::verify`] rejects.
///
/// With `verify`, the payload has to be used up exactly: no short reads, no
/// leftovers.
///
/// `progress` is called with the number of finished blocks as the image fills
/// in, at most once per [`COPY_CHUNK`].
pub fn write_image(
    list: &TransferList,
    payload: &mut impl Read,
    out: &mut impl ImageSink,
    verify: bool,
    mut progress: impl FnMut(u64),
) -> Result<ImageStats> {
    let image_bytes = list.image_bytes();
    out.set_len(image_bytes)
        .map_err(|e| Error::stream("resizing the output image", e))?;

    let mut stats = ImageStats {
        image_bytes,
        ..ImageStats::default()
    };
    let mut buffer = vec![0u8; COPY_CHUNK];
    let mut done = 0;

    for command in list.commands() {
        for range in command.ranges() {
            match command.kind {
                CommandKind::New => {
                    out.seek(SeekFrom::Start(range.byte_offset()))
                        .map_err(|e| Error::stream("seeking in the output image", e))?;

                    let mut remaining = range.byte_len();
                    while remaining > 0 {
                        let chunk = remaining.min(COPY_CHUNK as u64) as usize;
                        payload.read_exact(&mut buffer[..chunk]).map_err(|e| {
                            if e.kind() == io::ErrorKind::UnexpectedEof {
                                Error::TruncatedPayload {
                                    found: stats.payload_bytes / BLOCK_SIZE,
                                    expected: list.payload_blocks(),
                                }
                            } else {
                                Error::stream("reading the payload", e)
                            }
                        })?;
                        out.write_all(&buffer[..chunk])
                            .map_err(|e| Error::stream("writing the output image", e))?;

                        remaining -= chunk as u64;
                        stats.payload_bytes += chunk as u64;
                        stats.blocks_copied += chunk as u64 / BLOCK_SIZE;
                        done += chunk as u64 / BLOCK_SIZE;
                        progress(done);
                    }
                }
                CommandKind::Zero | CommandKind::Erase => {
                    stats.blocks_zeroed += range.len();
                    done += range.len();
                    progress(done);
                }
            }
        }
    }

    if verify {
        check_payload_exhausted(list, payload)?;
    }

    Ok(stats)
}

/// Make sure the payload did not carry more than the transfer list asked for.
/// For a brotli payload this also decodes the stream to its end, which is what
/// catches a stream that is corrupt past the last block we needed.
fn check_payload_exhausted(list: &TransferList, payload: &mut impl Read) -> Result<()> {
    let mut probe = [0u8; COPY_CHUNK];
    let mut extra = 0u64;
    loop {
        match payload.read(&mut probe) {
            Ok(0) => break,
            Ok(read) => extra += read as u64,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Error::stream("reading the payload", e)),
        }
    }

    if extra > 0 {
        return Err(Error::Verify {
            path: list.path().to_path_buf(),
            message: format!(
                "the payload carries {extra} bytes more than the {} blocks the transfer list needs",
                list.payload_blocks()
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn blocks(count: usize, byte: u8) -> Vec<u8> {
        (0..count)
            .flat_map(|_| std::iter::repeat_n(byte, BLOCK_SIZE as usize))
            .collect()
    }

    /// `new 0..2`, `zero 2..3`, `new 3..4` — one hole in the middle.
    const LIST: &str = "4\n3\n0\n0\nnew 2,0,2\nzero 2,2,3\nnew 2,3,4\n";

    fn list() -> TransferList {
        TransferList::parse(LIST.as_bytes()).unwrap()
    }

    #[test]
    fn blocks_land_where_the_transfer_list_says() {
        let payload = [blocks(2, 0xaa), blocks(1, 0xbb)].concat();
        let mut out = Cursor::new(Vec::new());

        let stats = write_image(&list(), &mut &payload[..], &mut out, true, |_| {}).unwrap();

        assert_eq!(stats.image_bytes, 4 * BLOCK_SIZE);
        assert_eq!(stats.blocks_copied, 3);
        assert_eq!(stats.blocks_zeroed, 1);
        assert_eq!(stats.payload_bytes, 3 * BLOCK_SIZE);
        assert_eq!(
            out.into_inner(),
            [blocks(2, 0xaa), blocks(1, 0), blocks(1, 0xbb)].concat()
        );
    }

    #[test]
    fn progress_counts_every_block_once() {
        let payload = [blocks(2, 1), blocks(1, 2)].concat();
        let mut out = Cursor::new(Vec::new());
        let mut seen = Vec::new();

        write_image(&list(), &mut &payload[..], &mut out, true, |done| {
            seen.push(done)
        })
        .unwrap();

        assert_eq!(seen, vec![2, 3, 4]);
    }

    #[test]
    fn a_short_payload_is_rejected() {
        let payload = blocks(2, 0xaa); // one block short
        let mut out = Cursor::new(Vec::new());

        let err = write_image(&list(), &mut &payload[..], &mut out, true, |_| {}).unwrap_err();

        assert!(
            matches!(
                err,
                Error::TruncatedPayload {
                    found: 2,
                    expected: 3
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn a_long_payload_is_rejected_when_verifying() {
        let payload = [blocks(2, 0), blocks(1, 0), blocks(1, 0)].concat();
        let mut out = Cursor::new(Vec::new());

        let err = write_image(&list(), &mut &payload[..], &mut out, true, |_| {}).unwrap_err();
        assert!(err.to_string().contains("more than the 3 blocks"), "{err}");

        // ... and tolerated when the caller knows what it is doing.
        let mut out = Cursor::new(Vec::new());
        write_image(&list(), &mut &payload[..], &mut out, false, |_| {}).unwrap();
    }

    #[test]
    fn a_single_new_range_spanning_several_chunks() {
        // 1 MiB + one block, so the copy loop runs twice.
        let count = COPY_CHUNK / BLOCK_SIZE as usize + 1;
        let list =
            TransferList::parse(format!("4\n{count}\n0\n0\nnew 2,0,{count}\n").as_bytes()).unwrap();
        let payload = blocks(count, 0x5a);
        let mut out = Cursor::new(Vec::new());

        let stats = write_image(&list, &mut &payload[..], &mut out, true, |_| {}).unwrap();

        assert_eq!(stats.blocks_copied, count as u64);
        assert_eq!(out.into_inner(), payload);
    }

    #[test]
    fn the_output_is_sized_before_anything_is_written() {
        // Nothing is ever written, but the image still has its full size.
        let list = TransferList::parse(&b"4\n2\n0\n0\nzero 2,0,2\n"[..]).unwrap();
        let mut out = Cursor::new(Vec::new());

        let stats = write_image(&list, &mut &b""[..], &mut out, true, |_| {}).unwrap();

        assert_eq!(stats.blocks_zeroed, 2);
        assert_eq!(stats.blocks_copied, 0);
        assert_eq!(out.into_inner().len(), 2 * BLOCK_SIZE as usize);
    }
}
