import { Component, type ErrorInfo, type ReactNode } from "react";

/**
 * App-root error boundary.
 *
 * Vangt alle render-errors binnen `<AppShell>` op. React biedt (nog) geen
 * hook-based equivalent, dus dit blijft een class component.
 *
 * De reset-knop flusht bekende localStorage sleutels (`isso51-*` en
 * `pref:*`) plus Zustand persist-stores en laadt de app opnieuw. Dit
 * herstelt in praktijk 99% van de corrupted-state scenarios (legacy
 * project-JSON, ongeldige materials override, theme key zonder value).
 *
 * Stack trace wordt alleen in DEV naar `console.error` gelogd (zie
 * FE-M7 lesson: productie-build mag geen diagnose-spam in Sentry-stream
 * genereren).
 */

interface AppErrorBoundaryProps {
  children: ReactNode;
}

interface AppErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  showDetails: boolean;
}

/**
 * Verwijder alle wv-project localStorage sleutels én de Tauri-fallback
 * pref-keys. Catch-all: bij onverwachte store-naam ruimt een volledige
 * `localStorage.clear()` alsnog alles op — edge-case tijdens dev.
 */
function flushPersistedState(): void {
  try {
    const keysToRemove: string[] = [];
    for (let i = 0; i < localStorage.length; i += 1) {
      const key = localStorage.key(i);
      if (!key) continue;
      if (key.startsWith("isso51-") || key.startsWith("pref:")) {
        keysToRemove.push(key);
      }
    }
    for (const key of keysToRemove) {
      localStorage.removeItem(key);
    }
  } catch {
    // Als iteratie faalt (bv. quota exceeded flow), volledige wipe.
    try {
      localStorage.clear();
    } catch {
      // Silent — niets dat we kunnen doen bij SecurityError.
    }
  }
}

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  constructor(props: AppErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, showDetails: false };
  }

  static getDerivedStateFromError(error: Error): Partial<AppErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    if (import.meta.env.DEV) {
      // eslint-disable-next-line no-console
      console.error("[AppErrorBoundary]", error, info.componentStack);
    }
  }

  handleReset = (): void => {
    flushPersistedState();
    window.location.reload();
  };

  handleToggleDetails = (): void => {
    this.setState((prev) => ({ showDetails: !prev.showDetails }));
  };

  render(): ReactNode {
    if (!this.state.hasError) {
      return this.props.children;
    }

    const { error, showDetails } = this.state;

    const containerStyle: React.CSSProperties = {
      minHeight: "100vh",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      padding: "2rem",
      background: "var(--theme-bg, #36363E)",
      color: "var(--theme-text, #FAFAF9)",
      fontFamily: "system-ui, -apple-system, sans-serif",
    };

    const cardStyle: React.CSSProperties = {
      maxWidth: "560px",
      width: "100%",
      padding: "2rem",
      background: "var(--theme-bg-lighter, #44444C)",
      border: "1px solid var(--theme-border, rgba(217, 119, 6, 0.25))",
      borderRadius: "8px",
      boxShadow: "var(--theme-dialog-shadow, 0 4px 16px rgba(0, 0, 0, 0.35))",
    };

    const titleStyle: React.CSSProperties = {
      margin: "0 0 0.5rem",
      fontSize: "1.5rem",
      fontWeight: 600,
    };

    const subTextStyle: React.CSSProperties = {
      margin: "0 0 1.5rem",
      color: "var(--theme-text-secondary, rgba(250, 250, 249, 0.6))",
      lineHeight: 1.5,
    };

    const buttonRowStyle: React.CSSProperties = {
      display: "flex",
      gap: "0.75rem",
      flexWrap: "wrap",
    };

    const primaryButtonStyle: React.CSSProperties = {
      padding: "0.6rem 1.2rem",
      background: "var(--theme-accent, #D97706)",
      color: "var(--theme-accent-text, #36363E)",
      border: "none",
      borderRadius: "4px",
      fontSize: "0.95rem",
      fontWeight: 600,
      cursor: "pointer",
    };

    const secondaryButtonStyle: React.CSSProperties = {
      padding: "0.6rem 1.2rem",
      background: "transparent",
      color: "var(--theme-text, #FAFAF9)",
      border: "1px solid var(--theme-border, rgba(217, 119, 6, 0.25))",
      borderRadius: "4px",
      fontSize: "0.95rem",
      cursor: "pointer",
    };

    const detailsStyle: React.CSSProperties = {
      marginTop: "1.25rem",
      padding: "0.75rem 1rem",
      background: "rgba(0, 0, 0, 0.25)",
      border: "1px solid var(--theme-border-subtle, rgba(217, 119, 6, 0.15))",
      borderRadius: "4px",
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
      fontSize: "0.8rem",
      color: "var(--theme-text-secondary, rgba(250, 250, 249, 0.6))",
      whiteSpace: "pre-wrap",
      overflowX: "auto",
      maxHeight: "240px",
    };

    return (
      <div style={containerStyle} role="alert">
        <div style={cardStyle}>
          <h1 style={titleStyle}>Er ging iets mis</h1>
          <p style={subTextStyle}>
            De applicatie is vastgelopen. Gegevens herstellen doorgaans via de
            knop hieronder.
          </p>
          <div style={buttonRowStyle}>
            <button type="button" onClick={this.handleReset} style={primaryButtonStyle}>
              Reset &amp; herlaad
            </button>
            <button
              type="button"
              onClick={this.handleToggleDetails}
              style={secondaryButtonStyle}
            >
              {showDetails ? "Verberg details" : "Details"}
            </button>
          </div>
          {showDetails && error ? (
            <pre style={detailsStyle}>
              {error.name}: {error.message}
              {error.stack ? `\n\n${error.stack}` : ""}
            </pre>
          ) : null}
        </div>
      </div>
    );
  }
}
