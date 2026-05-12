use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Archive as TarArchive;

pub struct ArchiveService {
    server_root: PathBuf,
    temp_dir: PathBuf,
    max_archive_size: usize,
}

impl ArchiveService {
    pub fn new(server_root: PathBuf) -> Self {
        let temp_dir = server_root.join(".temp");
        fs::create_dir_all(&temp_dir).ok();
        
        Self {
            server_root,
            temp_dir,
            max_archive_size: 500 * 1024 * 1024,
        }
    }

    pub fn create_archive(&self, request: &ArchiveRequest) -> Result<ArchiveResult> {
        let output_path = self.server_root.join(&request.output_name);
        
        let extension = match request.format {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::TarGz => "tar.gz",
        };

        let final_path = if !request.output_name.ends_with(&format!(".{}", extension)) {
            output_path.with_extension(extension)
        } else {
            output_path
        };

        match request.format {
            ArchiveFormat::Zip => self.create_zip(&request.files, &final_path)?,
            ArchiveFormat::Tar => self.create_tar(&request.files, &final_path)?,
            ArchiveFormat::TarGz => self.create_tar_gz(&request.files, &final_path)?,
        }

        let metadata = fs::metadata(&final_path)?;
        
        Ok(ArchiveResult {
            archive_path: final_path.to_string_lossy().to_string(),
            total_size: metadata.len(),
            files_included: request.files.len(),
            format: format!("{:?}", request.format),
        })
    }

    fn create_zip(&self, files: &[String], output: &Path) -> Result<()> {
        let file = File::create(output)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for file_path in files {
            let full_path = self.resolve_path(file_path)?;
            
            if full_path.is_dir() {
                self.add_directory_to_zip(&mut zip, &full_path, &full_path, options)?;
            } else {
                self.add_file_to_zip(&mut zip, &full_path, &full_path, options)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    fn add_directory_to_zip(&self, zip: &mut ZipWriter<File>, base_path: &Path, current: &Path, options: SimpleFileOptions) -> Result<()> {
        for entry in WalkDir::new(current).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(base_path.parent().unwrap_or(base_path))?;
            let relative_str = relative.to_string_lossy();
            
            if path.is_file() {
                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                
                zip.start_file(relative_str.to_string(), options)?;
                zip.write_all(&buffer)?;
            } else if path.is_dir() && relative_str != "." {
                zip.add_directory(&format!("{}/", relative_str), options)?;
            }
        }
        Ok(())
    }

    fn add_file_to_zip(&self, zip: &mut ZipWriter<File>, base_path: &Path, file_path: &Path, options: SimpleFileOptions) -> Result<()> {
        let relative = file_path.strip_prefix(base_path.parent().unwrap_or(base_path))?;
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        
        zip.start_file(relative.to_string_lossy().to_string(), options)?;
        zip.write_all(&buffer)?;
        Ok(())
    }

    fn create_tar(&self, files: &[String], output: &Path) -> Result<()> {
        let file = File::create(output)?;
        let mut tar = tar::Builder::new(file);

        for file_path in files {
            let full_path = self.resolve_path(file_path)?;
            let name = full_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.clone());

            if full_path.is_dir() {
                tar.append_dir_all(&name, &full_path)?;
            } else {
                tar.append_path_with_name(&full_path, &name)?;
            }
        }

        tar.finish()?;
        Ok(())
    }

    fn create_tar_gz(&self, files: &[String], output: &Path) -> Result<()> {
        let file = File::create(output)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut tar = tar::Builder::new(encoder);

        for file_path in files {
            let full_path = self.resolve_path(file_path)?;
            let name = full_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.clone());

            if full_path.is_dir() {
                tar.append_dir_all(&name, &full_path)?;
            } else {
                tar.append_path_with_name(&full_path, &name)?;
            }
        }

        tar.into_inner()?.finish()?;
        Ok(())
    }

    pub fn extract_archive(&self, archive_path: &str, destination: &str) -> Result<ExtractResult> {
        let full_path = self.resolve_path(archive_path)?;
        let dest_path = self.server_root.join(destination);
        
        fs::create_dir_all(&dest_path)?;

        let extension = full_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let name_without_ext = full_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let is_tar_gz = name_without_ext.ends_with(".tar");

        let files_extracted = if extension == "zip" {
            self.extract_zip(&full_path, &dest_path)?
        } else if is_tar_gz || extension == "gz" {
            self.extract_tar_gz(&full_path, &dest_path)?
        } else if extension == "tar" {
            self.extract_tar(&full_path, &dest_path)?
        } else {
            anyhow::bail!("Unsupported archive format: {}", extension);
        };

        Ok(ExtractResult {
            destination: dest_path.to_string_lossy().to_string(),
            files_extracted,
        })
    }

    fn extract_zip(&self, archive: &Path, destination: &Path) -> Result<usize> {
        let file = File::open(archive)?;
        let mut zip = ZipArchive::new(file)?;
        let mut count = 0;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let outpath = destination.join(file.mangled_name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn extract_tar(&self, archive: &Path, destination: &Path) -> Result<usize> {
        let file = File::open(archive)?;
        let mut tar = TarArchive::new(file);
        tar.unpack(destination)?;
        Ok(tar.entries()?.count())
    }

    fn extract_tar_gz(&self, archive: &Path, destination: &Path) -> Result<usize> {
        let file = File::open(archive)?;
        let decoder = GzDecoder::new(file);
        let mut tar = TarArchive::new(decoder);
        tar.unpack(destination)?;
        Ok(tar.entries()?.count())
    }

    pub fn list_archive_contents(&self, archive_path: &str) -> Result<Vec<ArchiveEntry>> {
        let full_path = self.resolve_path(archive_path)?;
        let extension = full_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let name_without_ext = full_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let is_tar_gz = name_without_ext.ends_with(".tar");

        if extension == "zip" {
            self.list_zip_contents(&full_path)
        } else if is_tar_gz || extension == "gz" {
            self.list_tar_gz_contents(&full_path)
        } else if extension == "tar" {
            self.list_tar_contents(&full_path)
        } else {
            anyhow::bail!("Unsupported archive format: {}", extension);
        }
    }

    fn list_zip_contents(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(archive)?;
        let zip = ZipArchive::new(file)?;
        let mut entries = Vec::new();

        for i in 0..zip.len() {
            let file = zip.by_index(i)?;
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                is_dir: file.is_dir(),
                size: file.size(),
                compressed_size: file.compressed_size(),
            });
        }

        Ok(entries)
    }

    fn list_tar_contents(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(archive)?;
        let mut tar = TarArchive::new(file);
        let mut entries = Vec::new();

        for entry in tar.entries()? {
            let entry = entry?;
            entries.push(ArchiveEntry {
                name: entry.path()?.to_string_lossy().to_string(),
                is_dir: entry.header().entry_type().is_dir(),
                size: entry.size(),
                compressed_size: entry.size(),
            });
        }

        Ok(entries)
    }

    fn list_tar_gz_contents(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(archive)?;
        let decoder = GzDecoder::new(file);
        let mut tar = TarArchive::new(decoder);
        let mut entries = Vec::new();

        for entry in tar.entries()? {
            let entry = entry?;
            entries.push(ArchiveEntry {
                name: entry.path()?.to_string_lossy().to_string(),
                is_dir: entry.header().entry_type().is_dir(),
                size: entry.size(),
                compressed_size: entry.size(),
            });
        }

        Ok(entries)
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let requested_path = self.server_root.join(path);
        
        if requested_path.exists() {
            let canonical = requested_path.canonicalize()?;
            if !canonical.starts_with(&self.server_root) {
                anyhow::bail!("Path escape detected: {}", path);
            }
            Ok(canonical)
        } else {
            Ok(requested_path)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveRequest {
    pub files: Vec<String>,
    pub output_name: String,
    pub format: ArchiveFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveResult {
    pub archive_path: String,
    pub total_size: u64,
    pub files_included: usize,
    pub format: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractResult {
    pub destination: String,
    pub files_extracted: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: u64,
}
