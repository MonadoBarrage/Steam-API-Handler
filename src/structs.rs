use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OwnedGames {
    pub response: GameResponse
}

#[derive(Serialize, Deserialize)]
pub struct GameResponse {
    pub game_count: i64,
    pub games: Vec<GameDetails>
}
#[derive(Serialize, Deserialize)]
pub struct ItemDataJson{
    pub ids: Vec<GameDetails>,
    pub context: ContextDetails,
    pub data_request: DataRequestDetails
}

#[derive(Serialize, Deserialize)]
pub struct ContextDetails {
    pub language: String,
    pub country_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct DataRequestDetails {
    pub include_assets: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GameDetails {
    pub appid: i64,
}