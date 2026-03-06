import type { ReactNode } from "react";

import { Sidebar } from "./Sidebar";
import { ToastContainer } from "../ui/Toast";
import { ConflictDialog } from "../ui/ConflictDialog";
import { useAutoSave } from "../../hooks/useAutoSave";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  useAutoSave();

  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="ml-sidebar flex-1">{children}</main>
      <ToastContainer />
      <ConflictDialog />
    </div>
  );
}
