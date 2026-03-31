/**
 * SVG-diagram voor de Glaser-methode (dampspanningsverloop).
 *
 * Toont verzadigingsdampdruk (pSat) en werkelijke dampdruk (pActual)
 * door de constructie-opbouw heen. Materiaallagen worden getoond met
 * categorie-specifieke kleuren en NEN 47-conforme arceringen.
 *
 * Hatch patterns komen uit lib/hatchPatterns.ts (gedeeld met glaserSvg.ts).
 */

import { useMemo } from "react";

import type { GlaserResult, LayerStudInfo } from "../../lib/glaserCalculation";
import {
  getAllPatternDefs,
  resolvePatternId,
} from "../../lib/hatchPatterns";
import {
  MATERIAL_CATEGORY_VISUALS,
  type MaterialCategory,
} from "../../lib/materialsDatabase";

interface GlaserDiagramProps {
  result: GlaserResult;
  thetaI: number;
  thetaE: number;
}

// ---------- Layout constanten ----------

const WIDTH = 640;
const HEIGHT = 420;
const MARGIN = { top: 20, right: 55, bottom: 60, left: 58 };
const PLOT_W = WIDTH - MARGIN.left - MARGIN.right;
const PLOT_H = HEIGHT - MARGIN.top - MARGIN.bottom;

/** Breedte van de binnen/buiten-lucht zones (px). */
const AIR_ZONE_W = 30;
/** Kleur van de binnen/buiten-lucht zones (lichtblauw, zelfde als oude spouw). */
const AIR_ZONE_COLOR = "#bfdbfe";

/** Aantal schematische studs per laag. */
const STUD_COUNT = 1;

// ---------- Helpers ----------

function niceMax(value: number): number {
  if (value <= 0) return 100;
  const magnitude = Math.pow(10, Math.floor(Math.log10(value)));
  const normalized = value / magnitude;
  if (normalized <= 1) return magnitude;
  if (normalized <= 2) return 2 * magnitude;
  if (normalized <= 5) return 5 * magnitude;
  return 10 * magnitude;
}

function generateTicks(max: number, count: number): number[] {
  const step = max / count;
  const ticks: number[] = [];
  for (let i = 0; i <= count; i++) {
    ticks.push(Math.round(step * i));
  }
  return ticks;
}

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen - 1) + "\u2026";
}

function categoryColor(cat: MaterialCategory): string {
  return MATERIAL_CATEGORY_VISUALS[cat]?.color ?? "#e5e7eb";
}

/** Isolatie-categorieën die een directe zigzag-arcering krijgen (geen tiled pattern). */
const INSULATION_CATEGORIES: ReadonlySet<MaterialCategory> = new Set([
  "isolatie_mineraal",
  "isolatie_kunststof",
  "isolatie_natuurlijk",
]);

/** Hoek van de zigzag-lijnen ten opzichte van horizontaal (60°). */
const ZIGZAG_ANGLE_DEG = 60;
const ZIGZAG_TAN = Math.tan((ZIGZAG_ANGLE_DEG * Math.PI) / 180); // ~1.732

/** Zigzag styling per isolatietype. */
const ZIGZAG_STYLE: Record<string, { strokeWidth: number; offset?: number }> = {
  isolatie_mineraal: { strokeWidth: 0.8 },
  isolatie_kunststof: { strokeWidth: 0.8, offset: 1.5 },
  isolatie_natuurlijk: { strokeWidth: 0.8 },
};

/**
 * Genereer een SVG path (M/L) voor een doorlopend zigzag-patroon binnen een band.
 *
 * De zigzag loopt VERTICAAL langs de hoogte van de band (de langste richting).
 * Elke lijn gaat van de linkerzijde naar de rechterzijde (of terug) onder ~60°
 * t.o.v. de verticale as. Zo ontstaat het kenmerkende NEN 47 isolatie-patroon
 * waarbij de lijnen de volledige breedte van de laag overspannen.
 *
 * @param bandX   - linker x-coördinaat van de band
 * @param bandY   - boven y-coördinaat van de band
 * @param bandW   - breedte van de band
 * @param bandH   - hoogte van de band
 * @param periodScale - vermenigvuldigingsfactor voor de zigzag-periode (1.0 = standaard)
 * @returns SVG path string
 */
function generateZigzagPath(
  bandX: number,
  bandY: number,
  bandW: number,
  bandH: number,
  periodScale = 1.0,
): string {
  if (bandW < 1 || bandH < 1) return "";

  // Halve zigzag-hoogte: verticale afstand per zijde-oversteek (links→rechts of rechts→links)
  // Bij 60° t.o.v. verticaal: tan(60°) = bandW / halfPeriod → halfPeriod = bandW / tan(60°)
  const halfPeriod = (bandW / ZIGZAG_TAN) * periodScale;
  if (halfPeriod < 0.5) return "";

  const left = bandX;
  const right = bandX + bandW;

  const points: string[] = [];
  let y = bandY;
  let goingRight = true;

  // Start linksboven
  points.push(`M${left.toFixed(1)},${y.toFixed(1)}`);

  while (y < bandY + bandH + halfPeriod) {
    y += halfPeriod;
    const x = goingRight ? right : left;
    points.push(`L${x.toFixed(1)},${y.toFixed(1)}`);
    goingRight = !goingRight;
  }

  return points.join(" ");
}

// ---------- NEN 47 Hatch Pattern Defs (from shared module) ----------

function HatchPatterns() {
  const defs = getAllPatternDefs();
  // Render patterns via dangerouslySetInnerHTML for performance —
  // the pattern content is static SVG from our own hatchPatterns.ts.
  const svgContent = defs
    .map((p) => {
      const attrs = p.attrs ? ` ${p.attrs}` : "";
      return `<pattern id="${p.id}" width="${p.width}" height="${p.height}" patternUnits="userSpaceOnUse"${attrs}>${p.content}</pattern>`;
    })
    .join("");

  return <defs dangerouslySetInnerHTML={{ __html: svgContent }} />;
}

// ---------- Stud band computation ----------

interface StudBand {
  /** Y-positie van de stijl-band in SVG coords. */
  y: number;
  /** Hoogte van de stijl-band in SVG coords. */
  h: number;
}

/**
 * Bereken de posities van stijl-banden binnen een laagband.
 * Studs worden als HORIZONTALE banden getekend (over de volle breedte van de laag),
 * evenredig verdeeld over de hoogte. Dit omdat de laagband zelf een verticale strip
 * is en stijlen in werkelijkheid verticaal lopen (= horizontaal in het diagram).
 */
function computeStudBands(
  bandH: number,
  stud: LayerStudInfo,
): StudBand[] {
  if (bandH < 20) return []; // Te laag om studs te tonen

  // Stijlen prominent tonen: 2x proportionele hoogte voor leesbaarheid
  const fraction = stud.width / stud.spacing; // bijv. 38/600 = 0.063
  const count = STUD_COUNT;
  const studPixelH = Math.max(bandH * fraction * 2, 8);

  // Verdeel evenredig over de hoogte
  const totalStudH = count * studPixelH;
  const totalGapH = bandH - totalStudH;
  const gap = totalGapH / (count + 1);

  const bands: StudBand[] = [];
  for (let i = 0; i < count; i++) {
    bands.push({
      y: MARGIN.top + gap * (i + 1) + studPixelH * i,
      h: studPixelH,
    });
  }
  return bands;
}

// ---------- Layer band type ----------

interface LayerBand {
  x: number;
  w: number;
  name: string;
  color: string;
  category?: MaterialCategory;
  patternId?: string;
  stud?: LayerStudInfo;
  studPatternId?: string;
  studColor?: string;
}

// ---------- Component ----------

export function GlaserDiagram({ result, thetaI, thetaE }: GlaserDiagramProps) {
  const {
    curvePoints,
    interfacePoints,
    layerThicknesses,
    layerNames,
    layerCategories,
    layerHatchPatterns,
    layerStuds,
    totalThickness,
  } = result;

  const hasLayers = curvePoints.length >= 2 && totalThickness > 0;

  // Schalen berekenen
  const { yTicks, toX, toY, toYTemp, tempMin, tempMax } = useMemo(() => {
    if (!hasLayers) {
      return {
        yTicks: [0, 500, 1000, 1500, 2000, 2500],
        toX: () => MARGIN.left,
        toY: () => MARGIN.top + PLOT_H,
        toYTemp: () => MARGIN.top + PLOT_H,
        tempMin: 0,
        tempMax: 20,
      };
    }

    const allP = [
      ...curvePoints.map((p) => p.pSat),
      ...interfacePoints.map((p) => p.pActual),
    ];
    const rawMax = Math.max(...allP, 100);
    const nMax = niceMax(rawMax * 1.1);
    const ticks = generateTicks(nMax, 5);

    // Temperatuurschaal: van buitentemp tot binnentemp met marge
    const temps = curvePoints.map((p) => p.temperature);
    const tMin = Math.floor(Math.min(...temps) - 2);
    const tMax = Math.ceil(Math.max(...temps) + 2);

    return {
      yTicks: ticks,
      toX: (xMm: number) => MARGIN.left + AIR_ZONE_W + (xMm / totalThickness) * (PLOT_W - 2 * AIR_ZONE_W),
      toY: (pPa: number) => MARGIN.top + PLOT_H - (pPa / nMax) * PLOT_H,
      toYTemp: (t: number) => MARGIN.top + PLOT_H - ((t - tMin) / (tMax - tMin)) * PLOT_H,
      tempMin: tMin,
      tempMax: tMax,
    };
  }, [curvePoints, interfacePoints, totalThickness, hasLayers]);

  // pSat-curve pad
  const pSatPath = useMemo(() => {
    if (!hasLayers) return "";
    return curvePoints
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pSat).toFixed(1)}`,
      )
      .join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // pActual-lijn pad
  const pActualPath = useMemo(() => {
    if (!hasLayers) return "";
    return interfacePoints
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pActual).toFixed(1)}`,
      )
      .join(" ");
  }, [interfacePoints, hasLayers, toX, toY]);

  // Temperatuurlijn pad (inclusief Ri/Re verloop in luchtzones)
  const tempPath = useMemo(() => {
    if (!hasLayers) return "";
    // Start bij binnenluchttemperatuur (linkerrand binnenlucht zone)
    const parts: string[] = [];
    parts.push(`M${MARGIN.left.toFixed(1)},${toYTemp(thetaI).toFixed(1)}`);
    // Ri-verloop: van thetaI naar theta_si (eerste curvePoint)
    for (const p of curvePoints) {
      parts.push(`L${toX(p.x).toFixed(1)},${toYTemp(p.temperature).toFixed(1)}`);
    }
    // Re-verloop: van theta_se (laatste curvePoint) naar thetaE (rechterrand)
    parts.push(`L${(MARGIN.left + PLOT_W).toFixed(1)},${toYTemp(thetaE).toFixed(1)}`);
    return parts.join(" ");
  }, [curvePoints, hasLayers, toX, toYTemp, thetaI, thetaE]);

  // Condensatiezone: gebied waar pActual > pSat
  const condensationPath = useMemo(() => {
    if (!hasLayers) return "";

    const zones: string[] = [];
    let inZone = false;
    let zoneStart = "";

    for (let i = 0; i < curvePoints.length; i++) {
      const p = curvePoints[i]!;
      const x = toX(p.x).toFixed(1);
      const yActual = toY(p.pActual).toFixed(1);
      const ySat = toY(p.pSat).toFixed(1);

      if (p.pActual > p.pSat + 0.5) {
        if (!inZone) {
          zoneStart = `M${x},${ySat} L${x},${yActual}`;
          inZone = true;
        } else {
          zoneStart += ` L${x},${yActual}`;
        }
      } else if (inZone) {
        const backPath = curvePoints
          .slice(0, i)
          .reverse()
          .filter((bp) => bp.pActual > bp.pSat + 0.5)
          .map(
            (bp) =>
              `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`,
          )
          .join(" ");
        zones.push(`${zoneStart} ${backPath} Z`);
        inZone = false;
      }
    }

    if (inZone) {
      const lastInZone = curvePoints.filter(
        (p) => p.pActual > p.pSat + 0.5,
      );
      const backPath = [...lastInZone]
        .reverse()
        .map(
          (bp) =>
            `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`,
        )
        .join(" ");
      zones.push(`${zoneStart} ${backPath} Z`);
    }

    return zones.join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // Laag-banden met categorie-kleuren en patterns
  const layerBands = useMemo(() => {
    if (!hasLayers) return [];
    const bands: LayerBand[] = [];
    let xCum = 0;
    for (let i = 0; i < layerThicknesses.length; i++) {
      const d = layerThicknesses[i]!;
      const cat = layerCategories[i] as MaterialCategory | undefined;
      const hatchOverride = layerHatchPatterns?.[i];
      const stud = layerStuds?.[i];

      // Isolatie-categorieën krijgen directe zigzag-lijnen, geen tiled pattern
      const isInsulation = cat ? INSULATION_CATEGORIES.has(cat) : false;
      const patternId =
        cat && !isInsulation ? resolvePatternId(cat, hatchOverride) : undefined;

      // Stud pattern en kleur
      let studPatternId: string | undefined;
      let studColor: string | undefined;
      if (stud) {
        studPatternId = resolvePatternId(
          stud.studCategory,
          stud.studHatchPattern,
        );
        studColor = categoryColor(stud.studCategory);
      }

      bands.push({
        x: toX(xCum),
        w: (d / totalThickness) * (PLOT_W - 2 * AIR_ZONE_W),
        name: layerNames[i] ?? "",
        color: cat ? categoryColor(cat) : "#e5e7eb",
        category: cat,
        patternId,
        stud,
        studPatternId,
        studColor,
      });
      xCum += d;
    }
    return bands;
  }, [
    layerThicknesses,
    layerNames,
    layerCategories,
    layerHatchPatterns,
    layerStuds,
    totalThickness,
    hasLayers,
    toX,
  ]);

  if (!hasLayers) {
    return (
      <div className="flex h-48 items-center justify-center rounded-lg border border-dashed border-[var(--oaec-border)] text-sm text-on-surface-muted">
        Voeg lagen toe om het dampspanningsdiagram te zien.
      </div>
    );
  }

  return (
    <svg
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      className="w-full"
      style={{ maxHeight: 460 }}
    >
      <HatchPatterns />

      {/* Temperatuur gradient: rood (warm/binnen) → blauw (koud/buiten) */}
      <defs>
        <linearGradient id="temp-gradient" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor="#ef4444" />
          <stop offset="50%" stopColor="#a855f7" />
          <stop offset="100%" stopColor="#3b82f6" />
        </linearGradient>
      </defs>

      {/* Achtergrond */}
      <rect
        x={MARGIN.left}
        y={MARGIN.top}
        width={PLOT_W}
        height={PLOT_H}
        fill="white"
        stroke="#e7e5e4"
        strokeWidth={1}
      />

      {/* Binnenlucht zone (links) */}
      <rect
        x={MARGIN.left}
        y={MARGIN.top}
        width={AIR_ZONE_W}
        height={PLOT_H}
        fill={AIR_ZONE_COLOR}
        fillOpacity={0.45}
      />
      {/* Buitenlucht zone (rechts) */}
      <rect
        x={MARGIN.left + PLOT_W - AIR_ZONE_W}
        y={MARGIN.top}
        width={AIR_ZONE_W}
        height={PLOT_H}
        fill={AIR_ZONE_COLOR}
        fillOpacity={0.45}
      />

      {/* Laag-banden met kleur, arcering en studs */}
      {layerBands.map((band, i) => {
        const studBands = band.stud
          ? computeStudBands(PLOT_H, band.stud)
          : [];

        return (
          <g key={i}>
            {/* Witte ondergrond per laag (voorkomt doorbloeden) */}
            <rect
              x={band.x}
              y={MARGIN.top}
              width={Math.max(band.w, 1)}
              height={PLOT_H}
              fill="white"
            />
            {/* Kleurvulling voor hele laag */}
            <rect
              x={band.x}
              y={MARGIN.top}
              width={Math.max(band.w, 1)}
              height={PLOT_H}
              fill={band.color}
              fillOpacity={0.75}
            />
            {/* Arcering overlay voor hele laag (tiled pattern, niet voor isolatie) */}
            {band.patternId && (
              <rect
                x={band.x}
                y={MARGIN.top}
                width={Math.max(band.w, 1)}
                height={PLOT_H}
                fill={`url(#${band.patternId})`}
              />
            )}

            {/* Directe zigzag-arcering voor isolatie-lagen (NEN 47) */}
            {band.category && INSULATION_CATEGORIES.has(band.category) && band.w > 2 && (() => {
              const clipId = `insulation-clip-${i}`;
              const style = ZIGZAG_STYLE[band.category] ?? { strokeWidth: 0.8 };
              const periodScale = band.category === "isolatie_natuurlijk" ? 1.3 : 1.0;
              const extraH = band.w / ZIGZAG_TAN;
              const mainPath = generateZigzagPath(
                band.x,
                MARGIN.top - extraH,
                band.w,
                PLOT_H + 2 * extraH,
                periodScale,
              );
              return (
                <g>
                  <clipPath id={clipId}>
                    <rect x={band.x} y={MARGIN.top} width={band.w} height={PLOT_H} />
                  </clipPath>
                  <g clipPath={`url(#${clipId})`}>
                    <path
                      d={mainPath}
                      fill="none"
                      stroke="#555"
                      strokeWidth={style.strokeWidth}
                      opacity={0.6}
                      strokeLinejoin="round"
                    />
                    {/* Kunststof isolatie: tweede zigzag-lijn, licht verschoven */}
                    {style.offset && (
                      <path
                        d={generateZigzagPath(
                          band.x,
                          MARGIN.top - extraH + (style.offset ?? 0),
                          band.w,
                          PLOT_H + 2 * extraH,
                          periodScale,
                        )}
                        fill="none"
                        stroke="#555"
                        strokeWidth={style.strokeWidth}
                        opacity={0.6}
                        strokeLinejoin="round"
                      />
                    )}
                  </g>
                </g>
              );
            })()}

            {/* Studs: horizontale banden met hout-patroon */}
            {studBands.map((sb, si) => (
              <g key={`stud-${si}`}>
                {/* Stijl-kleur */}
                <rect
                  x={band.x}
                  y={sb.y}
                  width={band.w}
                  height={sb.h}
                  fill={band.studColor ?? "#c68642"}
                  fillOpacity={0.65}
                />
                {/* Stijl-arcering */}
                {band.studPatternId && (
                  <rect
                    x={band.x}
                    y={sb.y}
                    width={band.w}
                    height={sb.h}
                    fill={`url(#${band.studPatternId})`}
                  />
                )}
                {/* Scheidingslijnen stijl */}
                <line
                  x1={band.x}
                  y1={sb.y}
                  x2={band.x + band.w}
                  y2={sb.y}
                  stroke="#78716c"
                  strokeWidth={0.3}
                  strokeOpacity={0.5}
                />
                <line
                  x1={band.x}
                  y1={sb.y + sb.h}
                  x2={band.x + band.w}
                  y2={sb.y + sb.h}
                  stroke="#78716c"
                  strokeWidth={0.3}
                  strokeOpacity={0.5}
                />
              </g>
            ))}

            {/* Laag-scheidingslijn */}
            {i > 0 && (
              <line
                x1={band.x}
                y1={MARGIN.top}
                x2={band.x}
                y2={MARGIN.top + PLOT_H}
                stroke="#78716c"
                strokeWidth={0.5}
              />
            )}
            {/* Laagnaam */}
            {band.w > 14 && (
              <text
                x={band.x + band.w / 2}
                y={MARGIN.top + PLOT_H + 14}
                textAnchor="middle"
                fontSize={9}
                fill="#57534e"
                fontWeight={500}
                className="select-none"
              >
                {truncate(band.name, Math.max(4, Math.floor(band.w / 5.5)))}
              </text>
            )}
          </g>
        );
      })}

      {/* Y-as gridlijnen en labels */}
      {yTicks.map((tick) => {
        const y = toY(tick);
        return (
          <g key={tick}>
            {tick > 0 && (
              <line
                x1={MARGIN.left}
                y1={y}
                x2={MARGIN.left + PLOT_W}
                y2={y}
                stroke="#d6d3d1"
                strokeWidth={0.5}
                strokeDasharray="3,3"
              />
            )}
            <text
              x={MARGIN.left - 6}
              y={y + 3}
              textAnchor="end"
              fontSize={9}
              fill="#a8a29e"
            >
              {tick}
            </text>
          </g>
        );
      })}

      {/* Y-as label */}
      <text
        x={12}
        y={MARGIN.top + PLOT_H / 2}
        textAnchor="middle"
        fontSize={10}
        fill="#78716c"
        transform={`rotate(-90, 12, ${MARGIN.top + PLOT_H / 2})`}
      >
        Dampdruk [Pa]
      </text>

      {/* Condensatiezone */}
      {condensationPath && (
        <path d={condensationPath} fill="#fca5a5" fillOpacity={0.45} />
      )}

      {/* pSat curve (blauw) */}
      <path
        d={pSatPath}
        fill="none"
        stroke="#2563eb"
        strokeWidth={2.5}
        strokeLinejoin="round"
      />

      {/* pActual lijn (amber) */}
      <path
        d={pActualPath}
        fill="none"
        stroke="#d97706"
        strokeWidth={2.5}
        strokeLinejoin="round"
        strokeDasharray="6,3"
      />

      {/* Temperatuurlijn (gradient rood→blauw) */}
      <path
        d={tempPath}
        fill="none"
        stroke="url(#temp-gradient)"
        strokeWidth={1.8}
        strokeLinejoin="round"
      />

      {/* Punten op interfaces */}
      {interfacePoints.map((p, i) => (
        <g key={i}>
          <circle cx={toX(p.x)} cy={toY(p.pSat)} r={3} fill="#2563eb" />
          <circle
            cx={toX(p.x)}
            cy={toY(p.pActual)}
            r={3}
            fill="#d97706"
          />
        </g>
      ))}

      {/* Rechter Y-as: Temperatuur [°C] */}
      {(() => {
        const tempRange = tempMax - tempMin;
        const tickCount = Math.min(6, Math.max(3, Math.ceil(tempRange / 5)));
        const step = tempRange / tickCount;
        const ticks: number[] = [];
        for (let i = 0; i <= tickCount; i++) {
          ticks.push(Math.round((tempMin + step * i) * 10) / 10);
        }
        return ticks.map((t) => {
          const y = toYTemp(t);
          return (
            <g key={`temp-${t}`}>
              <line
                x1={MARGIN.left + PLOT_W}
                y1={y}
                x2={MARGIN.left + PLOT_W + 4}
                y2={y}
                stroke="#ef4444"
                strokeWidth={0.8}
              />
              <text
                x={MARGIN.left + PLOT_W + 7}
                y={y + 3}
                fontSize={9}
                fill="#ef4444"
              >
                {t.toFixed(0)}°
              </text>
            </g>
          );
        });
      })()}

      {/* Rechter Y-as label */}
      <text
        x={WIDTH - 8}
        y={MARGIN.top + PLOT_H / 2}
        textAnchor="middle"
        fontSize={10}
        fill="#ef4444"
        transform={`rotate(90, ${WIDTH - 8}, ${MARGIN.top + PLOT_H / 2})`}
      >
        Temperatuur [°C]
      </text>

      {/* Temperatuurlabels bij interface-punten */}
      {interfacePoints.map((p, i) => {
        const x = toX(p.x);
        if (
          interfacePoints.length > 4 &&
          i !== 0 &&
          i !== interfacePoints.length - 1
        )
          return null;
        return (
          <text
            key={`t-${i}`}
            x={x}
            y={MARGIN.top + PLOT_H + 28}
            textAnchor="middle"
            fontSize={9}
            fill="#78716c"
          >
            {p.temperature.toFixed(1)}°C
          </text>
        );
      })}

      {/* Binnen/Buiten labels */}
      <text
        x={MARGIN.left + 4}
        y={MARGIN.top + PLOT_H + 46}
        fontSize={10}
        fontWeight={600}
        fill="#57534e"
      >
        Binnen ({thetaI}°C)
      </text>
      <text
        x={MARGIN.left + PLOT_W - 4}
        y={MARGIN.top + PLOT_H + 46}
        textAnchor="end"
        fontSize={10}
        fontWeight={600}
        fill="#57534e"
      >
        Buiten ({thetaE}°C)
      </text>

      {/* Legenda */}
      <g
        transform={`translate(${MARGIN.left + PLOT_W - 200}, ${MARGIN.top + 8})`}
      >
        <rect
          x={-6}
          y={-4}
          width={206}
          height={58}
          rx={4}
          fill="white"
          fillOpacity={0.92}
          stroke="#d6d3d1"
          strokeWidth={0.5}
        />
        <line
          x1={0}
          y1={8}
          x2={18}
          y2={8}
          stroke="#2563eb"
          strokeWidth={2.5}
        />
        <circle cx={9} cy={8} r={3} fill="#2563eb" />
        <text x={24} y={11} fontSize={10} fill="#57534e">
          Verzadigingsdruk (p_sat)
        </text>
        <line
          x1={0}
          y1={24}
          x2={18}
          y2={24}
          stroke="#d97706"
          strokeWidth={2.5}
          strokeDasharray="6,3"
        />
        <circle cx={9} cy={24} r={3} fill="#d97706" />
        <text x={24} y={27} fontSize={10} fill="#57534e">
          Werkelijke dampdruk (p)
        </text>
        <defs>
          <linearGradient id="legend-temp-grad" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" stopColor="#ef4444" />
            <stop offset="100%" stopColor="#3b82f6" />
          </linearGradient>
        </defs>
        <line
          x1={0}
          y1={40}
          x2={18}
          y2={40}
          stroke="url(#legend-temp-grad)"
          strokeWidth={1.8}
        />
        <text x={24} y={43} fontSize={10} fill="#ef4444">
          Temperatuur (°C)
        </text>
      </g>
    </svg>
  );
}
