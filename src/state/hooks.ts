import { useContext } from "react";

import type { AppCoreContextValue, AdvisorContextValue, AppStateContextValue, ChampSelectContextValue, LeagueAssetsContextValue } from "./types";

// These are created in AppStateProvider and imported here
import { createContext } from "react";

export const AppStateContext = createContext<AppStateContextValue | null>(null);
export const AppCoreContext = createContext<AppCoreContextValue | null>(null);
export const LeagueAssetsContext = createContext<LeagueAssetsContextValue | null>(null);
export const ChampSelectContext = createContext<ChampSelectContextValue | null>(null);
export const AdvisorContext = createContext<AdvisorContextValue | null>(null);

export function useAppState() {
  const context = useContext(AppStateContext);

  if (!context) {
    throw new Error("AppStateProvider is missing");
  }

  return context;
}

export function useAppCore() {
  const context = useContext(AppCoreContext);

  if (!context) {
    throw new Error("AppStateProvider is missing");
  }

  return context;
}

export function useLeagueAssets() {
  const context = useContext(LeagueAssetsContext);

  if (!context) {
    throw new Error("AppStateProvider is missing");
  }

  return context;
}

export function useChampSelect() {
  const context = useContext(ChampSelectContext);

  if (!context) {
    throw new Error("AppStateProvider is missing");
  }

  return context;
}

export function useAdvisor() {
  const context = useContext(AdvisorContext);

  if (!context) {
    throw new Error("AppStateProvider is missing");
  }

  return context;
}
