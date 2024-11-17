use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[tracing::instrument(skip(storage))]
pub async fn run_gc(storage: &mut dyn crate::storage::DemoStorage) -> Result<(), ()> {
    let stored_demos = match storage.list_demos().await {
        Ok(ds) => ds,
        Err(e) => return Err(()),
    };

    tracing::info!("Found {} demos in storage", stored_demos.len());

    let mut db_con = crate::db_connection().await;

    let db_res = db_con
        .build_transaction()
        .run(move |con| {
            Box::pin(async move {
                for demo in stored_demos {
                    let query = crate::schema::demos::dsl::demos
                        .filter(crate::schema::demos::dsl::demo_id.eq(&demo)).select(crate::models::Demo::as_select());
                    
                    let matching: Vec<crate::models::Demo> = query.load(con).await?;

                    if matching.is_empty() {
                        tracing::debug!("Should delete old demo {:?}", demo);
                    }
                }

                Ok::<(), diesel::result::Error>(())
            })
        })
        .await;

    db_res.map_err(|e| ())
}
