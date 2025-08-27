use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::{ChessGame, UserProfile};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Initialize or update user profile
    InitializeUser {
        username: String,
    },
    /// Verify a chess position for checkmate/stalemate
    VerifyPosition {
        fen: String,
        claimed_status: GameStatus,
    },
    /// Create a new chess game
    CreateGame {
        game_id: String,
        opponent: Addr,
        time_control: String,
    },
    /// Make a move in a chess game
    MakeMove {
        game_id: String,
        from: String,
        to: String,
        promotion: Option<String>,
    },
    /// Update game status (resign, claim victory, etc.)
    UpdateGameStatus {
        game_id: String,
        status: String,
    },
    /// Resign from a game
    ResignGame {
        game_id: String,
    },
    /// Propose a draw
    ProposeDrawRequest {
        game_id: String,
    },
    /// Accept or decline a draw proposal
    RespondToDrawRequest {
        game_id: String,
        accept: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Verify if a FEN position is checkmate/stalemate/ongoing
    #[returns(VerificationResponse)]
    VerifyPosition { fen: String },
    
    /// Check if a move is legal from current position
    #[returns(MoveValidationResponse)]
    ValidateMove { 
        current_fen: String,
        move_from: String,
        move_to: String,
        promotion: Option<String>,
    },

    /// Get a specific game by ID
    #[returns(GameResponse)]
    GetGame { game_id: String },

    /// Get all games for a player
    #[returns(GamesResponse)]
    GetPlayerGames { player: Addr },

    /// Get all game IDs
    #[returns(GameIdsResponse)]
    GetAllGameIds {},

    /// Check if a player's time has expired
    #[returns(TimeStatusResponse)]
    CheckTimeStatus { game_id: String },

    /// Get user profile
    #[returns(UserProfileResponse)]
    GetUserProfile { address: Addr },

    /// Get all users
    #[returns(UsersResponse)]
    GetAllUsers {},
}

#[cw_serde]
pub enum GameStatus {
    Active,
    Checkmate,
    Stalemate,
    Draw,
}

#[cw_serde]
pub struct VerificationResponse {
    pub status: GameStatus,
    pub is_check: bool,
    pub legal_moves: Vec<String>,
}

#[cw_serde]
pub struct MoveValidationResponse {
    pub is_valid: bool,
    pub resulting_fen: Option<String>,
    pub error: Option<String>,
}

#[cw_serde]
pub struct GameResponse {
    pub game: Option<ChessGame>,
}

#[cw_serde]
pub struct GamesResponse {
    pub games: Vec<ChessGame>,
}

#[cw_serde]
pub struct GameIdsResponse {
    pub game_ids: Vec<String>,
}

#[cw_serde]
pub struct TimeStatusResponse {
    pub white_time_remaining: u64,
    pub black_time_remaining: u64,
    pub current_player: String,
    pub time_expired: bool,
    pub move_count: u32,
    pub time_since_last_move: u64,
}

#[cw_serde]
pub struct UserProfileResponse {
    pub profile: Option<UserProfile>,
}

#[cw_serde]
pub struct UsersResponse {
    pub users: Vec<Addr>,
}
