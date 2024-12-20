use crate::UserSession;
use axum::extract::{Path, State};
use diesel::prelude::*;
use diesel::JoinOnDsl;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

struct DemoState {
    storage: Box<dyn crate::storage::DemoStorage + Send + Sync>,
}

pub fn router(storage: Box<dyn crate::storage::DemoStorage>) -> axum::Router {
    axum::Router::new()
        .route("/list", axum::routing::get(list))
        .route(
            "/upload",
            axum::routing::post(upload)
                .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/:id/info", axum::routing::get(info))
        .route("/:id/reanalyse", axum::routing::get(analyise))
        .route("/:id/analysis/scoreboard", axum::routing::get(scoreboard))
        .route("/:id/analysis/perround", axum::routing::get(perround))
        .route("/:id/analysis/heatmap", axum::routing::get(heatmap))
        .route("/:id/analysis/headtohead", axum::routing::get(head_to_head))
        .with_state(Arc::new(DemoState { storage }))
}

#[tracing::instrument(skip(session))]
async fn list(
    session: UserSession,
) -> Result<axum::response::Json<common::DemoList>, axum::http::StatusCode> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| axum::http::StatusCode::UNAUTHORIZED)?;
    tracing::info!("SteamID: {:?}", steam_id);

    let done_query = crate::schema::demos::dsl::demos
        .inner_join(
            crate::schema::demo_info::table
                .on(crate::schema::demos::dsl::demo_id.eq(crate::schema::demo_info::dsl::demo_id)),
        )
        .inner_join(
            crate::schema::demo_teams::table
                .on(crate::schema::demos::dsl::demo_id.eq(crate::schema::demo_teams::dsl::demo_id)),
        )
        .inner_join(
            crate::schema::demo_players::table
                .on(crate::schema::demos::dsl::demo_id
                    .eq(crate::schema::demo_players::dsl::demo_id)),
        )
        .select((
            crate::models::Demo::as_select(),
            crate::models::DemoInfo::as_select(),
            crate::models::DemoTeam::as_select(),
            crate::models::DemoPlayer::as_select(),
        ))
        .filter(
            crate::schema::demos::dsl::steam_id
                .eq(steam_id.to_string())
                .and(crate::schema::demo_players::dsl::steam_id.eq(steam_id.to_string())),
        );
    let pending_query = crate::schema::demos::dsl::demos
        .inner_join(crate::schema::processing_status::table.on(
            crate::schema::demos::dsl::demo_id.eq(crate::schema::processing_status::dsl::demo_id),
        ))
        .select((crate::models::Demo::as_select()))
        .filter(
            crate::schema::demos::dsl::steam_id
                .eq(steam_id.to_string())
                .and(crate::schema::processing_status::dsl::info.ne(1)),
        );

    let mut db_con = crate::db_connection().await;

    let (results, pending) = db_con
        .build_transaction()
        .read_only()
        .run::<_, diesel::result::Error, _>(|con| {
            Box::pin(async move {
                let done_results: Vec<(
                    crate::models::Demo,
                    crate::models::DemoInfo,
                    crate::models::DemoTeam,
                    crate::models::DemoPlayer,
                )> = done_query.load(con).await?;

                let pending_results: Vec<(crate::models::Demo)> = pending_query.load(con).await?;

                Ok((done_results, pending_results))
            })
        })
        .await
        .unwrap();

    let mut demos = std::collections::HashMap::new();
    for (demo, info, team, player) in results.into_iter() {
        let entry = demos
            .entry(demo.demo_id.clone())
            .or_insert(common::BaseDemoInfo {
                id: demo.demo_id,
                map: info.map,
                uploaded_at: demo.uploaded_at,
                team2_score: 0,
                team3_score: 0,
                player_team: player.team,
            });

        if team.team == 2 {
            entry.team2_score = team.end_score;
        } else if team.team == 3 {
            entry.team3_score = team.end_score;
        } else {
            tracing::warn!("Unknown Team: {:?}", team);
        }
    }

    let mut done_demos = demos.into_values().collect::<Vec<_>>();
    done_demos.sort_unstable_by_key(|d| std::cmp::Reverse(d.uploaded_at));

    Ok(axum::response::Json(common::DemoList {
        done: done_demos,
        pending: pending.into_iter().map(|d| ()).collect(),
    }))
}

#[tracing::instrument(skip(state, session, form))]
async fn upload(
    State(state): State<Arc<DemoState>>,
    session: crate::UserSession,
    mut form: axum::extract::Multipart,
) -> Result<(), (axum::http::StatusCode, String)> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in".into()))?;

    tracing::info!("Starting upload for Session: {:?}", steam_id);

    let file_field = loop {
        let field = match form.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => {
                tracing::error!("");
                return Err((axum::http::StatusCode::BAD_REQUEST, "Missing Data".into()));
            }
            Err(e) => {
                tracing::error!("");
                return Err((axum::http::StatusCode::BAD_REQUEST, "".into()));
            }
        };

        if field.name().map(|n| n == "demo").unwrap_or(false) {
            break field;
        }
    };

    let raw_demo_id = uuid::Uuid::now_v7();
    let demo_id = raw_demo_id.to_string();

    use futures::stream::StreamExt;
    state
        .storage
        .upload(
            steam_id.to_string(),
            demo_id.clone(),
            file_field
                .filter_map(|b| async { b.ok() })
                .inspect(|b| {
                    tracing::debug!("Received {} bytes", b.len());
                })
                .boxed(),
        )
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    tracing::info!("Saved to storage");

    let mut db_con = crate::db_connection().await;

    let db_trans_result = db_con
        .build_transaction()
        .run(|c| {
            Box::pin(async move {
                let query = diesel::dsl::insert_into(crate::schema::demos::dsl::demos).values(
                    crate::models::NewDemo {
                        demo_id: demo_id.clone(),
                        steam_id: steam_id.to_string(),
                    },
                );
                query.execute(c).await?;

                let queue_query =
                    diesel::dsl::insert_into(crate::schema::analysis_queue::dsl::analysis_queue)
                        .values(crate::models::AddAnalysisTask {
                            demo_id: demo_id.clone(),
                            steam_id: steam_id.to_string(),
                        });
                queue_query.execute(c).await?;

                let processing_query = diesel::dsl::insert_into(
                    crate::schema::processing_status::dsl::processing_status,
                )
                .values(crate::models::ProcessingStatus { demo_id, info: 0 });
                processing_query.execute(c).await?;

                Ok::<(), diesel::result::Error>(())
            })
        })
        .await;

    if let Err(e) = db_trans_result {
        tracing::error!("Inserting data into db: {:?}", e);
    }

    tracing::info!("Done with upload");

    Ok(())
}

#[tracing::instrument(skip(session))]
async fn analyise(
    session: crate::UserSession,
    Path(demo_id): Path<String>,
) -> Result<(), (axum::http::StatusCode, &'static str)> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in"))?;

    tracing::info!("Upload for Session: {:?}", steam_id);

    let mut db_con = crate::db_connection().await;

    let query = crate::schema::demos::dsl::demos
        .filter(crate::schema::demos::dsl::steam_id.eq(steam_id.to_string()))
        .filter(crate::schema::demos::dsl::demo_id.eq(demo_id.clone()));
    let result: Vec<_> = query
        .load::<crate::models::Demo>(&mut db_con)
        .await
        .unwrap();

    if result.len() != 1 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Expected exactly 1 demo to match",
        ));
    }

    let queue_query = diesel::dsl::insert_into(crate::schema::analysis_queue::dsl::analysis_queue)
        .values(crate::models::AddAnalysisTask {
            demo_id,
            steam_id: steam_id.to_string(),
        });
    queue_query.execute(&mut db_con).await.unwrap();

    Ok(())
}

#[tracing::instrument(skip(_session))]
async fn info(
    _session: UserSession,
    Path(demo_id): Path<String>,
) -> Result<axum::response::Json<common::DemoInfo>, axum::http::StatusCode> {
    tracing::info!("Get info for Demo: {:?}", demo_id);

    let query = crate::schema::demo_info::dsl::demo_info
        .select(crate::models::DemoInfo::as_select())
        .filter(crate::schema::demo_info::dsl::demo_id.eq(demo_id));
    let mut results: Vec<crate::models::DemoInfo> =
        query.load(&mut crate::db_connection().await).await.unwrap();

    if results.len() != 1 {
        tracing::error!("Expected only 1 match but got {} matches", results.len());
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let result = results.pop().unwrap();

    Ok(axum::Json(common::DemoInfo {
        id: result.demo_id,
        map: result.map,
    }))
}

#[tracing::instrument(skip(session))]
async fn scoreboard(
    session: UserSession,
    Path(demo_id): Path<String>,
) -> Result<axum::response::Json<common::demo_analysis::ScoreBoard>, axum::http::StatusCode> {
    let query = crate::schema::demo_players::dsl::demo_players
        .inner_join(
            crate::schema::demo_player_stats::dsl::demo_player_stats.on(
                crate::schema::demo_players::dsl::demo_id
                    .eq(crate::schema::demo_player_stats::dsl::demo_id)
                    .and(
                        crate::schema::demo_players::dsl::steam_id
                            .eq(crate::schema::demo_player_stats::dsl::steam_id),
                    ),
            ),
        )
        .filter(crate::schema::demo_players::dsl::demo_id.eq(demo_id.clone()));

    let team_query = crate::schema::demo_teams::dsl::demo_teams
        .filter(crate::schema::demo_teams::dsl::demo_id.eq(demo_id));

    let mut db_con = crate::db_connection().await;

    let db_result = db_con
        .build_transaction()
        .read_only()
        .run::<_, diesel::result::Error, _>(|con| {
            Box::pin(async move {
                let players: Vec<(crate::models::DemoPlayer, crate::models::DemoPlayerStats)> =
                    query.load(con).await?;
                let teams: Vec<crate::models::DemoTeam> = team_query.load(con).await?;

                Ok((players, teams))
            })
        })
        .await;

    let (response, team_response) = match db_result {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Querying DB {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if response.is_empty() {
        tracing::error!("DB Response was empty");
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut teams = std::collections::BTreeMap::new();
    for (player, stats) in response {
        let team =
            teams
                .entry(player.team as u32)
                .or_insert(common::demo_analysis::ScoreBoardTeam {
                    number: player.team as u32,
                    players: Vec::new(),
                    score: 0,
                });

        team.players.push(common::demo_analysis::ScoreBoardPlayer {
            name: player.name,
            kills: stats.kills as usize,
            deaths: stats.deaths as usize,
            damage: stats.damage as usize,
            assists: stats.assists as usize,
        });
    }

    for team in team_response {
        let number = team.team as u32;
        if let Some(entry) = teams.get_mut(&number) {
            entry.score = team.end_score;
        }
    }

    Ok(axum::Json(common::demo_analysis::ScoreBoard {
        teams: teams.into_values().collect::<Vec<_>>(),
    }))
}

#[tracing::instrument(skip(session))]
async fn heatmap(
    session: UserSession,
    Path(demo_id): Path<String>,
) -> Result<axum::response::Json<Vec<common::demo_analysis::PlayerHeatmap>>, axum::http::StatusCode>
{
    use base64::prelude::Engine;

    let mut db_con = crate::db_connection().await;

    let demo_info_query = crate::schema::demo_info::dsl::demo_info
        .filter(crate::schema::demo_info::dsl::demo_id.eq(demo_id.clone()));
    let demo_info: crate::models::DemoInfo = match demo_info_query.first(&mut db_con).await {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Could not find Demo '{:?}': {:?}", demo_id, e);
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };

    let query = crate::schema::demo_players::dsl::demo_players
        .inner_join(
            crate::schema::demo_heatmaps::dsl::demo_heatmaps.on(
                crate::schema::demo_players::dsl::steam_id
                    .eq(crate::schema::demo_heatmaps::dsl::steam_id)
                    .and(
                        crate::schema::demo_players::dsl::demo_id
                            .eq(crate::schema::demo_heatmaps::dsl::demo_id),
                    ),
            ),
        )
        .filter(crate::schema::demo_players::dsl::demo_id.eq(demo_id));

    let result: Vec<(crate::models::DemoPlayer, crate::models::DemoPlayerHeatmap)> =
        match query.load(&mut db_con).await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Querying DB: {:?}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    let demo_map = &demo_info.map;
    let minimap_coords = match MINIMAP_COORDINATES.get(demo_map) {
        Some(c) => c,
        None => {
            tracing::error!("Unknown Map in Demo: {:?}", demo_map);
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };

    let data: Vec<common::demo_analysis::PlayerHeatmap> = result
        .into_iter()
        .map(|(player, heatmap)| {
            let team = heatmap.team.clone();
            let mut heatmap: analysis::heatmap::HeatMap =
                serde_json::from_str(&heatmap.data).unwrap();
            heatmap.fit(
                minimap_coords.x_coord(0.0)..minimap_coords.x_coord(1024.0),
                minimap_coords.y_coord(1024.0)..minimap_coords.y_coord(0.0),
            );
            let h_image = heatmap.as_image();

            let mut buffer = std::io::Cursor::new(Vec::new());
            h_image
                .write_to(&mut buffer, image::ImageFormat::Png)
                .unwrap();

            common::demo_analysis::PlayerHeatmap {
                name: player.name,
                team,
                png_data: base64::prelude::BASE64_STANDARD.encode(buffer.into_inner()),
            }
        })
        .collect();

    Ok(axum::Json(data))
}

#[tracing::instrument(skip(session))]
async fn perround(
    session: UserSession,
    Path(demo_id): Path<String>,
) -> Result<axum::response::Json<common::demo_analysis::PerRoundResult>, axum::http::StatusCode> {
    let rounds_query = crate::schema::demo_round::dsl::demo_round
        .filter(crate::schema::demo_round::dsl::demo_id.eq(demo_id.clone()));
    let round_players_query = crate::schema::demo_players::dsl::demo_players
        .filter(crate::schema::demo_players::dsl::demo_id.eq(demo_id.clone()));
    let demo_teams = crate::schema::demo_teams::dsl::demo_teams
        .filter(crate::schema::demo_teams::dsl::demo_id.eq(demo_id));

    let mut db_con = crate::db_connection().await;

    let raw_rounds: Vec<crate::models::DemoRound> = rounds_query.load(&mut db_con).await.unwrap();
    let players: Vec<crate::models::DemoPlayer> =
        round_players_query.load(&mut db_con).await.unwrap();
    let raw_teams: Vec<crate::models::DemoTeam> = demo_teams.load(&mut db_con).await.unwrap();

    let mut result = Vec::with_capacity(raw_rounds.len());
    for raw_round in raw_rounds.into_iter() {
        let reason = match serde_json::from_str(&raw_round.win_reason) {
            Ok(analysis::perround::WinReason::StillInProgress) => {
                common::demo_analysis::RoundWinReason::StillInProgress
            }
            Ok(analysis::perround::WinReason::TKilled) => {
                common::demo_analysis::RoundWinReason::TKilled
            }
            Ok(analysis::perround::WinReason::CTKilled) => {
                common::demo_analysis::RoundWinReason::CTKilled
            }
            Ok(analysis::perround::WinReason::BombDefused) => {
                common::demo_analysis::RoundWinReason::BombDefused
            }
            Ok(analysis::perround::WinReason::BombExploded) => {
                common::demo_analysis::RoundWinReason::BombExploded
            }
            Ok(analysis::perround::WinReason::TimeRanOut) => {
                common::demo_analysis::RoundWinReason::TimeRanOut
            }
            Ok(other) => {
                tracing::error!("Unknown Mapping {:?}", other);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(e) => {
                tracing::error!("Deserializing Win Reason: {:?}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let parsed_events: Vec<analysis::perround::RoundEvent> =
            serde_json::from_value(raw_round.events).unwrap();
        let events: Vec<_> = parsed_events
            .into_iter()
            .map(|event| match event {
                analysis::perround::RoundEvent::BombPlanted => {
                    common::demo_analysis::RoundEvent::BombPlanted
                }
                analysis::perround::RoundEvent::BombDefused => {
                    common::demo_analysis::RoundEvent::BombDefused
                }
                analysis::perround::RoundEvent::Kill {
                    attacker,
                    died,
                    weapon,
                    noscope,
                    headshot,
                } => {
                    let attacker_name = players
                        .iter()
                        .find(|p| p.steam_id == attacker.to_string())
                        .map(|p| p.name.clone())
                        .unwrap();
                    let died_name = players
                        .iter()
                        .find(|p| p.steam_id == died.to_string())
                        .map(|p| p.name.clone())
                        .unwrap();

                    common::demo_analysis::RoundEvent::Killed {
                        attacker: attacker_name,
                        died: died_name,
                        weapon,
                        headshot,
                        noscope,
                    }
                }
            })
            .collect();

        result.push(common::demo_analysis::DemoRound { reason, events });
    }

    let teams = raw_teams
        .into_iter()
        .map(|dteam| common::demo_analysis::PerRoundTeam {
            name: dteam.start_name,
            number: dteam.team as u32,
            players: players
                .iter()
                .filter(|p| p.team == dteam.team)
                .map(|p| p.name.clone())
                .collect(),
        })
        .collect();

    Ok(axum::Json(common::demo_analysis::PerRoundResult {
        rounds: result,
        teams,
    }))
}

#[tracing::instrument(skip(session))]
async fn head_to_head(
    session: UserSession,
    Path(demo_id): Path<String>,
) -> Result<axum::response::Json<common::demo_analysis::HeadToHead>, axum::http::StatusCode> {
    let mut db_con = crate::db_connection().await;

    let head_to_head_query = crate::schema::demo_head_to_head::dsl::demo_head_to_head
        .filter(crate::schema::demo_head_to_head::dsl::demo_id.eq(demo_id.clone()));

    let player_query = crate::schema::demo_players::dsl::demo_players
        .filter(crate::schema::demo_players::dsl::demo_id.eq(demo_id));

    let (players, head_to_head_entries) = db_con.build_transaction().read_only().run(|connection| Box::pin(async move {
        let head_to_head_entries: Vec<crate::models::DemoHeadToHead> = head_to_head_query.load(connection).await?;
        let players: Vec<crate::models::DemoPlayer> = player_query.load(connection).await?;

        Ok::<_, diesel::result::Error>((players, head_to_head_entries))
    })).await.map_err(|e| {
            tracing::error!("Querying DB: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let (mut row_team, mut column_team): (Vec<_>, Vec<_>) = players.into_iter().partition(|p| p.team == 2);
    row_team.sort_unstable_by_key(|p| p.color);
    column_team.sort_unstable_by_key(|p| p.color);

    let results: Vec<_> = row_team.iter().map(|row_player| {
        column_team.iter().map(|column_player| {
            let row_kills = head_to_head_entries.iter().find(|entry| entry.player == row_player.steam_id && entry.enemy == column_player.steam_id);
            let column_kills = head_to_head_entries.iter().find(|entry| entry.player == column_player.steam_id && entry.enemy == row_player.steam_id);

            (row_kills.map(|k| k.kills).unwrap_or(0), column_kills.map(|k| k.kills).unwrap_or(0))
        }).collect::<Vec<_>>()
    }).collect();

    Ok(axum::Json(common::demo_analysis::HeadToHead {
        row_players: row_team.into_iter().map(|p| p.name).collect(),
        column_players: column_team.into_iter().map(|p| p.name).collect(),
        entries: results,
    }))
}

// The corresponding values for each map can be found using the Source2 Viewer and opening the
// files in 'game/csgo/pak01_dir.vpk' and then 'resource/overviews/{map}.txt'
static MINIMAP_COORDINATES: phf::Map<&str, MiniMapDefinition> = phf::phf_map! {
    "cs_italy" => MiniMapDefinition {
        pos_x: -2647.0,
        pos_y: 2592.0,
        scale: 4.6
    },
    "cs_office" => MiniMapDefinition {
        pos_x: -1838.0,
        pos_y: 1858.0,
        scale: 4.1,
    },
    "de_ancient" => MiniMapDefinition {
        pos_x: -2953.0,
        pos_y: 2164.0,
        scale: 5.0,
    },
    "de_anubis" => MiniMapDefinition {
        pos_x: -2796.0,
        pos_y: 3328.0,
        scale: 5.22,
    },
    "de_dust2" => MiniMapDefinition {
        pos_x: -2476.0,
        pos_y: 3239.0,
        scale: 4.4
    },
    "de_inferno" => MiniMapDefinition {
        pos_x: -2087.0,
        pos_y: 3870.0,
        scale: 4.9,
    },
    "de_mirage" => MiniMapDefinition {
        pos_x: -3230.0,
        pos_y: 1713.0,
        scale: 5.0,
    },
    "de_nuke" => MiniMapDefinition {
        pos_x: -3453.0,
        pos_y: 2887.0,
        scale: 7.0,
    },
    "de_overpass" => MiniMapDefinition {
        pos_x: -4831.0,
        pos_y: 1781.0,
        scale: 5.2,
    },
    "de_vertigo" => MiniMapDefinition {
        pos_x: -3168.0,
        pos_y: 1762.0,
        scale: 4.0,
    },
};

#[derive(Debug, PartialEq)]
struct MiniMapDefinition {
    pos_x: f32,
    pos_y: f32,
    scale: f32,
}

impl MiniMapDefinition {
    pub fn x_coord(&self, map_coord: f32) -> f32 {
        (map_coord * self.scale) + self.pos_x + analysis::heatmap::MAX_COORD
    }
    pub fn y_coord(&self, map_coord: f32) -> f32 {
        -(map_coord * self.scale) + self.pos_y + analysis::heatmap::MAX_COORD
    }
}
