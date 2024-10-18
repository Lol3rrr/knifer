use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub mod base;
pub mod heatmap;
pub mod perround;

#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub steamid: String,
    pub demoid: String,
    data: std::sync::Arc<memmap2::Mmap>,
}

impl AnalysisInput {
    pub async fn load(
        steamid: String,
        demoid: String,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, ()> {
        let file = std::fs::File::open(path.as_ref()).unwrap();
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

        Ok(Self {
            steamid,
            demoid,
            data: std::sync::Arc::new(mmap),
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub trait Analysis {
    fn analyse(
        &self,
        input: AnalysisInput,
    ) -> Result<
        Box<
            dyn FnOnce(
                    &mut diesel_async::pg::AsyncPgConnection,
                ) -> core::pin::Pin<
                    Box<
                        (dyn core::future::Future<Output = Result<(), diesel::result::Error>>
                             + Send
                             + '_),
                    >,
                > + Send,
        >,
        (),
    >;
}

pub static ANALYSIS_METHODS: std::sync::LazyLock<[std::sync::Arc<dyn Analysis + Send + Sync>; 3]> =
    std::sync::LazyLock::new(|| {
        [
            std::sync::Arc::new(base::BaseAnalysis::new()),
            std::sync::Arc::new(heatmap::HeatmapAnalysis::new()),
            std::sync::Arc::new(perround::PerRoundAnalysis::new()),
        ]
    });

#[derive(Debug)]
pub enum TaskError<AE> {
    Diesel(diesel::result::Error),
    RunningAction(AE),
}

impl<AE> From<diesel::result::Error> for TaskError<AE> {
    fn from(value: diesel::result::Error) -> Self {
        Self::Diesel(value)
    }
}

pub async fn poll_next_task<A, AE>(
    upload_folder: impl Into<std::path::PathBuf>,
    db_con: &mut diesel_async::pg::AsyncPgConnection,
    action: A,
) -> Result<(), TaskError<AE>>
where
    A: Fn(
            AnalysisInput,
            &mut diesel_async::pg::AsyncPgConnection,
        )
            -> core::pin::Pin<Box<(dyn core::future::Future<Output = Result<(), AE>> + Send + '_)>>
        + Send
        + Clone
        + Sync,
{
    let query = crate::schema::analysis_queue::dsl::analysis_queue
        .order(crate::schema::analysis_queue::dsl::created_at.asc())
        .limit(1)
        .select(crate::models::AnalysisTask::as_select())
        .for_update()
        .skip_locked();

    let upload_folder: std::path::PathBuf = upload_folder.into();

    loop {
        let upload_folder = upload_folder.clone();
        let action = action.clone();

        let result = db_con
            .build_transaction()
            .run::<_, TaskError<AE>, _>(|conn| {
                Box::pin(async move {
                    let mut results: Vec<crate::models::AnalysisTask> = query.load(conn).await?;
                    let task = match results.pop() {
                        Some(r) => r,
                        None => return Ok(None),
                    };

                    let input = AnalysisInput::load(
                        task.steam_id.clone(),
                        task.demo_id.clone(),
                        upload_folder
                            .join(&task.steam_id)
                            .join(format!("{}.dem", task.demo_id)),
                    )
                    .await
                    .unwrap();

                    let tmp = action(input, &mut *conn);
                    tmp.await.map_err(|e| TaskError::RunningAction(e))?;

                    let delete_query =
                        diesel::dsl::delete(crate::schema::analysis_queue::dsl::analysis_queue)
                            .filter(crate::schema::analysis_queue::dsl::demo_id.eq(task.demo_id))
                            .filter(crate::schema::analysis_queue::dsl::steam_id.eq(task.steam_id));
                    delete_query.execute(conn).await?;

                    Ok(Some(()))
                })
            })
            .await;

        match result {
            Ok(Some(())) => {
                return Ok(());
            }
            Ok(None) => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
}
