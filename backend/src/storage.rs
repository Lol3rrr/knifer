use futures::FutureExt;
use futures::StreamExt;

pub trait DemoStorage: Send + Sync {
    fn duplicate(&self) -> Box<dyn DemoStorage>;

    fn upload<'f, 's, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
        stream: futures_util::stream::BoxStream<'s, axum::body::Bytes>,
    ) -> futures::future::BoxFuture<'f, Result<(), String>>
    where
        's: 'f,
        'own: 'f;

    fn load<'f, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
    ) -> futures::future::BoxFuture<'f, Result<crate::analysis::AnalysisData, String>>
    where
        'own: 'f;
}

pub struct FileStorage {
    folder: std::sync::Arc<std::path::PathBuf>,
}

impl FileStorage {
    pub fn new<P>(folder: P) -> Self
    where
        P: Into<std::path::PathBuf>,
    {
        Self {
            folder: std::sync::Arc::new(folder.into()),
        }
    }
}

impl DemoStorage for FileStorage {
    fn duplicate(&self) -> Box<dyn DemoStorage> {
        Box::new(Self {
            folder: self.folder.clone(),
        })
    }

    fn upload<'f, 's, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
        stream: futures_util::stream::BoxStream<'s, axum::body::Bytes>,
    ) -> futures::future::BoxFuture<'f, Result<(), String>>
    where
        's: 'f,
        'own: 'f,
    {
        let path = self.folder.clone();

        async move {
            let user_folder = std::path::Path::new(path.as_path()).join(format!("{}/", user_id));
            if !tokio::fs::try_exists(&user_folder).await.unwrap_or(false) {
                tokio::fs::create_dir_all(&user_folder).await.unwrap();
            }

            let demo_file_path = user_folder.join(format!("{}.dem", demo_id));

            async {
                // Convert the stream into an `AsyncRead`.
                let body_with_io_error = stream.map(|b| Ok::<_, std::io::Error>(b));
                let body_reader = tokio_util::io::StreamReader::new(body_with_io_error);
                futures::pin_mut!(body_reader);

                // Create the file. `File` implements `AsyncWrite`.
                let mut file =
                    tokio::io::BufWriter::new(tokio::fs::File::create(demo_file_path).await?);

                // Copy the body into the file.
                tokio::io::copy(&mut body_reader, &mut file).await?;

                Ok::<_, std::io::Error>(())
            }
            .await
            .map_err(|err| err.to_string())
        }
        .boxed()
    }

    fn load<'f, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
    ) -> futures::future::BoxFuture<'f, Result<crate::analysis::AnalysisData, String>>
    where
        'own: 'f,
    {
        async move {
            let user_folder =
                std::path::Path::new(self.folder.as_path()).join(format!("{}/", user_id));
            let demo_file_path = user_folder.join(format!("{}.dem", demo_id));
            let file = std::fs::File::open(demo_file_path.as_path()).unwrap();
            let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

            Ok(crate::analysis::AnalysisData::MemMapped(
                std::sync::Arc::new(mmap),
            ))
        }
        .boxed()
    }
}

pub struct S3Storage {
    bucket: std::sync::Arc<s3::Bucket>,
}

impl S3Storage {
    pub fn new(
        bucket_name: &str,
        region: s3::region::Region,
        credentials: s3::creds::Credentials,
    ) -> Self {
        let mut bucket = s3::bucket::Bucket::new(bucket_name, region, credentials).unwrap();
        bucket.set_path_style();

        Self {
            bucket: bucket.into(),
        }
    }
}

impl DemoStorage for S3Storage {
    fn duplicate(&self) -> Box<dyn DemoStorage> {
        Box::new(Self {
            bucket: self.bucket.clone(),
        })
    }

    fn upload<'f, 's, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
        stream: futures_util::stream::BoxStream<'s, axum::body::Bytes>,
    ) -> futures::future::BoxFuture<'f, Result<(), String>>
    where
        's: 'f,
        'own: 'f,
    {
        async move {
            let path = std::path::PathBuf::new().join(user_id).join(demo_id);
            let path = path.to_str().unwrap();

            // Convert the stream into an `AsyncRead`.
            let body_with_io_error = stream.map(|b| Ok::<_, std::io::Error>(b));
            let body_reader = tokio_util::io::StreamReader::new(body_with_io_error);
            futures::pin_mut!(body_reader);

            self.bucket
                .put_object_stream(&mut body_reader, path)
                .await
                .map_err(|e| format!("Uploading Stream to bucket: {:?}", e))?;

            Ok(())
        }
        .boxed()
    }

    fn load<'f, 'own>(
        &'own self,
        user_id: String,
        demo_id: String,
    ) -> futures::future::BoxFuture<'f, Result<crate::analysis::AnalysisData, String>>
    where
        'own: 'f,
    {
        async move {
            let path = std::path::PathBuf::new().join(user_id).join(demo_id);
            let path = path.to_str().unwrap();

            let resp = self.bucket.get_object(path).await.map_err(|e| format!("Loading from Bucket: {:?}", e))?;

            Ok(crate::analysis::AnalysisData::Preloaded(
                resp.to_vec().into(),
            ))
        }
        .boxed()
    }
}
