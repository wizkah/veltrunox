//! Stem files and stem packages (ADR-015): reading a ZIP package into the stem directory
//! and writing one from it.

use crate::audio::{append_mono, decode_blocks};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
};
use veltrunox_core::{MAX_STEMS, Millis, Section, StemName, WaveformBuilder};
use veltrunox_protocol::{
    MANIFEST_FILE, PACKAGE_FORMAT, PackageAnalysisDto, PackageManifestDto, PackageRecordingDto,
    PackageStemDto,
};
use veltrunox_server_app::{Stem, StemRepository, StemSet, StorageError, Structure};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

const TEMPORARY_PREFIX: &str = ".stem-";
const MAX_MANIFEST_BYTES: u64 = 8 << 20;
/// Audio formats a package may carry.
const EXTENSIONS: [&str; 4] = ["ogg", "mp3", "flac", "m4a"];

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("not a stem package: {0}")]
    Invalid(String),
    #[error("cannot store the stems: {0}")]
    Io(#[from] io::Error),
}

fn invalid(detail: impl std::fmt::Display) -> PackageError {
    PackageError::Invalid(detail.to_string())
}

/// Directory of the stem files, named by the hash of their content, and the size limit of
/// one package and of each file in it.
#[derive(Debug)]
pub struct StemStore {
    directory: PathBuf,
    max_bytes: u64,
    /// Held while files are added or removed, so that no sweep takes files that are about to
    /// be referenced.
    writing: Mutex<()>,
}

/// A file that is removed when dropped, unless it was moved away.
pub struct TemporaryFile(PathBuf);

impl TemporaryFile {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// A package that was read and checked; its files wait under temporary names.
pub struct PreparedPackage {
    pub content_hash: String,
    pub structure: Structure,
    analysis: Option<String>,
    stems: Vec<(Stem, TemporaryFile)>,
}

impl StemStore {
    /// Creates the directory if needed and removes what no recording refers to.
    pub fn open(
        directory: &Path,
        max_bytes: u64,
        stems: &dyn StemRepository,
    ) -> Result<Self, StorageError> {
        fs::create_dir_all(directory).map_err(|error| StorageError(error.to_string()))?;
        let store = Self {
            directory: directory.to_owned(),
            max_bytes,
            writing: Mutex::new(()),
        };
        store.remove_temporaries()?;
        store.sweep(stems)?;
        Ok(store)
    }

    /// What an interrupted import or analysis left behind.
    fn remove_temporaries(&self) -> Result<(), StorageError> {
        let entries =
            fs::read_dir(&self.directory).map_err(|error| StorageError(error.to_string()))?;
        for path in entries.flatten().map(|entry| entry.path()) {
            if is_temporary(&path) {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub fn lock(&self) -> MutexGuard<'_, ()> {
        self.writing.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Removes the stored stem files no recording refers to. Temporary files stay: they
    /// belong to running work.
    pub fn sweep(&self, stems: &dyn StemRepository) -> Result<(), StorageError> {
        let _guard = self.lock();
        let referenced = stems.stem_paths()?;
        let entries =
            fs::read_dir(&self.directory).map_err(|error| StorageError(error.to_string()))?;
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_file() && !is_temporary(&path) && !referenced.contains(&path) {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    pub fn temporary_file(&self, extension: &str) -> TemporaryFile {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TemporaryFile(self.directory.join(format!(
            "{TEMPORARY_PREFIX}{}-{}.{extension}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )))
    }

    /// Reads the package at `path`: checks the manifest, unpacks every stem and decodes it
    /// once for its length and waveform. Nothing is kept unless `keep` is called.
    pub fn read(&self, path: &Path) -> Result<PreparedPackage, PackageError> {
        let mut archive = ZipArchive::new(File::open(path)?).map_err(invalid)?;
        let manifest = read_manifest(&mut archive)?;
        let mut stems = Vec::new();
        for entry in &manifest.stems {
            let name = StemName::new(&entry.name).map_err(invalid)?;
            let extension = extension_of(&entry.file)?;
            let temporary = self.temporary_file(extension);
            let source = archive.by_name(&entry.file).map_err(invalid)?;
            let hash = copy_limited(source, temporary.path(), self.max_bytes)?;
            let (length, waveform) = measure(temporary.path())?;
            let stem = Stem {
                name,
                path: self.directory.join(format!("{hash}.{extension}")),
                length,
                waveform,
            };
            stems.push((stem, temporary));
        }
        let first_rate = stems.first().map(|(stem, _)| stem.waveform.sample_rate);
        if stems
            .iter()
            .any(|(stem, _)| Some(stem.waveform.sample_rate) != first_rate)
        {
            return Err(invalid("the stems have different sample rates"));
        }
        let analysis = manifest
            .analysis
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(invalid)?;
        Ok(PreparedPackage {
            content_hash: manifest.recording.content_hash,
            structure: manifest.analysis.map(structure_of).unwrap_or_default(),
            analysis,
            stems,
        })
    }

    /// Moves the files of a read package into place and returns the set that refers to them.
    /// The caller holds `lock` until the set is stored.
    pub fn keep(&self, package: &PreparedPackage) -> Result<StemSet, PackageError> {
        for (stem, temporary) in &package.stems {
            fs::copy(temporary.path(), &stem.path)?;
        }
        Ok(StemSet {
            stems: package.stems.iter().map(|(stem, _)| stem.clone()).collect(),
            analysis: package.analysis.clone(),
        })
    }

    /// Writes the package of a stored set to a temporary file.
    pub fn write(
        &self,
        recording: PackageRecordingDto,
        set: &StemSet,
    ) -> Result<TemporaryFile, PackageError> {
        let target = self.temporary_file("zip");
        let mut writer = ZipWriter::new(File::create(target.path())?);
        // The audio is compressed already.
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let entries: Vec<PackageStemDto> = set
            .stems
            .iter()
            .enumerate()
            .map(|(index, stem)| PackageStemDto {
                name: stem.name.as_str().to_owned(),
                file: format!(
                    "stems/{index}.{}",
                    stem.path
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .unwrap_or("ogg")
                ),
            })
            .collect();
        for (stem, entry) in set.stems.iter().zip(&entries) {
            writer
                .start_file(entry.file.as_str(), options)
                .map_err(invalid)?;
            io::copy(&mut File::open(&stem.path)?, &mut writer)?;
        }
        let manifest = PackageManifestDto {
            format_version: PACKAGE_FORMAT,
            recording,
            stems: entries,
            analysis: set
                .analysis
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(invalid)?,
        };
        writer.start_file(MANIFEST_FILE, options).map_err(invalid)?;
        writer.write_all(&serde_json::to_vec_pretty(&manifest).map_err(invalid)?)?;
        writer.finish().map_err(invalid)?;
        Ok(target)
    }
}

fn is_temporary(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(TEMPORARY_PREFIX))
}

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<PackageManifestDto, PackageError> {
    let entry = archive.by_name(MANIFEST_FILE).map_err(invalid)?;
    let manifest: PackageManifestDto =
        serde_json::from_reader(entry.take(MAX_MANIFEST_BYTES)).map_err(invalid)?;
    if manifest.format_version != PACKAGE_FORMAT {
        return Err(invalid("unknown format version"));
    }
    if manifest.stems.is_empty() || manifest.stems.len() > MAX_STEMS {
        return Err(invalid(format!("a package has 1 to {MAX_STEMS} stems")));
    }
    Ok(manifest)
}

fn extension_of(file: &str) -> Result<&'static str, PackageError> {
    let extension = Path::new(file)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    EXTENSIONS
        .into_iter()
        .find(|known| Some(*known) == extension.as_deref())
        .ok_or_else(|| invalid("stems are Ogg, MP3, FLAC or M4A files"))
}

/// Copies at most `limit` bytes to `target` and returns the SHA-256 of them, hex encoded.
fn copy_limited(mut source: impl Read, target: &Path, limit: u64) -> Result<String, PackageError> {
    let mut file = File::create(target)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1 << 16];
    let mut copied = 0_u64;
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        copied += read as u64;
        if copied > limit {
            return Err(invalid("a stem is larger than the upload limit"));
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read])?;
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Length and waveform of a stem file.
fn measure(path: &Path) -> Result<(Millis, veltrunox_core::Waveform), PackageError> {
    let mut builder: Option<WaveformBuilder> = None;
    let mut mono = Vec::new();
    let mut frames = 0_u64;
    let rate = decode_blocks(path, &|_| {}, &mut |rate, interleaved, channels| {
        mono.clear();
        append_mono(&mut mono, interleaved, channels);
        frames += mono.len() as u64;
        builder
            .get_or_insert_with(|| WaveformBuilder::new(rate))
            .push(&mono);
    })
    .map_err(invalid)?;
    let waveform = builder.ok_or_else(|| invalid("empty stem"))?.finish();
    Ok((Millis::new(frames * 1_000 / u64::from(rate)), waveform))
}

fn structure_of(analysis: PackageAnalysisDto) -> Structure {
    Structure {
        sections: analysis
            .sections
            .into_iter()
            .map(|section| Section {
                start: Millis::new(section.start_ms),
                end: Millis::new(section.end_ms),
                label: section.label,
            })
            .collect(),
        downbeats: analysis.downbeats_ms.into_iter().map(Millis::new).collect(),
    }
}
