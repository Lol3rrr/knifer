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
}
