"use client";
import React, { useState, useEffect } from "react";
import {
  useAbstraxionAccount,
  useAbstraxionSigningClient,
  useAbstraxionClient,
} from "@burnt-labs/abstraxion";
import { Button } from "@burnt-labs/ui";
import "@burnt-labs/ui/dist/index.css";
import LoadingModal from "@/components/LoadingModal";
import ChessBoard from "@/components/ChessBoard";
import { useUserProfileUnified } from "@/hooks/useUserProfileUnified";
import { useChessGame } from "@/hooks/useChessGame";
import { useGlobalStats } from "@/hooks/useGlobalStats";
import type { ChessMove } from "@/types/chess";

export default function Page(): JSX.Element {
  // Abstraxion hooks
  const { data: account, login } = useAbstraxionAccount();
  const { client, logout } = useAbstraxionSigningClient();
  const { client: queryClient } = useAbstraxionClient();

  const {
    userProfile,
    allUsers,
    loading: userLoading,
    error: userError,
    initializeUser,
    refreshAllUsers,
  } = useUserProfileUnified();

  const {
    currentGame,
    userActiveGames,
    loading: gameLoading,
    error: gameError,
    moveError,
    timeStatus,
    createGame,
    makeMove,
    fetchGame,
    resignGame,
    selectGame,
    refreshUserGames,
    proposeDraw,
    respondToDraw,
    clearMoveError,
  } = useChessGame();

  const { stats, loading: statsLoading } = useGlobalStats();

  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [opponentAddress, setOpponentAddress] = useState("");
  const [showNewGameForm, setShowNewGameForm] = useState(false);
  const [moveInput, setMoveInput] = useState("");

  const handleLogin = async () => {
    if (!account?.bech32Address) {
      setIsLoggingIn(true);
      try {
        await login();
      } catch (error) {
        console.error('Login failed:', error);
      } finally {
        setIsLoggingIn(false);
      }
    }
  };

  const handleInitializeUser = async () => {
    if (!userProfile && account?.bech32Address) {
      await initializeUser();
    }
  };

  const handleCreateGame = async () => {
    if (opponentAddress && opponentAddress !== account?.bech32Address) {
      const result = await createGame(opponentAddress);
      if (result) {
        setShowNewGameForm(false);
        setOpponentAddress("");
      }
    }
  };

  const handleMove = async (move: ChessMove): Promise<boolean> => {
    if (currentGame) {
      return await makeMove(currentGame.id, move);
    }
    return false;
  };

  const handleMoveInput = async () => {
    if (moveInput.length >= 4 && currentGame) {
      const from = moveInput.substring(0, 2);
      const to = moveInput.substring(2, 4);
      const promotion = moveInput.length > 4 ? moveInput.substring(4) : undefined;
      
      const success = await makeMove(currentGame.id, { from, to, promotion });
      if (success) {
        setMoveInput("");
      }
    }
  };

  // Copy address to clipboard
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      // Could add a toast notification here if needed
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  // Auto-load leaderboard when component mounts
  React.useEffect(() => {
    if (queryClient) {
      refreshAllUsers();
    }
  }, [queryClient, refreshAllUsers]);

  return (
    <main className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-700 p-4">
        <h1 className="text-3xl font-bold text-center">Xion Chess</h1>
        <div className="flex justify-center gap-8 mt-2 text-sm text-gray-400">
          <span>Total Games: {stats.totalGames || 0}</span>
          <span>Active Players: {stats.activePlayers || 0}</span>
          <span>Total Moves: {stats.totalMoves || 0}</span>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex max-w-7xl mx-auto p-4 gap-6">
        {/* Left Sidebar */}
        <aside className="w-80 flex flex-col gap-4">
          {/* Connection Button */}
          {account?.bech32Address ? (
            <div className="bg-gray-800 p-3 rounded-lg">
              <div className="text-xs text-gray-400 mb-1">Your Address</div>
              <button
                onClick={() => copyToClipboard(account.bech32Address)}
                className="w-full text-left font-mono text-sm bg-gray-700 hover:bg-gray-600 p-2 rounded transition-colors"
                title="Click to copy full address"
              >
                {account.bech32Address.slice(0, 12)}...{account.bech32Address.slice(-8)}
              </button>
            </div>
          ) : (
            <Button 
              fullWidth 
              onClick={handleLogin}
              structure="base"
            >
              CONNECT
            </Button>
          )}

          {/* Initialize Profile */}
          {account?.bech32Address && !userProfile && (
            <Button 
              fullWidth 
              onClick={handleInitializeUser}
              structure="base"
              disabled={userLoading}
            >
              Initialize Profile
            </Button>
          )}

          {/* User Profile */}
          {userProfile && (
            <div className="bg-gray-800 p-4 rounded-lg">
              <h3 className="font-bold mb-2">Your Profile</h3>
              <div className="text-sm space-y-1">
                <div className="truncate">User: {userProfile.username.slice(0, 15)}...</div>
                <div>ELO: <span className="text-yellow-400 font-bold">{userProfile.elo}</span></div>
                <div>Games: {userProfile.games_played}</div>
                <div>
                  <span className="text-green-400">{userProfile.wins}W</span> - 
                  <span className="text-gray-400">{userProfile.draws}D</span> - 
                  <span className="text-red-400">{userProfile.losses}L</span>
                </div>
              </div>
            </div>
          )}

          {/* New Game */}
          {userProfile && (
            <div className="bg-gray-800 p-4 rounded-lg">
              <h3 className="font-bold mb-2">New Game</h3>
              <input
                type="text"
                value={opponentAddress}
                onChange={(e) => setOpponentAddress(e.target.value)}
                placeholder="Opponent address..."
                className="w-full p-2 rounded bg-gray-700 text-sm mb-2"
              />
              <Button
                fullWidth
                onClick={handleCreateGame}
                structure="base"
                disabled={!opponentAddress || gameLoading}
              >
                Create Game
              </Button>
            </div>
          )}

          {/* Active Games */}
          {userActiveGames.length > 0 && (
            <div className="bg-gray-800 p-4 rounded-lg">
              <h3 className="font-bold mb-2">Active Games ({userActiveGames.length})</h3>
              <div className="space-y-2 max-h-40 overflow-y-auto">
                {userActiveGames.map(({ id, game }) => (
                  <div
                    key={id}
                    className={`p-2 rounded cursor-pointer text-sm ${
                      currentGame?.id === id 
                        ? 'bg-blue-600' 
                        : 'bg-gray-700 hover:bg-gray-600'
                    }`}
                    onClick={() => selectGame(id)}
                  >
                    <div className="truncate">vs {
                      game.white === account?.bech32Address 
                        ? game.black.slice(0, 10) + '...'
                        : game.white.slice(0, 10) + '...'
                    }</div>
                    <div className="text-xs text-gray-400">Turn: {game.current_turn}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Leaderboard */}
          <div className="bg-gray-800 p-4 rounded-lg">
            <div className="flex justify-between items-center mb-2">
              <h3 className="font-bold">Leaderboard</h3>
              <button
                onClick={refreshAllUsers}
                className="text-xs px-2 py-1 bg-gray-700 rounded hover:bg-gray-600"
              >
                🔄
              </button>
            </div>
            <div className="space-y-1 max-h-40 overflow-y-auto">
              {allUsers.slice(0, 10).map((user, index) => (
                <div key={user.username} className="text-sm py-1">
                  <div className="flex justify-between items-center">
                    <span className="flex items-center gap-2">
                      <span className={`font-bold ${
                        index === 0 ? 'text-yellow-400' : 'text-gray-400'
                      }`}>
                        {index + 1}
                      </span>
                      <span className="truncate max-w-[120px]">{user.username}</span>
                    </span>
                    <span className="text-yellow-400 font-bold">{user.elo}</span>
                  </div>
                  <button
                    onClick={() => copyToClipboard(user.address)}
                    className="w-full text-left text-xs font-mono text-gray-400 hover:text-gray-200 bg-gray-700 hover:bg-gray-600 px-2 py-1 rounded mt-1 transition-colors"
                    title="Click to copy address"
                  >
                    {user.address.slice(0, 10)}...{user.address.slice(-6)}
                  </button>
                </div>
              ))}
            </div>
          </div>

          {/* Logout */}
          {logout && (
            <Button fullWidth onClick={logout} structure="base">
              Logout
            </Button>
          )}
        </aside>

        {/* Main Game Area */}
        <div className="flex-1 flex flex-col gap-4">
          {currentGame ? (
            <>
              {/* Game Header */}
              <div className="flex justify-between items-center bg-gray-800 p-3 rounded-lg">
                <h2 className="font-bold">Game: {currentGame.id.slice(0, 12)}...</h2>
                <button
                  onClick={() => fetchGame(currentGame.id)}
                  className="px-3 py-1 bg-gray-700 rounded hover:bg-gray-600"
                  disabled={gameLoading}
                >
                  🔄 Refresh
                </button>
              </div>

              {currentGame.status === 'active' ? (
                <>
                  {/* Compact Time Display */}
                  <div className="bg-gray-800 p-2 rounded-lg flex justify-between items-center">
                    <div className="flex items-center gap-2">
                      <span className="text-sm">⚫ Black</span>
                      <span className={`font-mono text-lg ${
                        currentGame.current_turn === 'black' ? 'text-yellow-400' : ''
                      }`}>
                        {timeStatus ? formatTime(timeStatus.black_time_remaining) : '--:--'}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`font-mono text-lg ${
                        currentGame.current_turn === 'white' ? 'text-yellow-400' : ''
                      }`}>
                        {timeStatus ? formatTime(timeStatus.white_time_remaining) : '--:--'}
                      </span>
                      <span className="text-sm">⚪ White</span>
                    </div>
                  </div>

                  {/* Chess Board */}
                  <ChessBoard
                    game={currentGame}
                    currentPlayer={account?.bech32Address || ''}
                    onMove={handleMove}
                    disabled={gameLoading}
                    moveError={moveError}
                    onClearError={clearMoveError}
                  />

                  {/* Game Controls */}
                  <div className="bg-gray-800 p-4 rounded-lg space-y-3">
                    {/* Move Input */}
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={moveInput}
                        onChange={(e) => setMoveInput(e.target.value)}
                        placeholder="Enter move (e.g., e2e4)"
                        className="flex-1 p-2 rounded bg-gray-700"
                      />
                      <Button
                        onClick={handleMoveInput}
                        structure="base"
                        disabled={!moveInput || gameLoading}
                      >
                        Submit
                      </Button>
                    </div>

                    {/* Status */}
                    <div className="flex justify-between text-sm">
                      <span>Turn: <span className="text-yellow-400">{currentGame.current_turn}</span></span>
                      <span>Moves: {currentGame.moves.split(',').filter(Boolean).length}</span>
                    </div>

                    {/* Draw/Resign Buttons */}
                    {currentGame.draw_proposed_by && currentGame.draw_proposed_by !== account?.bech32Address ? (
                      <div className="bg-yellow-900/30 border border-yellow-600 p-3 rounded">
                        <div className="text-yellow-400 text-center mb-2">Opponent proposed draw!</div>
                        <div className="flex gap-2">
                          <Button
                            onClick={() => respondToDraw(currentGame.id, true)}
                            structure="base"
                            className="flex-1"
                          >
                            Accept
                          </Button>
                          <Button
                            onClick={() => respondToDraw(currentGame.id, false)}
                            structure="base"
                            className="flex-1"
                          >
                            Decline
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <div className="flex gap-2">
                        <Button
                          onClick={() => proposeDraw(currentGame.id)}
                          structure="base"
                          disabled={gameLoading || currentGame.draw_proposed_by === account?.bech32Address}
                          className="flex-1"
                        >
                          {currentGame.draw_proposed_by === account?.bech32Address ? 'Draw Pending' : 'Propose Draw'}
                        </Button>
                        <Button
                          onClick={() => resignGame(currentGame.id)}
                          structure="base"
                          disabled={gameLoading}
                          className="flex-1"
                        >
                          Resign
                        </Button>
                      </div>
                    )}
                  </div>
                </>
              ) : (
                /* Game Over Display */
                <div className="bg-gray-800 p-8 rounded-lg text-center">
                  <h3 className="text-2xl font-bold mb-4">Game Over!</h3>
                  <div className="text-lg">
                    Result: <span className={`font-bold ${
                      currentGame.status === 'draw' ? 'text-yellow-400' :
                      (currentGame.status === 'white_won' && currentGame.white === account?.bech32Address) ||
                      (currentGame.status === 'black_won' && currentGame.black === account?.bech32Address) 
                        ? 'text-green-400' : 'text-red-400'
                    }`}>
                      {currentGame.status.replace('_', ' ').toUpperCase()}
                    </span>
                  </div>
                </div>
              )}
            </>
          ) : (
            <div className="flex items-center justify-center h-96 bg-gray-800 rounded-lg">
              <p className="text-gray-400">
                {account?.bech32Address 
                  ? (userProfile ? "Select or create a game to start playing" : "Initialize your profile first")
                  : "Connect your wallet to start playing"}
              </p>
            </div>
          )}
        </div>
      </div>

      <LoadingModal isOpen={isLoggingIn} message="Connecting to your wallet..." />
    </main>
  );
}

// Helper function for time formatting
function formatTime(blocks: number): string {
  if (blocks <= 0) return "0:00";
  const totalSeconds = blocks;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  }
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}