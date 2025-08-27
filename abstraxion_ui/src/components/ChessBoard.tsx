"use client";
import React, { useState, useEffect } from "react";
import { Chess } from "chess.js";
import type { ChessGame, ChessMove } from "@/types/chess";

interface ChessBoardProps {
  game: ChessGame | null;
  currentPlayer: string;
  onMove: (move: ChessMove) => Promise<boolean>;
  disabled?: boolean;
  moveError?: string | null;
  onClearError?: () => void;
}

export default function ChessBoard({ game, currentPlayer, onMove, disabled, moveError, onClearError }: ChessBoardProps) {
  const [chess] = useState(new Chess());
  const [board, setBoard] = useState<string[][]>([]);
  const [selectedSquare, setSelectedSquare] = useState<string | null>(null);
  const [possibleMoves, setPossibleMoves] = useState<string[]>([]);
  const [turn, setTurn] = useState<'w' | 'b'>('w');
  const [pendingMove, setPendingMove] = useState<{ from: string; to: string } | null>(null);
  const [previousFen, setPreviousFen] = useState<string | null>(null);

  useEffect(() => {
    if (game?.current_fen) {
      // Load position from FEN instead of replaying all moves
      try {
        chess.load(game.current_fen);
        setPreviousFen(game.current_fen); // Store for rollback
      } catch (e) {
        console.error("Invalid FEN:", game.current_fen);
        // Fallback to replaying moves
        if (game?.moves) {
          chess.reset();
          const moves = game.moves.split(',').filter(Boolean);
          moves.forEach(move => {
            try {
              chess.move(move);
            } catch (e) {
              console.error("Invalid move:", move);
            }
          });
          setPreviousFen(chess.fen());
        }
      }
    } else if (game?.moves) {
      // Fallback: replay moves if no FEN available
      chess.reset();
      const moves = game.moves.split(',').filter(Boolean);
      moves.forEach(move => {
        try {
          chess.move(move);
        } catch (e) {
          console.error("Invalid move:", move);
        }
      });
      setPreviousFen(chess.fen());
    }
    updateBoard();
    setTurn(chess.turn());
    setPendingMove(null); // Clear any pending moves when game updates
  }, [game?.current_fen, game?.moves]);

  // Handle move errors by rolling back
  useEffect(() => {
    if (moveError && pendingMove && previousFen) {
      // Rollback the optimistic move
      try {
        chess.load(previousFen);
        updateBoard();
        setTurn(chess.turn());
        setPendingMove(null);
        setSelectedSquare(null);
        setPossibleMoves([]);
      } catch (e) {
        console.error("Failed to rollback move:", e);
      }
    }
  }, [moveError, pendingMove, previousFen]);

  const updateBoard = () => {
    const newBoard: string[][] = [];
    for (let row = 7; row >= 0; row--) {
      const boardRow: string[] = [];
      for (let col = 0; col < 8; col++) {
        const square = String.fromCharCode(97 + col) + (row + 1);
        const piece = chess.get(square as any);
        boardRow.push(piece ? `${piece.color}${piece.type}` : '');
      }
      newBoard.push(boardRow);
    }
    setBoard(newBoard);
  };

  const isPlayerTurn = () => {
    if (!game) return false;
    const isWhite = game.white === currentPlayer;
    return (isWhite && turn === 'w') || (!isWhite && turn === 'b');
  };

  const handleSquareClick = async (row: number, col: number) => {
    if (disabled || !isPlayerTurn() || pendingMove) return;

    const file = String.fromCharCode(97 + col);
    const rank = 8 - row;
    const square = `${file}${rank}`;

    if (selectedSquare) {
      const move = { from: selectedSquare, to: square };
      try {
        // Store the current position for potential rollback
        const currentFen = chess.fen();
        setPreviousFen(currentFen);
        
        // Apply the move with chess.js (only legal moves allowed)
        const result = chess.move(move);
        if (result) {
          // Generate FEN after the move
          const resulting_fen = chess.fen();
          
          setPendingMove(move);
          setSelectedSquare(null);
          setPossibleMoves([]);
          updateBoard();
          setTurn(chess.turn());
          
          // Try to execute the move on-chain
          const success = await onMove({
            ...move,
            resulting_fen
          });
          
          if (success) {
            setPendingMove(null);
          }
          // If not successful, the useEffect will handle rollback when moveError updates
        }
      } catch (e) {
        setSelectedSquare(square);
        updatePossibleMoves(square);
      }
    } else {
      setSelectedSquare(square);
      updatePossibleMoves(square);
    }
  };

  const updatePossibleMoves = (square: string) => {
    // Use chess.js to get legal moves (prevents putting king in check)
    const moves = chess.moves({ square: square as any, verbose: true });
    setPossibleMoves(moves.map(m => m.to));
  };

  const getPieceSymbol = (piece: string) => {
    const symbols: { [key: string]: string } = {
      'wp': '♙', 'wn': '♘', 'wb': '♗', 'wr': '♖', 'wq': '♕', 'wk': '♔',
      'bp': '♟', 'bn': '♞', 'bb': '♝', 'br': '♜', 'bq': '♛', 'bk': '♚'
    };
    return symbols[piece] || '';
  };

  const getSquareColor = (row: number, col: number) => {
    const file = String.fromCharCode(97 + col);
    const rank = 8 - row;
    const square = `${file}${rank}`;
    
    if (selectedSquare === square) return 'bg-yellow-400';
    if (possibleMoves.includes(square)) return 'bg-green-300';
    return (row + col) % 2 === 0 ? 'bg-amber-100' : 'bg-amber-700';
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="text-white text-center">
        {pendingMove ? (
          <div className="text-yellow-400">Processing move...</div>
        ) : (
          <div>
            {isPlayerTurn() ? "Your turn" : "Opponent's turn"} ({turn === 'w' ? 'White' : 'Black'} to move)
          </div>
        )}
      </div>
      
      {moveError && (
        <div className="bg-red-900/50 border border-red-600 text-red-400 p-3 rounded text-center">
          <div>Move failed: {moveError}</div>
          <button 
            onClick={onClearError}
            className="mt-2 px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded text-sm"
          >
            Dismiss
          </button>
        </div>
      )}
      <div className="grid grid-cols-8 gap-0 border-2 border-black w-fit mx-auto">
        {board.map((row, rowIndex) => (
          row.map((piece, colIndex) => (
            <div
              key={`${rowIndex}-${colIndex}`}
              className={`w-16 h-16 flex items-center justify-center text-5xl cursor-pointer ${getSquareColor(rowIndex, colIndex)}`}
              onClick={() => handleSquareClick(rowIndex, colIndex)}
            >
              {getPieceSymbol(piece)}
            </div>
          ))
        ))}
      </div>
    </div>
  );
}