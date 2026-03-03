import type { ReactNode } from "react";

import { Sidebar } from "./Sidebar";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="ml-sidebar flex-1">{children}</main>
    </div>
  );
}
