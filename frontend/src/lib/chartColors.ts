/**
 * Chart & diagram kleuren — centraal gedefinieerd via CSS custom properties.
 *
 * De daadwerkelijke hex-waarden staan in themes.css als --domain-* tokens.
 * Hierdoor kunnen ze in de toekomst per thema variëren.
 */

/** Kleuren voor de SummaryDonut (gebouwtotaal per verliestype). */
export const LOSS_TYPE_COLORS = {
  transmission: "var(--domain-chart-transmission)",
  ventilation: "var(--domain-chart-ventilation)",
  heatingUp: "var(--domain-chart-heating-up)",
  system: "var(--domain-chart-system)",
  neighbor: "var(--domain-chart-neighbor)",
} as const;

/** Kleuren voor de ConstructionLossChart (per constructiecategorie). */
export const CONSTRUCTION_CATEGORY_COLORS = {
  walls: "var(--domain-chart-walls)",
  glazing: "var(--domain-chart-glazing)",
  roofs: "var(--domain-chart-roofs)",
  floors: "var(--domain-chart-floors)",
  internalWalls: "var(--domain-chart-internal-walls)",
  other: "var(--domain-chart-other)",
} as const;

/** Kleuren voor de StackedBarChart (per vertrek). */
export const STACKED_BAR_COLORS = {
  transmission: "var(--domain-chart-transmission)",
  ventilation: "var(--domain-chart-ventilation)",
  infiltration: "var(--domain-chart-neighbor)",
  heatingUp: "var(--domain-chart-heating-up)",
  system: "var(--domain-chart-system)",
} as const;

/** SVG grid/axis kleuren. */
export const CHART_GRID_COLORS = {
  grid: "var(--domain-chart-grid)",
  gridStrong: "var(--domain-chart-grid-strong)",
} as const;
