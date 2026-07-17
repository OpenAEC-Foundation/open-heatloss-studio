import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./i18n/config";
import "./index.css";

declare global {
  interface Window {
    __splashStart?: number;
  }
}

/** Minimale zichtbaarheidsduur van de splash, ook als de app sneller klaar is. */
const MIN_SPLASH_MS = 900;
/** Fallback: als transitionend nooit vuurt, toch opruimen. */
const SPLASH_FADE_MS = 350;

function dismissSplash(): void {
  const params = new URLSearchParams(window.location.search);
  if (params.get("splash") === "hold") {
    return; // design/debug: splash blijft zichtbaar
  }

  const splash = document.getElementById("splash");
  if (!splash) {
    return;
  }

  const start = window.__splashStart ?? performance.now();
  const elapsed = performance.now() - start;
  const remaining = Math.max(0, MIN_SPLASH_MS - elapsed);

  window.setTimeout(() => {
    splash.classList.add("splash-out");
    let removed = false;
    const remove = () => {
      if (removed) return;
      removed = true;
      splash.remove();
    };
    splash.addEventListener("transitionend", remove, { once: true });
    window.setTimeout(remove, SPLASH_FADE_MS + 200);
  }, remaining);
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

dismissSplash();
