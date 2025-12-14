use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub owner_scope: ZoneOwnerScope,
    pub visibility: ZoneVisibility,
    pub ordered: bool,
    pub allow_duplicates: bool,
    pub default_capacity: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoneOwnerScope {
    Player,
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoneVisibility {
    Public,
    Private,
    TopCardPublic,
}
