use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub mod base;
pub mod heatmap;
pub mod perround;

#[derive(Debug, Clone)]
pub enum AnalysisData {
    MemMapped(std::sync::Arc<memmap2::Mmap>),
    Preloaded(std::sync::Arc<[u8]>),
}

#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub steamid: String,
    pub demoid: String,
    data: AnalysisData,
}

impl AnalysisInput {
    pub async fn load(
        steamid: String,
        demoid: String,
        storage: &dyn crate::storage::DemoStorage,
    ) -> Result<Self, String> {
        let data = storage.load(steamid.clone(), demoid.clone()).await?;

        Ok(Self {
            steamid,
            demoid,
            data,
        })
    }

    pub fn data(&self) -> &[u8] {
        match &self.data {
            AnalysisData::MemMapped(v) => &v,
            AnalysisData::Preloaded(v) => &v,
        }
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
    storage: Box<dyn crate::storage::DemoStorage>,
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

    loop {
        let storage = storage.duplicate();
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

                    let delete_query =
                        diesel::dsl::delete(crate::schema::analysis_queue::dsl::analysis_queue)
                            .filter(crate::schema::analysis_queue::dsl::demo_id.eq(task.demo_id.clone()))
                            .filter(crate::schema::analysis_queue::dsl::steam_id.eq(task.steam_id.clone()));

                    let input = match AnalysisInput::load(
                        task.steam_id,
                        task.demo_id,
                        storage.as_ref(),
                    )
                    .await {
                        Ok(i) => i,
                        Err(e) => {
                            tracing::error!("Loading Analysis Input: {:?}", e);
                            delete_query.execute(conn).await?;
                            return Ok(Some(()));
                        }
                    };

                    let tmp = action(input, &mut *conn);
                    tmp.await.map_err(|e| TaskError::RunningAction(e))?;

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
