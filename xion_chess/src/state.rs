use cosmwasm_std::Addr;
use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;

#[cw_serde]
pub struct UserProfile {
    pub username: String,
    pub elo: u32,
    pub games_played: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub current_games: Vec<String>,  // Active game IDs
    pub created_at: u64,  // Block height when created
}

impl Default for UserProfile {
    fn default() -> Self {
        UserProfile {
            username: String::new(),
            elo: 1200,  // Starting ELO
            games_played: 0,
            wins: 0,
            draws: 0,
            losses: 0,
            current_games: Vec::new(),
            created_at: 0,
        }
    }
}

#[cw_serde]
pub struct ChessGame {
    pub id: String,
    pub white: Addr,
    pub black: Addr,
    pub moves: String,
    pub current_fen: String,
    pub status: String,
    pub current_turn: String,
    pub last_move_block: u64,
    pub white_time_remaining: u64,  // Blocks remaining for white
    pub black_time_remaining: u64,  // Blocks remaining for black
    pub created_block: u64,
    pub claim_block: Option<u64>,
    pub time_control: String,
    pub move_count: u32,  // Track total moves for increment calculation
    pub draw_proposed_by: Option<String>,  // Address of player who proposed draw
}

// Game storage: game_id -> ChessGame
pub const GAMES: Map<String, ChessGame> = Map::new("games");

// Index of all game IDs for listing
pub const GAME_IDS: Map<String, bool> = Map::new("game_ids");

// User profiles: address -> UserProfile
pub const USER_PROFILES: Map<Addr, UserProfile> = Map::new("user_profiles");

// Index of all user addresses
pub const USER_ADDRESSES: Map<Addr, bool> = Map::new("user_addresses");
