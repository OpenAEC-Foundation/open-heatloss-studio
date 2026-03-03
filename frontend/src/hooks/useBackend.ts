import { useMemo } from "react";

import { createBackend, type Backend } from "../lib/backend";

/** Returns a stable Backend instance for the current environment. */
export function useBackend(): Backend {
  return useMemo(() => createBackend(), []);
}
