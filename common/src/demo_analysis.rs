#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScoreBoard {
    pub teams: Vec<ScoreBoardTeam>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScoreBoardTeam {
    pub number: u32,
    pub score: i16,
    pub players: Vec<ScoreBoardPlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScoreBoardPlayer {
    pub name: String,
    pub kills: usize,
    pub deaths: usize,
    pub damage: usize,
    pub assists: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerHeatmap {
    pub name: String,
    pub team: String,
    pub png_data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerRoundResult {
    pub teams: Vec<PerRoundTeam>,
    pub rounds: Vec<DemoRound>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerRoundTeam {
    pub name: String,
    pub number: u32,
    pub players: std::collections::HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoRound {
    pub reason: RoundWinReason,
    pub events: Vec<RoundEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RoundWinReason {
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RoundEvent {
    BombPlanted,
    BombDefused,
    Killed {
        attacker: String,
        died: String,
        weapon: Option<String>,
        noscope: bool,
        headshot: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HeadToHead {
    pub row_players: Vec<String>,
    pub column_players: Vec<String>,
    pub entries: Vec<Vec<(i16, i16)>>,
}
