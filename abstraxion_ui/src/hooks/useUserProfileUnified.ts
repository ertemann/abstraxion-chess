import { useState, useEffect, useCallback } from "react";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import {
  useAbstraxionAccount,
  useAbstraxionSigningClient,
  useAbstraxionClient,
} from "@burnt-labs/abstraxion";
import type { ChessUser } from "../types/chess";

const CHESS_CONTRACT_ADDRESS = process.env.NEXT_PUBLIC_CHESS_GAME_ADDRESS || "";

interface UseUserProfileReturn {
  userProfile: ChessUser | null;
  allUsers: ChessUser[];
  loading: boolean;
  error: string | null;
  initializeUser: (username?: string) => Promise<ExecuteResult | null>;
  refreshProfile: () => Promise<void>;
  refreshAllUsers: () => Promise<void>;
}

export function useUserProfileUnified(): UseUserProfileReturn {
  const { data: account } = useAbstraxionAccount();
  const { client } = useAbstraxionSigningClient();
  const { client: queryClient } = useAbstraxionClient();
  
  const [userProfile, setUserProfile] = useState<ChessUser | null>(null);
  const [allUsers, setAllUsers] = useState<ChessUser[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const initializeUser = useCallback(async (username?: string): Promise<ExecuteResult | null> => {
    if (!client || !account) {
      setError("Client or account not available");
      return null;
    }

    setLoading(true);
    setError(null);

    try {
      const msg = {
        initialize_user: {
          username: username || account.bech32Address || "Anonymous"
        }
      };

      const result = await client.execute(
        account.bech32Address,
        CHESS_CONTRACT_ADDRESS,
        msg,
        "auto"
      );

      if (result.transactionHash) {
        // Refresh profile after initialization
        await new Promise(resolve => setTimeout(resolve, 2000));
        await refreshProfile();
      }

      return result;
    } catch (err) {
      console.error("Error initializing user:", err);
      setError(err instanceof Error ? err.message : "Failed to initialize user");
      return null;
    } finally {
      setLoading(false);
    }
  }, [client, account]);

  const refreshProfile = useCallback(async () => {
    if (!queryClient || !account?.bech32Address) {
      return;
    }

    try {
      const response = await queryClient.queryContractSmart(CHESS_CONTRACT_ADDRESS, {
        get_user_profile: { address: account.bech32Address }
      });

      if (response.profile) {
        // Convert contract format to frontend format
        const profile: ChessUser = {
          username: response.profile.username,
          elo: response.profile.elo,
          games_played: response.profile.games_played,
          wins: response.profile.wins,
          draws: response.profile.draws,
          losses: response.profile.losses,
          current_games: response.profile.current_games,
          created_at: response.profile.created_at.toString(),
        };
        setUserProfile(profile);
      } else {
        setUserProfile(null);
      }
    } catch (err) {
      console.error("Error fetching user profile:", err);
      // Don't set error for missing profiles - it's expected for new users
    }
  }, [queryClient, account?.bech32Address]);

  const refreshAllUsers = useCallback(async () => {
    if (!queryClient) {
      return;
    }

    try {
      // Get all user addresses
      const usersResponse = await queryClient.queryContractSmart(CHESS_CONTRACT_ADDRESS, {
        get_all_users: {}
      });

      const users: ChessUser[] = [];
      
      // Fetch each user's profile
      for (const userAddress of usersResponse.users) {
        try {
          const profileResponse = await queryClient.queryContractSmart(CHESS_CONTRACT_ADDRESS, {
            get_user_profile: { address: userAddress }
          });

          if (profileResponse.profile) {
            const user: ChessUser = {
              username: profileResponse.profile.username,
              elo: profileResponse.profile.elo,
              games_played: profileResponse.profile.games_played,
              wins: profileResponse.profile.wins,
              draws: profileResponse.profile.draws,
              losses: profileResponse.profile.losses,
              current_games: profileResponse.profile.current_games,
              created_at: profileResponse.profile.created_at.toString(),
            };
            users.push(user);
          }
        } catch (err) {
          console.error(`Error fetching profile for ${userAddress}:`, err);
          // Continue with other users
        }
      }

      // Sort by ELO rating (highest first)
      users.sort((a, b) => b.elo - a.elo);
      setAllUsers(users);
    } catch (err) {
      console.error("Error fetching all users:", err);
      setError(err instanceof Error ? err.message : "Failed to fetch users");
    }
  }, [queryClient]);

  // Auto-refresh profile when account changes
  useEffect(() => {
    if (account?.bech32Address && queryClient) {
      refreshProfile();
    }
  }, [account?.bech32Address, queryClient, refreshProfile]);

  return {
    userProfile,
    allUsers,
    loading,
    error,
    initializeUser,
    refreshProfile,
    refreshAllUsers,
  };
}