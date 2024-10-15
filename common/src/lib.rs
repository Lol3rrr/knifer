#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BaseDemoInfo {
    pub id: String,
    pub map: String,
    pub team2_score: i16,
    pub team3_score: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserStatus {
    pub name: String,
    pub steamid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoInfo {
    pub id: String,
    pub map: String,
}

pub mod demo_analysis;
