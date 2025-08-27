export interface ChessUser {
  username: string;
  elo: number;
  games_played: number;
  wins: number;
  draws: number;
  losses: number;
  current_games: string[];
  created_at: string;
}

export interface ChessGame {
  id: string;
  white: string;
  black: string;
  moves: string;
  current_fen: string;  // Current board position in FEN notation
  status: 'active' | 'checkmate_claimed' | 'disputed' | 'white_won' | 'black_won' | 'draw' | 'stalemate' | 'timeout';
  current_turn: 'white' | 'black';
  last_move_block: number;
  white_time_remaining: number;  // Blocks remaining for white
  black_time_remaining: number;  // Blocks remaining for black
  created_block: number;
  claim_block: number | null;
  time_control: string;
  move_count: number;  // Total moves made in the game
  draw_proposed_by: string | null;  // Address of player who proposed draw
}

export interface TimeStatus {
  white_time_remaining: number;
  black_time_remaining: number;
  current_player: string;
  time_expired: boolean;
  move_count: number;
  time_since_last_move: number;
}

export interface ChessMove {
  from: string;
  to: string;
  promotion?: string;
  resulting_fen?: string;  // FEN after this move is applied
}