use super::*;

pub struct HeatmapAnalysis {}

impl HeatmapAnalysis {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analysis for HeatmapAnalysis {
    #[tracing::instrument(name = "Heatmap", skip(self, input))]
    fn analyse(&self, input: AnalysisInput) -> Result<Box<dyn FnOnce(&mut diesel_async::pg::AsyncPgConnection) -> core::pin::Pin<Box<(dyn core::future::Future<Output = Result<(), diesel::result::Error>> + Send + '_)>> + Send>, ()> {
        tracing::info!("Generating HEATMAPs");

    let file = std::fs::File::open(&input.path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

    let config = analysis::heatmap::Config {
        cell_size: 5.0,
    };
    let result = analysis::heatmap::parse(&config, &mmap).unwrap();

    tracing::info!("Got {} Entity-Heatmaps", result.player_heatmaps.len());
    let heatmap_result: Vec<_> = result.player_heatmaps.into_iter().filter_map(|((userid, team), heatmap)| {
        let player = match result.player_info.get(&userid) {
            Some(p) => p,
            None => {
                tracing::warn!("Could not find player: {:?}", userid);
                return None;
            }
        };
        
        Some(((player.xuid.to_string(), team), heatmap))
    }).collect();

        let player_heatmaps: Vec<_> = heatmap_result.into_iter().map(|((player, team), heatmap)| {
            tracing::trace!("HeatMap for Player: {:?} in Team {:?}", player, team);

            crate::models::DemoPlayerHeatmap {
                demo_id: input.demoid.clone(),
                steam_id: player,
                team,
                data: serde_json::to_string(&heatmap).unwrap(),
            }
        }).collect();

        Ok(Box::new(move |connection| {
            let store_demo_player_heatmaps_query = diesel::dsl::insert_into(crate::schema::demo_heatmaps::dsl::demo_heatmaps)
            .values(player_heatmaps)
            .on_conflict((crate::schema::demo_heatmaps::dsl::demo_id, crate::schema::demo_heatmaps::dsl::steam_id, crate::schema::demo_heatmaps::dsl::team))
            .do_update()
            .set(crate::schema::demo_heatmaps::dsl::data.eq(diesel::upsert::excluded(crate::schema::demo_heatmaps::dsl::data)));

            Box::pin(async move {
                store_demo_player_heatmaps_query.execute(connection).await?;

                Ok(())
            })
        }))
    }
}
