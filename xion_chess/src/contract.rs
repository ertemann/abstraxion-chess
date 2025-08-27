#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use shakmaty::{Chess, Position, Move, Role};
use shakmaty::fen::Fen;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GameStatus, VerificationResponse, MoveValidationResponse, GameResponse, GamesResponse, GameIdsResponse, TimeStatusResponse, UserProfileResponse, UsersResponse};
use crate::state::{ChessGame, UserProfile, GAMES, GAME_IDS, USER_PROFILES, USER_ADDRESSES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:xion_chess";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitializeUser { username } => {
            execute::initialize_user(deps, env, info, username)
        }
        ExecuteMsg::VerifyPosition { fen, claimed_status } => {
            execute::verify_position(fen, claimed_status)
        }
        ExecuteMsg::CreateGame { game_id, opponent, time_control } => {
            execute::create_game(deps, env, info, game_id, opponent, time_control)
        }
        ExecuteMsg::MakeMove { game_id, from, to, promotion } => {
            execute::make_move(deps, env, info, game_id, from, to, promotion)
        }
        ExecuteMsg::UpdateGameStatus { game_id, status } => {
            execute::update_game_status(deps, info, game_id, status)
        }
        ExecuteMsg::ResignGame { game_id } => {
            execute::resign_game(deps, info, game_id)
        }
        ExecuteMsg::ProposeDrawRequest { game_id } => {
            execute::propose_draw(deps, info, game_id)
        }
        ExecuteMsg::RespondToDrawRequest { game_id, accept } => {
            execute::respond_to_draw(deps, info, game_id, accept)
        }
    }
}

pub mod execute {
    use super::*;

    /// Calculate new ELO ratings after a game using integer arithmetic
    /// Uses scaled integers (x1000) to avoid floating point operations
    /// k_factor is 32, scores: win=1000, draw=500, loss=0
    fn calculate_elo(winner_elo: u32, loser_elo: u32, is_draw: bool) -> (u32, u32) {
        let k_factor = 32u32;
        let scale = 1000u32; // Scale factor for precision
        
        // Calculate rating difference (clamped to prevent overflow)
        let rating_diff = if winner_elo >= loser_elo {
            (winner_elo - loser_elo).min(800) // Cap at 800 to prevent overflow
        } else {
            (loser_elo - winner_elo).min(800)
        };
        
        // Simplified ELO expected score calculation using integer approximation
        // For rating differences, use lookup table approach
        let expected_winner_scaled = match rating_diff {
            0..=25 => 500,      // ~0.50
            26..=50 => 537,     // ~0.537
            51..=100 => 640,    // ~0.64
            101..=150 => 691,   // ~0.691
            151..=200 => 760,   // ~0.76
            201..=300 => 849,   // ~0.849
            301..=400 => 909,   // ~0.909
            _ => 950,           // ~0.95 for large differences
        };
        
        // If loser had higher rating, invert expectation
        let (winner_expected, loser_expected) = if winner_elo >= loser_elo {
            (expected_winner_scaled, scale - expected_winner_scaled)
        } else {
            (scale - expected_winner_scaled, expected_winner_scaled)
        };
        
        // Set actual scores
        let (winner_actual, loser_actual) = if is_draw {
            (scale / 2, scale / 2) // Both get 0.5
        } else {
            (scale, 0) // Winner gets 1.0, loser gets 0.0
        };
        
        // Calculate ELO changes: K * (actual - expected) / scale
        // Use signed arithmetic for the differences
        let winner_change = (k_factor as i32 * (winner_actual as i32 - winner_expected as i32)) / scale as i32;
        let loser_change = (k_factor as i32 * (loser_actual as i32 - loser_expected as i32)) / scale as i32;
        
        // Apply changes (ensure no underflow below 100)
        let new_winner_elo = if winner_change >= 0 {
            winner_elo + winner_change as u32
        } else {
            winner_elo.saturating_sub((-winner_change) as u32)
        };
        
        let new_loser_elo = if loser_change >= 0 {
            loser_elo + loser_change as u32
        } else {
            loser_elo.saturating_sub((-loser_change) as u32)
        };
        
        // Ensure ratings don't go below 100 (minimum rating)
        (new_winner_elo.max(100), new_loser_elo.max(100))
    }

    /// Update player profiles after game ends
    fn update_profiles_after_game(
        deps: &mut DepsMut,
        game: &ChessGame,
        white_won: bool,
        black_won: bool,
        is_draw: bool,
    ) -> Result<(), ContractError> {
        let white_addr = game.white.clone();
        let black_addr = game.black.clone();
        
        // Load both profiles
        let mut white_profile = USER_PROFILES.load(deps.storage, white_addr.clone())?;
        let mut black_profile = USER_PROFILES.load(deps.storage, black_addr.clone())?;
        
        // Calculate new ELO ratings
        let (new_white_elo, new_black_elo) = if is_draw {
            calculate_elo(white_profile.elo, black_profile.elo, true)
        } else if white_won {
            calculate_elo(white_profile.elo, black_profile.elo, false)
        } else {
            let (black_new, white_new) = calculate_elo(black_profile.elo, white_profile.elo, false);
            (white_new, black_new)
        };
        
        // Update white profile
        white_profile.elo = new_white_elo;
        white_profile.games_played += 1;
        if white_won {
            white_profile.wins += 1;
        } else if black_won {
            white_profile.losses += 1;
        } else {
            white_profile.draws += 1;
        }
        white_profile.current_games.retain(|id| id != &game.id);
        
        // Update black profile
        black_profile.elo = new_black_elo;
        black_profile.games_played += 1;
        if black_won {
            black_profile.wins += 1;
        } else if white_won {
            black_profile.losses += 1;
        } else {
            black_profile.draws += 1;
        }
        black_profile.current_games.retain(|id| id != &game.id);
        
        // Save updated profiles
        USER_PROFILES.save(deps.storage, white_addr, &white_profile)?;
        USER_PROFILES.save(deps.storage, black_addr, &black_profile)?;
        
        Ok(())
    }

    pub fn initialize_user(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        username: String,
    ) -> Result<Response, ContractError> {
        let sender = info.sender.clone();
        
        // Load or create user profile
        let mut profile = USER_PROFILES.may_load(deps.storage, sender.clone())?
            .unwrap_or_else(|| UserProfile {
                username: username.clone(),
                elo: 1200,
                games_played: 0,
                wins: 0,
                draws: 0,
                losses: 0,
                current_games: Vec::new(),
                created_at: env.block.height,
            });
        
        // Update username if provided
        if !username.is_empty() {
            profile.username = username;
        }
        
        // Save profile
        USER_PROFILES.save(deps.storage, sender.clone(), &profile)?;
        USER_ADDRESSES.save(deps.storage, sender.clone(), &true)?;
        
        Ok(Response::new()
            .add_attribute("action", "initialize_user")
            .add_attribute("user", sender)
            .add_attribute("username", profile.username))
    }

    pub fn verify_position(fen: String, claimed_status: GameStatus) -> Result<Response, ContractError> {
        let verification = query::verify_position_internal(fen.clone())?;
        
        // Check if claimed status matches actual status
        let status_matches = match (&claimed_status, &verification.status) {
            (GameStatus::Checkmate, GameStatus::Checkmate) => true,
            (GameStatus::Stalemate, GameStatus::Stalemate) => true,
            (GameStatus::Draw, GameStatus::Draw) => true,
            (GameStatus::Active, GameStatus::Active) => true,
            _ => false,
        };

        if !status_matches {
            return Err(ContractError::InvalidClaim {
                claimed: format!("{:?}", claimed_status),
                actual: format!("{:?}", verification.status),
            });
        }

        Ok(Response::new()
            .add_attribute("action", "verify_position")
            .add_attribute("fen", fen)
            .add_attribute("status", format!("{:?}", verification.status))
            .add_attribute("is_check", verification.is_check.to_string()))
    }

    pub fn create_game(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        game_id: String,
        opponent: cosmwasm_std::Addr,
        time_control: String,
    ) -> Result<Response, ContractError> {
        // Check if game already exists
        if GAMES.has(deps.storage, game_id.clone()) {
            return Err(ContractError::GameAlreadyExists { id: game_id });
        }

        // Ensure both players have profiles
        let white_addr = info.sender.clone();
        let black_addr = opponent.clone();
        
        // Initialize white player if needed
        let mut white_profile = USER_PROFILES.may_load(deps.storage, white_addr.clone())?
            .unwrap_or_else(|| UserProfile {
                username: white_addr.to_string(),
                elo: 1200,
                games_played: 0,
                wins: 0,
                draws: 0,
                losses: 0,
                current_games: Vec::new(),
                created_at: env.block.height,
            });
        
        // Initialize black player if needed
        let mut black_profile = USER_PROFILES.may_load(deps.storage, black_addr.clone())?
            .unwrap_or_else(|| UserProfile {
                username: black_addr.to_string(),
                elo: 1200,
                games_played: 0,
                wins: 0,
                draws: 0,
                losses: 0,
                current_games: Vec::new(),
                created_at: env.block.height,
            });
        
        // Add game to both players' current games
        white_profile.current_games.push(game_id.clone());
        black_profile.current_games.push(game_id.clone());
        
        // Save updated profiles
        USER_PROFILES.save(deps.storage, white_addr.clone(), &white_profile)?;
        USER_PROFILES.save(deps.storage, black_addr.clone(), &black_profile)?;
        USER_ADDRESSES.save(deps.storage, white_addr.clone(), &true)?;
        USER_ADDRESSES.save(deps.storage, black_addr.clone(), &true)?;

        // Time control: ~2 days total per player (172,800 blocks at 1 block/second)
        let initial_time = 172_800u64;
        
        let game = ChessGame {
            id: game_id.clone(),
            white: info.sender.clone(),
            black: opponent.clone(),
            moves: String::new(),
            current_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            status: "active".to_string(),
            current_turn: "white".to_string(),
            last_move_block: env.block.height,
            white_time_remaining: initial_time,
            black_time_remaining: initial_time,
            created_block: env.block.height,
            claim_block: None,
            time_control,
            move_count: 0,
            draw_proposed_by: None,
        };

        GAMES.save(deps.storage, game_id.clone(), &game)?;
        GAME_IDS.save(deps.storage, game_id.clone(), &true)?;

        Ok(Response::new()
            .add_attribute("action", "create_game")
            .add_attribute("game_id", game_id)
            .add_attribute("white", info.sender)
            .add_attribute("black", opponent))
    }

    pub fn make_move(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        game_id: String,
        from: String,
        to: String,
        promotion: Option<String>,
    ) -> Result<Response, ContractError> {
        let mut game = GAMES.load(deps.storage, game_id.clone())?;

        // Check if it's the player's turn
        let is_white = game.white == info.sender;
        let is_black = game.black == info.sender;
        
        if !is_white && !is_black {
            return Err(ContractError::NotPlayerInGame {});
        }

        if (is_white && game.current_turn != "white") || (is_black && game.current_turn != "black") {
            return Err(ContractError::NotYourTurn {});
        }

        // Time control logic with increments
        // Only enforce time after both players have made their first move (move_count >= 2)
        if game.move_count >= 2 {
            // Calculate time used since last move
            let time_used = env.block.height.saturating_sub(game.last_move_block);
            
            // Check if current player has enough time
            let current_time_remaining = if is_white {
                game.white_time_remaining
            } else {
                game.black_time_remaining
            };
            
            if time_used >= current_time_remaining {
                // Player has run out of time - they lose
                let white_won = !is_white;  // Opponent wins
                let black_won = !is_black;  // Opponent wins
                game.status = if is_white { "black_won" } else { "white_won" }.to_string();
                
                // Update ELO ratings for timeout loss
                update_profiles_after_game(&mut deps, &game, white_won, black_won, false)?;
                
                GAMES.save(deps.storage, game_id.clone(), &game)?;
                
                return Err(ContractError::IllegalMove { 
                    error: "Time expired - you have lost the game".to_string() 
                });
            }
            
            // Deduct time used from current player's clock
            if is_white {
                game.white_time_remaining = game.white_time_remaining.saturating_sub(time_used);
            } else {
                game.black_time_remaining = game.black_time_remaining.saturating_sub(time_used);
            }
        }

        // Validate the move using existing chess logic
        let move_validation = query::validate_move(game.current_fen.clone(), from.clone(), to.clone(), promotion.clone())?;
        
        if !move_validation.is_valid {
            return Err(ContractError::IllegalMove { 
                error: move_validation.error.unwrap_or("Unknown error".to_string()) 
            });
        }

        // Update game state
        let move_string = format!("{}{}{}", from, to, promotion.unwrap_or_default());
        game.moves = if game.moves.is_empty() {
            move_string
        } else {
            format!("{},{}", game.moves, move_string)
        };
        
        if let Some(new_fen) = move_validation.resulting_fen {
            game.current_fen = new_fen.clone();
            
            // Check for checkmate/stalemate/draw after the move
            let position_check = query::verify_position_internal(new_fen)?;
            match position_check.status {
                GameStatus::Checkmate => {
                    // The current player (who just moved) wins by checkmate
                    let white_won = game.current_turn == "white";
                    let black_won = game.current_turn == "black";
                    game.status = if white_won { "white_won" } else { "black_won" }.to_string();
                    
                    // Update ELO ratings
                    update_profiles_after_game(&mut deps, &game, white_won, black_won, false)?;
                },
                GameStatus::Stalemate => {
                    game.status = "draw".to_string();
                    
                    // Update ELO ratings for draw
                    update_profiles_after_game(&mut deps, &game, false, false, true)?;
                },
                GameStatus::Draw => {
                    game.status = "draw".to_string();
                    
                    // Update ELO ratings for draw
                    update_profiles_after_game(&mut deps, &game, false, false, true)?;
                },
                GameStatus::Active => {
                    // Game continues
                }
            }
        }
        
        // Add time increment after successful move
        // First 20 moves: 10 minutes (600 blocks) increment
        // After 20 moves: 1 minute (60 blocks) increment
        if game.move_count >= 2 {  // Only add increment after both players have made first move
            let increment = if game.move_count <= 20 {
                600u64  // 10 minutes for moves 1-20
            } else {
                60u64   // 1 minute for moves 21+
            };
            
            // Add increment to the player who just moved
            if is_white {
                game.white_time_remaining = game.white_time_remaining.saturating_add(increment);
            } else {
                game.black_time_remaining = game.black_time_remaining.saturating_add(increment);
            }
        }
        
        // Update move count and last move block
        game.move_count += 1;
        game.last_move_block = env.block.height;
        
        // Only switch turns if game is still active
        if game.status == "active" {
            game.current_turn = if game.current_turn == "white" { "black" } else { "white" }.to_string();
        }

        GAMES.save(deps.storage, game_id.clone(), &game)?;

        Ok(Response::new()
            .add_attribute("action", "make_move")
            .add_attribute("game_id", game_id)
            .add_attribute("player", info.sender)
            .add_attribute("move", format!("{}{}", from, to)))
    }

    pub fn update_game_status(
        mut deps: DepsMut,
        info: MessageInfo,
        game_id: String,
        status: String,
    ) -> Result<Response, ContractError> {
        let mut game = GAMES.load(deps.storage, game_id.clone())?;

        // Check if sender is a player in the game
        if game.white != info.sender && game.black != info.sender {
            return Err(ContractError::NotPlayerInGame {});
        }

        let old_status = game.status.clone();
        game.status = status.clone();
        
        // Handle ELO updates for game-ending status changes
        if old_status == "active" {
            match status.as_str() {
                "white_won" => {
                    update_profiles_after_game(&mut deps, &game, true, false, false)?;
                },
                "black_won" => {
                    update_profiles_after_game(&mut deps, &game, false, true, false)?;
                },
                "draw" => {
                    update_profiles_after_game(&mut deps, &game, false, false, true)?;
                },
                _ => {
                    // Other status changes don't affect ELO
                }
            }
        }

        GAMES.save(deps.storage, game_id.clone(), &game)?;

        Ok(Response::new()
            .add_attribute("action", "update_game_status")
            .add_attribute("game_id", game_id)
            .add_attribute("status", status)
            .add_attribute("player", info.sender))
    }

    pub fn resign_game(
        mut deps: DepsMut,
        info: MessageInfo,
        game_id: String,
    ) -> Result<Response, ContractError> {
        let mut game = GAMES.load(deps.storage, game_id.clone())?;

        // Check if sender is a player in the game
        if game.white != info.sender && game.black != info.sender {
            return Err(ContractError::NotPlayerInGame {});
        }

        // Can only resign active games
        if game.status != "active" {
            return Err(ContractError::GameNotActive {});
        }

        // Determine winner based on who resigned
        let is_white = game.white == info.sender;
        let white_won = !is_white;  // Opponent wins
        let black_won = is_white;   // Opponent wins
        
        game.status = if is_white { "black_won" } else { "white_won" }.to_string();
        
        // Update ELO ratings for resignation
        update_profiles_after_game(&mut deps, &game, white_won, black_won, false)?;

        GAMES.save(deps.storage, game_id.clone(), &game)?;

        Ok(Response::new()
            .add_attribute("action", "resign_game")
            .add_attribute("game_id", game_id)
            .add_attribute("resigned_player", info.sender)
            .add_attribute("winner", if is_white { game.black.to_string() } else { game.white.to_string() }))
    }

    pub fn propose_draw(
        deps: DepsMut,
        info: MessageInfo,
        game_id: String,
    ) -> Result<Response, ContractError> {
        let mut game = GAMES.load(deps.storage, game_id.clone())?;

        // Check if sender is a player in the game
        if game.white != info.sender && game.black != info.sender {
            return Err(ContractError::NotPlayerInGame {});
        }

        // Can only propose draw in active games
        if game.status != "active" {
            return Err(ContractError::GameNotActive {});
        }

        // Check if draw already proposed by this player
        if let Some(ref proposer) = game.draw_proposed_by {
            if proposer == &info.sender.to_string() {
                return Err(ContractError::DrawAlreadyProposed {});
            }
        }

        game.draw_proposed_by = Some(info.sender.to_string());
        GAMES.save(deps.storage, game_id.clone(), &game)?;

        Ok(Response::new()
            .add_attribute("action", "propose_draw")
            .add_attribute("game_id", game_id)
            .add_attribute("proposed_by", info.sender))
    }

    pub fn respond_to_draw(
        mut deps: DepsMut,
        info: MessageInfo,
        game_id: String,
        accept: bool,
    ) -> Result<Response, ContractError> {
        let mut game = GAMES.load(deps.storage, game_id.clone())?;

        // Check if sender is a player in the game
        if game.white != info.sender && game.black != info.sender {
            return Err(ContractError::NotPlayerInGame {});
        }

        // Can only respond to draw proposals in active games
        if game.status != "active" {
            return Err(ContractError::GameNotActive {});
        }

        // Check if there's a draw proposal
        let draw_proposer = game.draw_proposed_by.clone()
            .ok_or(ContractError::NoDrawProposal {})?;

        // Can't respond to your own draw proposal
        if draw_proposer == info.sender.to_string() {
            return Err(ContractError::CannotRespondToOwnProposal {});
        }

        if accept {
            // Accept draw - game ends in draw
            game.status = "draw".to_string();
            game.draw_proposed_by = None;
            
            // Update ELO ratings for draw
            update_profiles_after_game(&mut deps, &game, false, false, true)?;
            
            GAMES.save(deps.storage, game_id.clone(), &game)?;

            Ok(Response::new()
                .add_attribute("action", "accept_draw")
                .add_attribute("game_id", game_id)
                .add_attribute("accepted_by", info.sender)
                .add_attribute("result", "draw"))
        } else {
            // Decline draw - clear proposal and continue game
            game.draw_proposed_by = None;
            GAMES.save(deps.storage, game_id.clone(), &game)?;

            Ok(Response::new()
                .add_attribute("action", "decline_draw")
                .add_attribute("game_id", game_id)
                .add_attribute("declined_by", info.sender))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyPosition { fen } => {
            to_json_binary(&query::verify_position_internal(fen)?)
        }
        QueryMsg::ValidateMove { current_fen, move_from, move_to, promotion } => {
            to_json_binary(&query::validate_move(current_fen, move_from, move_to, promotion)?)
        }
        QueryMsg::GetGame { game_id } => {
            to_json_binary(&query::get_game(deps, game_id)?)
        }
        QueryMsg::GetPlayerGames { player } => {
            to_json_binary(&query::get_player_games(deps, player)?)
        }
        QueryMsg::GetAllGameIds {} => {
            to_json_binary(&query::get_all_game_ids(deps)?)
        }
        QueryMsg::CheckTimeStatus { game_id } => {
            to_json_binary(&query::check_time_status(deps, env, game_id)?)
        }
        QueryMsg::GetUserProfile { address } => {
            to_json_binary(&query::get_user_profile(deps, address)?)
        }
        QueryMsg::GetAllUsers {} => {
            to_json_binary(&query::get_all_users(deps)?)
        }
    }
}

pub mod query {
    use super::*;
    use cosmwasm_std::Addr;
    use shakmaty::san::SanPlus;

    pub fn verify_position_internal(fen: String) -> StdResult<VerificationResponse> {
        // Parse FEN into chess position
        let fen_parsed: Fen = fen.parse()
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid FEN format"))?;
        
        let pos: Chess = fen_parsed.into_position(shakmaty::CastlingMode::Standard)
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid chess position"))?;

        // Check if king is in check
        let is_check = pos.checkers().any();

        // Get all legal moves
        let legal_moves: Vec<String> = pos.legal_moves().iter()
            .map(|m| SanPlus::from_move(pos.clone(), m).to_string())
            .collect();

        // Determine game status
        let status = if legal_moves.is_empty() {
            if is_check {
                GameStatus::Checkmate
            } else {
                GameStatus::Stalemate
            }
        } else if pos.is_insufficient_material() {
            GameStatus::Draw
        } else {
            GameStatus::Active
        };

        Ok(VerificationResponse {
            status,
            is_check,
            legal_moves,
        })
    }

    pub fn validate_move(
        current_fen: String,
        move_from: String,
        move_to: String,
        promotion: Option<String>,
    ) -> StdResult<MoveValidationResponse> {
        // Parse current position
        let fen_parsed: Fen = current_fen.parse()
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid FEN format"))?;
        
        let mut pos: Chess = fen_parsed.into_position(shakmaty::CastlingMode::Standard)
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid chess position"))?;

        // Parse squares
        let from_square = move_from.parse()
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid from square"))?;
        let to_square = move_to.parse()
            .map_err(|_| cosmwasm_std::StdError::generic_err("Invalid to square"))?;

        // Parse promotion piece if provided
        let promotion_role = if let Some(promo) = promotion {
            match promo.to_lowercase().as_str() {
                "q" => Some(Role::Queen),
                "r" => Some(Role::Rook),
                "b" => Some(Role::Bishop),
                "n" => Some(Role::Knight),
                _ => return Ok(MoveValidationResponse {
                    is_valid: false,
                    resulting_fen: None,
                    error: Some("Invalid promotion piece".to_string()),
                }),
            }
        } else {
            None
        };

        // Get the piece info from current position
        let piece = pos.board().piece_at(from_square);
        let capture_role = pos.board().piece_at(to_square).map(|p| p.role);
        
        if piece.is_none() {
            return Ok(MoveValidationResponse {
                is_valid: false,
                resulting_fen: None,
                error: Some("No piece at from square".to_string()),
            });
        }

        // Create move
        let chess_move = Move::Normal {
            from: from_square,
            to: to_square,
            capture: capture_role,
            promotion: promotion_role,
            role: piece.unwrap().role,
        };

        // Check if move is legal (follows all chess rules)
        if pos.is_legal(&chess_move) {
            // Make the move and get resulting FEN
            pos.play_unchecked(&chess_move);
            let resulting_fen = Fen::from_position(pos, shakmaty::EnPassantMode::Legal).to_string();

            Ok(MoveValidationResponse {
                is_valid: true,
                resulting_fen: Some(resulting_fen),
                error: None,
            })
        } else {
            Ok(MoveValidationResponse {
                is_valid: false,
                resulting_fen: None,
                error: Some("Illegal move".to_string()),
            })
        }
    }

    pub fn get_game(deps: Deps, game_id: String) -> StdResult<GameResponse> {
        let game = GAMES.may_load(deps.storage, game_id)?;
        Ok(GameResponse { game })
    }

    pub fn get_player_games(deps: Deps, player: cosmwasm_std::Addr) -> StdResult<GamesResponse> {
        let all_game_ids: Vec<String> = GAME_IDS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        let mut player_games = Vec::new();
        
        for game_id in all_game_ids {
            if let Some(game) = GAMES.may_load(deps.storage, game_id)? {
                if game.white == player || game.black == player {
                    player_games.push(game);
                }
            }
        }

        Ok(GamesResponse { games: player_games })
    }

    pub fn get_all_game_ids(deps: Deps) -> StdResult<GameIdsResponse> {
        let game_ids: Vec<String> = GAME_IDS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        Ok(GameIdsResponse { game_ids })
    }

    pub fn check_time_status(deps: Deps, env: Env, game_id: String) -> StdResult<TimeStatusResponse> {
        let game = GAMES.load(deps.storage, game_id)?;
        
        // Calculate time elapsed since last move
        let time_since_last_move = env.block.height.saturating_sub(game.last_move_block);
        
        // Calculate actual time remaining for each player
        let (white_time_remaining, black_time_remaining) = if game.move_count >= 2 {
            // Time is being tracked after both players' first moves
            let current_player_is_white = game.current_turn == "white";
            
            if current_player_is_white {
                // White's turn - deduct time used from their clock
                let white_remaining = game.white_time_remaining.saturating_sub(time_since_last_move);
                (white_remaining, game.black_time_remaining)
            } else {
                // Black's turn - deduct time used from their clock
                let black_remaining = game.black_time_remaining.saturating_sub(time_since_last_move);
                (game.white_time_remaining, black_remaining)
            }
        } else {
            // Time not tracked yet - return full time
            (game.white_time_remaining, game.black_time_remaining)
        };
        
        // Check if current player has run out of time
        let time_expired = if game.move_count >= 2 {
            let current_time = if game.current_turn == "white" {
                white_time_remaining
            } else {
                black_time_remaining
            };
            current_time == 0
        } else {
            false
        };
        
        Ok(TimeStatusResponse {
            white_time_remaining,
            black_time_remaining,
            current_player: game.current_turn.clone(),
            time_expired,
            move_count: game.move_count,
            time_since_last_move,
        })
    }

    pub fn get_user_profile(deps: Deps, address: Addr) -> StdResult<UserProfileResponse> {
        let profile = USER_PROFILES.may_load(deps.storage, address)?;
        Ok(UserProfileResponse { profile })
    }

    pub fn get_all_users(deps: Deps) -> StdResult<UsersResponse> {
        let users: Vec<Addr> = USER_ADDRESSES
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        Ok(UsersResponse { users })
    }
}

