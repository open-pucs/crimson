I currently have this snippet of code from local store:

```rs

/// Local filesystem-based implementation of FileStore.
#[derive(Debug, Clone)]
pub struct LocalFileStore {
    base_path: PathBuf,
}

impl LocalFileStore {
    /// Create a new LocalFileStore with the given base directory.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        LocalFileStore {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

impl FileStoreImplementation for LocalFileStore {
    async fn upload_from_file(
        &self,
        local_path: LocalPath,
        _: String,
    ) -> Result<FileLocation, StoreError> {
        Ok(FileLocation::LocalPath(local_path))
    }

    async fn download_to_file(&self, src: &FileLocation) -> Result<LocalPath, StoreError> {
        match src {
            FileLocation::LocalPath(rel) => return Ok(rel.clone()),
            FileLocation::S3Uri(s3_uri) => {
                todo!("Implement downloading from s3 for local file store implementation")
            }
        }
    }

    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError> {
        match target {
            FileLocation::LocalPath(rel) => {
                let path = self.base_path.join(rel);
                fs::remove_file(&path)
                    .await
                    .map_err(|_| StoreError::InvalidLocation)?;
                Ok(())
            }
            _ => Err(StoreError::InvalidLocation),
        }
    }
}
```
