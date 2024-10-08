#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BaseDemoInfo {
    pub id: i64,
    pub map: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserStatus {
    pub name: String,
    pub steamid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoInfo {
    pub id: i64,
    pub map: String,
}

pub mod demo_analysis {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct ScoreBoard {
        pub team1: Vec<ScoreBoardPlayer>,
        pub team2: Vec<ScoreBoardPlayer>,
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
        pub png_data: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct DemoRound {
        pub reason: RoundWinReason,
        pub events: Vec<RoundEvent>
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
        },
    }
}
