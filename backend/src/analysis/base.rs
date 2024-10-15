use super::*;

#[derive(Debug)]
struct BaseInfo {
    pub map: String,
    pub players: Vec<(BasePlayerInfo, BasePlayerStats)>,
    pub teams: std::collections::HashMap<i32, BaseTeamInfo>,
}

#[derive(Debug)]
struct BaseTeamInfo {
    pub start_side: String,
    pub end_score: usize,
}

#[derive(Debug)]
struct BasePlayerInfo {
    pub name: String,
    pub steam_id: String,
    pub team: i32,
    pub color: i32,
    pub ingame_id: i32,
}

#[derive(Debug)]
struct BasePlayerStats {
    pub kills: usize,
    pub deaths: usize,
    pub damage: usize,
    pub assists: usize,
}

pub struct BaseAnalysis {}

impl BaseAnalysis {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analysis for BaseAnalysis {
    #[tracing::instrument(name = "Base", skip(self, input))]
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
        tracing::info!("Performing Base analysis");

        let file = std::fs::File::open(&input.path).unwrap();
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

        let result = analysis::endofgame::parse(&mmap).unwrap();

        let base_result = BaseInfo {
            map: result.map,
            teams: result
                .teams
                .into_iter()
                .map(|(numb, team)| {
                    (
                        numb,
                        BaseTeamInfo {
                            end_score: team.end_score,
                            start_side: team.start_side,
                        },
                    )
                })
                .collect(),
            players: result
                .players
                .into_iter()
                .map(|(info, stats)| {
                    (
                        BasePlayerInfo {
                            name: info.name,
                            steam_id: info.steam_id,
                            team: info.team,
                            ingame_id: info.ingame_id,
                            color: info.color,
                        },
                        BasePlayerStats {
                            kills: stats.kills,
                            assists: stats.assists,
                            damage: stats.damage,
                            deaths: stats.deaths,
                        },
                    )
                })
                .collect(),
        };

        let (player_info, player_stats): (Vec<_>, Vec<_>) = base_result
            .players
            .into_iter()
            .map(|(info, stats)| {
                (
                    crate::models::DemoPlayer {
                        demo_id: input.demoid.clone(),
                        name: info.name,
                        steam_id: info.steam_id.clone(),
                        team: info.team as i16,
                        color: info.color as i16,
                    },
                    crate::models::DemoPlayerStats {
                        demo_id: input.demoid.clone(),
                        steam_id: info.steam_id,
                        deaths: stats.deaths as i16,
                        kills: stats.kills as i16,
                        damage: stats.damage as i16,
                        assists: stats.assists as i16,
                    },
                )
            })
            .unzip();

        let teams = base_result
            .teams
            .into_iter()
            .map(|(numb, team)| crate::models::DemoTeam {
                demo_id: input.demoid.clone(),
                team: numb as i16,
                end_score: team.end_score as i16,
                start_name: team.start_side,
            })
            .collect::<Vec<_>>();

        let demo_info = crate::models::DemoInfo {
            demo_id: input.demoid.clone(),
            map: base_result.map,
        };

        Ok(Box::new(move |connection| {
            let store_demo_info_query =
                diesel::dsl::insert_into(crate::schema::demo_info::dsl::demo_info)
                    .values(demo_info)
                    .on_conflict(crate::schema::demo_info::dsl::demo_id)
                    .do_update()
                    .set(
                        crate::schema::demo_info::dsl::map
                            .eq(diesel::upsert::excluded(crate::schema::demo_info::dsl::map)),
                    );
            let store_demo_players_query =
                diesel::dsl::insert_into(crate::schema::demo_players::dsl::demo_players)
                    .values(player_info)
                    .on_conflict_do_nothing();

            let store_demo_player_stats_query =
                diesel::dsl::insert_into(crate::schema::demo_player_stats::dsl::demo_player_stats)
                    .values(player_stats)
                    .on_conflict((
                        crate::schema::demo_player_stats::dsl::demo_id,
                        crate::schema::demo_player_stats::dsl::steam_id,
                    ))
                    .do_update()
                    .set((
                        crate::schema::demo_player_stats::dsl::deaths.eq(diesel::upsert::excluded(
                            crate::schema::demo_player_stats::dsl::deaths,
                        )),
                        crate::schema::demo_player_stats::dsl::kills.eq(diesel::upsert::excluded(
                            crate::schema::demo_player_stats::dsl::kills,
                        )),
                        crate::schema::demo_player_stats::dsl::assists.eq(
                            diesel::upsert::excluded(
                                crate::schema::demo_player_stats::dsl::assists,
                            ),
                        ),
                        crate::schema::demo_player_stats::dsl::damage.eq(diesel::upsert::excluded(
                            crate::schema::demo_player_stats::dsl::damage,
                        )),
                    ));

            let store_teams = diesel::dsl::insert_into(crate::schema::demo_teams::dsl::demo_teams)
                .values(teams)
                .on_conflict((
                    crate::schema::demo_teams::dsl::demo_id,
                    crate::schema::demo_teams::dsl::team,
                ))
                .do_update()
                .set((
                    crate::schema::demo_teams::dsl::start_name.eq(diesel::upsert::excluded(
                        crate::schema::demo_teams::dsl::start_name,
                    )),
                    crate::schema::demo_teams::dsl::end_score.eq(diesel::upsert::excluded(
                        crate::schema::demo_teams::dsl::end_score,
                    )),
                ));

            Box::pin(async move {
                store_demo_info_query.execute(connection).await?;
                store_demo_players_query.execute(connection).await?;
                store_demo_player_stats_query.execute(connection).await?;
                store_teams.execute(connection).await?;

                Ok(())
            })
        }))
    }
}
