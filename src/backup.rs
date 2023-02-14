use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use zip::result::ZipError;
use zip::write::FileOptions;

use std::fs::File;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

// zip = "0.6.4"
// Used from example in zip-rs:
// https://github.com/zip-rs/zip/blob/master/examples/write_dir.rs
pub(crate) fn create_backup(src_dir: &Path, dst_file: &Path) -> zip::result::ZipResult<()> {
    
    let method: zip::CompressionMethod = METHOD_DEFLATED.unwrap();
    
    if !src_dir.is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walk_dir = WalkDir::new(src_dir);
    let it = walk_dir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir.to_str().unwrap(), file, method)?;

    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item=DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
    where
        T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and map name conversion failed error on unzip
            println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

// --| Tests ------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Test that a backup zip file is created
    #[test]
    fn test_create_backup() {
        let dir = tempdir().unwrap();
        let mut path = PathBuf::new();
        path.push(dir.path());
        path.push("test.txt");
        
        let mut file = File::create(path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let mut backup_path = PathBuf::new();
        backup_path.push(dir.path());
        backup_path.push("backup.zip");

        create_backup(dir.path(), backup_path.as_path()).unwrap();
        
        assert!(backup_path.exists());
        dir.close().unwrap();
    }
}
