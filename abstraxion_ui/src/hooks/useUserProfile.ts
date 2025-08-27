import { useState, useEffect, useCallback } from "react";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import {
  useAbstraxionAccount,
  useAbstraxionSigningClient,
  useAbstraxionClient,
} from "@burnt-labs/abstraxion";
import type { ChessUser } from "../types/chess";

const USER_CONTRACT_ADDRESS = process.env.NEXT_PUBLIC_CONTRACT_ADDRESS || "";

interface UseUserProfileReturn {
  userProfile: ChessUser | null;
  loading: boolean;
  error: string | null;
  initializeUser: () => Promise<ExecuteResult | null>;
  updateUserProfile: (profile: ChessUser) => Promise<ExecuteResult | null>;
  fetchUserProfile: (address?: string) => Promise<ChessUser | null>;
  fetchAllUsers: () => Promise<string[]>;
  updateUserStats: (won: boolean, eloChange: number) => Promise<ExecuteResult | null>;
}

export function useUserProfile(): UseUserProfileReturn {
  const { data: account } = useAbstraxionAccount();
  const { client } = useAbstraxionSigningClient();
  const { client: queryClient } = useAbstraxionClient();
  
  const [userProfile, setUserProfile] = useState<ChessUser | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchUserProfile = useCallback(async (address?: string): Promise<ChessUser | null> => {
    const targetAddress = address || account?.bech32Address;
    if (!targetAddress || !queryClient) {
      setError("No address or query client available");
      return null;
    }
    if (!USER_CONTRACT_ADDRESS) {
      setError("User contract address not configured");
      return null;
    }

    console.log("Fetching user profile for:", targetAddress, "from contract:", USER_CONTRACT_ADDRESS);
    setLoading(true);
    setError(null);

    try {
      const response = await queryClient.queryContractSmart(USER_CONTRACT_ADDRESS, {
        get_value_by_user: { address: targetAddress }
      });

      if (response) {
        const profile = JSON.parse(response) as ChessUser;
        if (!address || address === account?.bech32Address) {
          setUserProfile(profile);
        }
        return profile;
      }
      return null;
    } catch (err) {
      console.error("Error fetching user profile:", err);
      setError(err instanceof Error ? err.message : "Failed to fetch profile");
      return null;
    } finally {
      setLoading(false);
    }
  }, [account?.bech32Address, queryClient]);

  const initializeUser = useCallback(async (): Promise<ExecuteResult | null> => {
    if (!client || !account) {
      setError("Client or account not available");
      return null;
    }

    setLoading(true);
    setError(null);

    try {
      const defaultProfile: ChessUser = {
        username: account.bech32Address.slice(0, 8),
        elo: 1200,
        games_played: 0,
        wins: 0,
        losses: 0,
        draws: 0,
        current_games: [],
        created_at: new Date().toISOString()
      };

      const msg = { update: { value: JSON.stringify(defaultProfile) } };
      const result = await client.execute(
        account.bech32Address,
        USER_CONTRACT_ADDRESS,
        msg,
        "auto"
      );

      // Wait for transaction to be confirmed
      if (result.transactionHash) {
        // Small delay to ensure state is updated
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Fetch the updated profile
        await fetchUserProfile();
      }

      return result;
    } catch (err) {
      console.error("Error initializing user:", err);
      setError(err instanceof Error ? err.message : "Failed to initialize user");
      return null;
    } finally {
      setLoading(false);
    }
  }, [client, account, fetchUserProfile]);

  const updateUserProfile = useCallback(async (profile: ChessUser): Promise<ExecuteResult | null> => {
    if (!client || !account) {
      setError("Client or account not available");
      return null;
    }

    setLoading(true);
    setError(null);

    try {
      const msg = { update: { value: JSON.stringify(profile) } };
      const result = await client.execute(
        account.bech32Address,
        USER_CONTRACT_ADDRESS,
        msg,
        "auto"
      );

      if (result.transactionHash) {
        setUserProfile(profile);
        // Small delay to ensure state is updated
        await new Promise(resolve => setTimeout(resolve, 2000));
      }

      return result;
    } catch (err) {
      console.error("Error updating profile:", err);
      setError(err instanceof Error ? err.message : "Failed to update profile");
      return null;
    } finally {
      setLoading(false);
    }
  }, [client, account]);

  const updateUserStats = useCallback(async (won: boolean, eloChange: number): Promise<ExecuteResult | null> => {
    if (!userProfile) {
      setError("No user profile loaded");
      return null;
    }

    const updatedProfile: ChessUser = {
      ...userProfile,
      games_played: userProfile.games_played + 1,
      wins: won ? userProfile.wins + 1 : userProfile.wins,
      losses: !won ? userProfile.losses + 1 : userProfile.losses,
      elo: Math.max(100, userProfile.elo + eloChange) // Minimum ELO of 100
    };

    return updateUserProfile(updatedProfile);
  }, [userProfile, updateUserProfile]);

  const fetchAllUsers = useCallback(async (): Promise<string[]> => {
    if (!queryClient) {
      setError("Query client not available");
      return [];
    }

    try {
      const response = await queryClient.queryContractSmart(USER_CONTRACT_ADDRESS, {
        get_users: {}
      });
      return response as string[];
    } catch (err) {
      console.error("Error fetching all users:", err);
      setError(err instanceof Error ? err.message : "Failed to fetch users");
      return [];
    }
  }, [queryClient]);

  // Auto-fetch profile when account is available
  useEffect(() => {
    if (account?.bech32Address && queryClient) {
      fetchUserProfile();
    }
  }, [account?.bech32Address, queryClient, fetchUserProfile]);

  return {
    userProfile,
    loading,
    error,
    initializeUser,
    updateUserProfile,
    fetchUserProfile,
    fetchAllUsers,
    updateUserStats,
  };
}