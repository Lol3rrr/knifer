#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WinReason {
    StillInProgress,
    BombExploded,
    VipEscaped,
    VipKilled,
    TSaved,
    CtStoppedEscape,
    RoundEndReasonTerroristsStopped,
    BombDefused,
    TKilled,
    CTKilled,
    Draw,
    HostageRescued,
    TimeRanOut,
    RoundEndReasonHostagesNotRescued,
    TerroristsNotEscaped,
    VipNotEscaped,
    GameStart,
    TSurrender,
    CTSurrender,
    TPlanted,
    CTReachedHostage,
}

// https://github.com/markus-wa/demoinfocs-golang/blob/205b0bb25e9f3e96e1d306d154199b4a6292940e/pkg/demoinfocs/events/events.go#L53
pub static ROUND_WIN_REASON: phf::Map<i32, WinReason> = phf::phf_map! {
    0_i32 => WinReason::StillInProgress,
    1_i32 => WinReason::BombExploded,
    2_i32 => WinReason::VipEscaped,
    3_i32 => WinReason::VipKilled,
    4_i32 => WinReason::TSaved,
    5_i32 => WinReason::CtStoppedEscape,
    6_i32 => WinReason::RoundEndReasonTerroristsStopped,
    7_i32 => WinReason::BombDefused,
    8_i32 => WinReason::TKilled,
    9_i32 => WinReason::CTKilled,
    10_i32 => WinReason::Draw,
    11_i32 => WinReason::HostageRescued,
    12_i32 => WinReason::TimeRanOut,
    13_i32 => WinReason::RoundEndReasonHostagesNotRescued,
    14_i32 => WinReason::TerroristsNotEscaped,
    15_i32 => WinReason::VipNotEscaped,
    16_i32 => WinReason::GameStart,
    17_i32 => WinReason::TSurrender,
    18_i32 => WinReason::CTSurrender,
    19_i32 => WinReason::TPlanted,
    20_i32 => WinReason::CTReachedHostage,
};

#[derive(Debug)]
pub struct Round {
    pub winreason: WinReason,
    pub start: u32,
    pub end: u32,
    pub events: Vec<RoundEvent>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RoundEvent {
    BombPlanted,
    BombDefused,
    Kill { attacker: u64, died: u64 },
}

#[derive(Debug)]
pub struct PerRound {
    pub rounds: Vec<Round>,
}

pub fn parse(buf: &[u8]) -> Result<PerRound, ()> {
    let tmp = csdemo::Container::parse(buf).map_err(|e| ())?;
    let output = csdemo::parser::parse(
        csdemo::FrameIterator::parse(tmp.inner),
        csdemo::parser::EntityFilter::all(),
    )
    .map_err(|e| ())?;

    let mut rounds: Vec<Round> = Vec::new();
    for tick in output.entity_states.ticks.iter() {
        for state in tick.states.iter() {
            let round_start_count = state
                .get_prop("CCSGameRulesProxy.CCSGameRules.m_nRoundStartCount")
                .map(|v| v.value.as_u32())
                .flatten();
            if let Some(round_start_count) = round_start_count {
                if rounds.len() < (round_start_count - 1) as usize {
                    rounds.push(Round {
                        winreason: WinReason::StillInProgress,
                        start: tick.tick,
                        end: u32::MAX,
                        events: Vec::new(),
                    });
                }
            }

            let round_end_count = state
                .get_prop("CCSGameRulesProxy.CCSGameRules.m_nRoundEndCount")
                .map(|v| v.value.as_u32())
                .flatten();
            if let Some(round_end_count) = round_end_count {
                if rounds.len() == (round_end_count - 1) as usize {
                    rounds.last_mut().unwrap().end = tick.tick;
                }
            }

            if state.class.as_ref() == "CCSGameRulesProxy" {
                let round_win_reason = state
                    .get_prop("CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason")
                    .map(|p| p.value.as_i32())
                    .flatten()
                    .map(|v| ROUND_WIN_REASON.get(&v))
                    .flatten()
                    .filter(|r| !matches!(r, WinReason::StillInProgress));
                if let Some(round_win_reason) = round_win_reason {
                    rounds.last_mut().unwrap().winreason = round_win_reason.clone();
                }
            }
        }
    }

    let mut rounds_iter = rounds.iter_mut();

    let mut current_tick = 0;
    let mut current_round = rounds_iter.next().unwrap();
    'events: for event in output.events.iter() {
        match event {
            csdemo::DemoEvent::Tick(tick) => {
                current_tick = tick.tick();
            }
            csdemo::DemoEvent::GameEvent(ge) => {
                if current_tick < current_round.start {
                    continue;
                }
                while current_tick > current_round.end {
                    match rounds_iter.next() {
                        Some(r) => {
                            current_round = r;
                        }
                        None => break 'events,
                    };
                }

                let event = match ge.as_ref() {
                    csdemo::game_event::GameEvent::BombPlanted(planted) => RoundEvent::BombPlanted,
                    csdemo::game_event::GameEvent::BombDefused(defused) => RoundEvent::BombDefused,
                    csdemo::game_event::GameEvent::PlayerDeath(death) => {
                        let died = match death.userid {
                            Some(d) => d,
                            None => continue,
                        };
                        let attacker = match death.attacker.filter(|p| p.0 <= 10) {
                            Some(a) => a,
                            None => died.clone(),
                        };

                        let died_player = output.player_info.get(&died).unwrap();
                        let attacker_player = output.player_info.get(&attacker).unwrap();

                        RoundEvent::Kill {
                            attacker: attacker_player.xuid,
                            died: died_player.xuid,
                        }
                    }
                    _ => continue,
                };

                current_round.events.push(event);
            }
            _ => {}
        };
    }

    Ok(PerRound { rounds })
}
