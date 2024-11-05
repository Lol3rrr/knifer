use super::*;

pub struct HeadToHeadAnalysis {}

impl HeadToHeadAnalysis {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analysis for HeadToHeadAnalysis {
    #[tracing::instrument(name = "HeadToHead", skip(self, input))]
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
    > {
        tracing::info!("Performing Head-to-Head analysis");

        let result = analysis::head_to_head::parse(input.data()).inspect_err(|e| {
            tracing::error!("{:?}", e);
        }).map_err(|e| ())?;

        let values_to_insert: Vec<_> = result.head_to_head.into_iter().flat_map(|(user_id, enemies)| {
            enemies.into_iter().map(move |(enemy, kills)| (user_id, enemy, kills))
        }).filter_map(|(user_id, enemy_id, kills)| {
                let player = result.players.get(&user_id)?;
                let enemy = result.players.get(&enemy_id)?;
                
                Some(crate::models::DemoHeadToHead {
                    demo_id: input.demoid.clone(),
                    player: player.xuid.to_string(),
                    enemy: enemy.xuid.to_string(),
                    kills: kills as i16,
                })
            }).collect();

        Ok(Box::new(move |connection| {
            // TODO
            // Construct the actual queries
            
            let query = diesel::insert_into(crate::schema::demo_head_to_head::dsl::demo_head_to_head)
                .values(values_to_insert)
                .on_conflict((
                    crate::schema::demo_head_to_head::dsl::demo_id,
                    crate::schema::demo_head_to_head::dsl::player,
                    crate::schema::demo_head_to_head::dsl::enemy,
                ))
                .do_update().set(crate::schema::demo_head_to_head::kills.eq(diesel::upsert::excluded(crate::schema::demo_head_to_head::kills)));

            Box::pin(async move {
                // TODO
                // Execute queries
                query.execute(connection).await?;

                Ok(())
            })
        }))
    }
}
