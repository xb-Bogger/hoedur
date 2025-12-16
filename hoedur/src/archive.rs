use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use archive::{Archive, ArchiveBuilder};
use tar::EntryType;

use crate::cli;

pub fn create_archive(name: &str, archive_dir: &Path) -> Result<ArchiveBuilder> {
    archive::create_archive(archive_dir, name, false, false).map(ArchiveBuilder::from)
}

pub fn opt_archive(archive: &cli::Archive) -> Option<&Path> {
    archive
        .write_archive
        .then_some(&archive.archive_dir)
        .map(|archive_dir| archive_dir.archive_dir.as_ref())
}

pub fn archive_file_path(archive_dir: &Path, name: &str) -> PathBuf {
    archive_dir.join(format!("{name}.corpus.tar.zst"))
}

pub fn export_archive_to_dir(archive_path: &Path, export_dir: &Path) -> Result<()> {
    fs::create_dir_all(export_dir).context("Failed to create export dir")?;
    let mut archive = Archive::from_reader(
        common::fs::decoder(archive_path).with_context(|| format!("open archive {archive_path:?}"))?,
    );
    let it = archive.iter::<archive::common::CommonEntryKind>()?;
    for result in it {
        let mut entry = result.context("Failed to read tar entry")?;
        let path = entry
            .raw_entry()
            .path()
            .context("entry path")?
            .to_path_buf();
        let ty = entry.header().entry_type();
        let out = export_dir.join(path);
        match ty {
            EntryType::Directory => {
                fs::create_dir_all(&out).with_context(|| format!("mkdir {:?}", out))?;
            }
            EntryType::Regular => {
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
                }
                let mut f = fs::File::create(&out).with_context(|| format!("create {:?}", out))?;
                std::io::copy(entry.raw_entry(), &mut f)
                    .with_context(|| format!("write {:?}", out))?;
            }
            _ => {
                // skip other types
            }
        }
    }
    Ok(())
}
