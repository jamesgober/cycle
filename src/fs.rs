//! Async file system operations

use crate::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::ready;

/// Async file handle
pub struct File {
    inner: std::fs::File,
}

/// File open options
#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    /// Create new open options
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
    
    /// Open for reading
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    
    /// Open for writing
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    
    /// Open for appending
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    
    /// Truncate file
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    
    /// Create file if it doesn't exist
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    
    /// Create new file (fail if exists)
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }
    
    /// Open the file with these options
    pub async fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let mut opts = std::fs::OpenOptions::new();
        opts.read(self.read)
            .write(self.write)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create)
            .create_new(self.create_new);
        
        // Execute blocking operation in thread pool
        let path = path.as_ref().to_owned();
        let file = crate::task::spawn_blocking(move || opts.open(path)).await??;
        
        Ok(File { inner: file })
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl File {
    /// Open file for reading
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        OpenOptions::new()
            .read(true)
            .open(path)
            .await
    }
    
    /// Create new file for writing
    pub async fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
    }
    
    /// Get file metadata
    pub async fn metadata(&self) -> io::Result<std::fs::Metadata> {
        let file = self.inner.try_clone()?;
        crate::task::spawn_blocking(move || file.metadata()).await?
    }
    
    /// Sync all data to disk
    pub async fn sync_all(&self) -> io::Result<()> {
        let file = self.inner.try_clone()?;
        crate::task::spawn_blocking(move || file.sync_all()).await?
    }
    
    /// Sync data (not metadata) to disk
    pub async fn sync_data(&self) -> io::Result<()> {
        let file = self.inner.try_clone()?;
        crate::task::spawn_blocking(move || file.sync_data()).await?
    }
    
    /// Set file length
    pub async fn set_len(&self, size: u64) -> io::Result<()> {
        let file = self.inner.try_clone()?;
        crate::task::spawn_blocking(move || file.set_len(size)).await?
    }
    
    /// Seek to position
    pub async fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        use std::io::Seek;
        let mut file = self.inner.try_clone()?;
        crate::task::spawn_blocking(move || file.seek(pos)).await?
    }
}

impl AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        use std::io::Read;
        // Note: This is a simplified implementation
        // A real implementation would use proper async I/O
        Poll::Ready((&self.inner).read(buf))
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        use std::io::Write;
        Poll::Ready((&self.inner).write(buf))
    }
    
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        use std::io::Write;
        Poll::Ready((&self.inner).flush())
    }
    
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Read entire file to string
pub async fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

/// Read entire file to bytes
pub async fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    Ok(contents)
}

/// Write string to file
pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let mut file = File::create(path).await?;
    file.write_all(contents.as_ref()).await?;
    file.sync_all().await?;
    Ok(())
}

/// Copy file
pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let mut source = File::open(from).await?;
    let mut dest = File::create(to).await?;
    crate::io::copy(&mut source, &mut dest).await
}

/// Remove file
pub async fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || std::fs::remove_file(path)).await?
}

/// Create directory
pub async fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || std::fs::create_dir(path)).await?
}

/// Create directory and all parent directories
pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || std::fs::create_dir_all(path)).await?
}

/// Remove directory
pub async fn remove_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || std::fs::remove_dir(path)).await?
}

/// Remove directory and all contents
pub async fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || std::fs::remove_dir_all(path)).await?
}

/// Read directory entries
pub async fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
    let path = path.as_ref().to_owned();
    crate::task::spawn_blocking(move || {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok::<_, io::Error>(entries)
    }).await?
}
