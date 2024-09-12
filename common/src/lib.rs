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
