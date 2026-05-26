import { createContext, useContext } from "react";

/**
 * Context die de norm-switch trigger doorgeeft van `AppShell` naar pages
 * die hem mogen aanroepen (momenteel alleen `WarmteverliesInstellingen`).
 *
 * `AppShell` host de `NormSwitchModal` open-state — zie `AppShell.tsx`.
 * De Card op de instellingen-pagina is de enige UI-trigger; eerdere
 * Backstage-entry is verwijderd omdat reken-instellingen daar niet thuis-
 * horen (zie sessie 2026-05-26).
 */
export interface NormSwitchContextValue {
  openNormSwitch: () => void;
}

export const NormSwitchContext = createContext<NormSwitchContextValue | null>(
  null,
);

/** Hook voor pages — gooit als de provider ontbreekt zodat misconfig
 * tijdens dev direct zichtbaar wordt. */
export function useNormSwitch(): NormSwitchContextValue {
  const ctx = useContext(NormSwitchContext);
  if (!ctx) {
    throw new Error(
      "useNormSwitch must be used within <NormSwitchContext.Provider>",
    );
  }
  return ctx;
}
