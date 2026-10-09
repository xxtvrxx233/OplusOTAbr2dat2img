//! The `*.new.dat` payload every transfer list draws its `new` blocks from.

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Buffer size used both for reading the file and for handing decompressed
/// output back to the caller.
const BUFFER_SIZE: usize = 64 * 1024;

/// A payload on disk: brotli-compressed (`*.new.dat.br`) or raw (`*.new.dat`).
///
/// Nothing is read until [`NewData::reader`] is called, and what it returns is a
/// stream, so a 4 GiB payload never has to fit in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewData {
    path: PathBuf,
    compressed: bool,
}

impl NewData {
    /// Open the payload at `path`, deciding how to decode it from the file
    /// name: `*.br` is brotli, everything else is a raw `.new.dat`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let compressed = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("br"));
        Self { path, compressed }
    }

    /// Treat `path` as an uncompressed `.new.dat`.
    pub fn raw(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            compressed: false,
        }
    }

    /// Treat `path` as a brotli-compressed `.new.dat.br`.
    pub fn brotli(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            compressed: true,
        }
    }

    /// Where the payload lives.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the payload has to be decompressed.
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }

    /// Size of the file on disk — the compressed size for a `.br` payload.
    pub fn file_len(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.path).map_err(|e| Error::reading(&self.path, e))?;
        Ok(metadata.len())
    }

    /// Size of the payload once decompressed, when it can be known without
    /// decompressing it. That is only the case for raw payloads.
    pub fn known_payload_len(&self) -> Result<Option<u64>> {
        if self.compressed {
            Ok(None)
        } else {
            self.file_len().map(Some)
        }
    }

    /// Open a streaming reader over the *decompressed* payload.
    pub fn reader(&self) -> Result<Box<dyn Read>> {
        let file = File::open(&self.path).map_err(|e| Error::reading(&self.path, e))?;
        let file = BufReader::with_capacity(BUFFER_SIZE, file);
        Ok(if self.compressed {
            let decoder = brotli::Decompressor::new(file, BUFFER_SIZE);
            Box::new(NamedDecodeErrors::new(decoder, self.path.clone()))
        } else {
            Box::new(file)
        })
    }

    /// Read the whole payload into memory.
    ///
    /// Handy for small payloads and tests; [`Job`](crate::Job) streams instead.
    pub fn read_all(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        self.reader()?
            .read_to_end(&mut data)
            .map_err(|e| Error::stream("reading the payload", e))?;
        Ok(data)
    }
}

/// Adds the file name to the brotli decoder's rather terse `InvalidData`
/// errors, so that a corrupt payload says *which* payload is corrupt.
struct NamedDecodeErrors<R> {
    inner: R,
    path: PathBuf,
}

impl<R> NamedDecodeErrors<R> {
    fn new(inner: R, path: PathBuf) -> Self {
        Self { inner, path }
    }
}

impl<R: Read> Read for NamedDecodeErrors<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map_err(|e| {
            if e.kind() == io::ErrorKind::InvalidData {
                io::Error::new(
                    e.kind(),
                    format!("corrupt brotli stream in `{}`", self.path.display()),
                )
            } else {
                e
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Compress `data` the way the OTA packer does.
    fn brotli(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut out, 4096, 5, 22);
            writer.write_all(data).unwrap();
        }
        out
    }

    /// Write `bytes` to a fresh temporary directory, which is deleted on drop.
    fn staged(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    #[test]
    fn compression_is_detected_from_the_extension() {
        assert!(NewData::new("system.new.dat.br").is_compressed());
        assert!(NewData::new("system.new.dat.BR").is_compressed());
        assert!(!NewData::new("system.new.dat").is_compressed());
        assert!(!NewData::new("system.img").is_compressed());
        assert!(NewData::brotli("payload").is_compressed());
        assert!(!NewData::raw("payload").is_compressed());
    }

    #[test]
    fn a_raw_payload_is_read_back_verbatim() {
        let data: Vec<u8> = (0..=255).cycle().take(10_000).collect();
        let (_dir, path) = staged("raw.new.dat", &data);

        let payload = NewData::new(&path);
        assert_eq!(payload.file_len().unwrap(), data.len() as u64);
        assert_eq!(
            payload.known_payload_len().unwrap(),
            Some(data.len() as u64)
        );
        assert_eq!(payload.read_all().unwrap(), data);
    }

    #[test]
    fn a_brotli_payload_is_decompressed() {
        let data: Vec<u8> = (0..=255).cycle().take(100_000).collect();
        let (_dir, path) = staged("packed.new.dat.br", &brotli(&data));

        let payload = NewData::new(&path);
        assert_eq!(payload.read_all().unwrap(), data);
        // The decompressed size is only known once the stream has been decoded.
        assert_eq!(payload.known_payload_len().unwrap(), None);
        assert!(payload.file_len().unwrap() < data.len() as u64);
    }

    #[test]
    fn a_corrupt_brotli_payload_names_the_file() {
        let (_dir, path) = staged("corrupt.new.dat.br", &[0xff; 4]);

        let err = NewData::new(&path).read_all().unwrap_err();
        let message = err.to_string();
        assert!(message.contains("corrupt brotli stream"), "{message}");
        assert!(message.contains("corrupt.new.dat.br"), "{message}");
    }

    #[test]
    fn a_truncated_brotli_payload_is_reported() {
        let data: Vec<u8> = (0..=255).cycle().take(100_000).collect();
        let mut packed = brotli(&data);
        packed.truncate(packed.len() / 2);
        let (_dir, path) = staged("truncated.new.dat.br", &packed);

        let err = NewData::new(&path).read_all().unwrap_err();
        assert!(err.to_string().contains("truncated.new.dat.br"), "{err}");
    }

    #[test]
    fn a_missing_payload_is_reported_with_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.new.dat");

        let err = NewData::new(&path).read_all().unwrap_err();
        assert!(err.to_string().contains("nope.new.dat"), "{err}");
    }
}
