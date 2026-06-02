/**
 * ISSO 53 room-functie celcombinatie (fase 3).
 *
 * Vervangt de ISSO 51 `RoomFunction`-dropdown in `RoomHeaderCells` met
 * twee compacte gekoppelde dropdowns:
 *   - GebruiksFunctie (kantoor/onderwijs/…) — Bouwbesluit
 *   - RuimteType (verblijfsruimte/badruimte/…)
 *
 * Beide opties zijn vlak — de norm wijst per combinatie de waardes toe
 * (geen UI-filtering nodig). State leeft in `projectStore.isso53Rooms`
 * gekeyed op `room.id`.
 */
import { useTranslation } from "react-i18next";

import { useProjectStore } from "../../store/projectStore";
import type {
  Isso53GebruiksFunctie,
  Isso53RuimteType,
} from "../../types/projectV2";
import { EditableSelect } from "./EditableSelect";

const GEBRUIKS_FUNCTIES: Isso53GebruiksFunctie[] = [
  "kantoor",
  "onderwijs",
  "gezondheidszorg",
  "bijeenkomst",
  "logies",
  "sport",
  "winkel",
  "cel",
  "industrie",
];

const RUIMTE_TYPES: Isso53RuimteType[] = [
  "verblijfsruimte",
  "verblijfsgebied",
  "badruimte",
  "toiletruimte",
  "verkeersruimte",
  "technischeRuimte",
  "bergruimte",
  "onbenoemdeRuimte",
  "stallingsruimte",
  "garage",
  "kantoorruimte",
  "receptie",
  "lesruimte",
  "collegezaal",
  "werkplaats",
  "bureauruimte",
  "patientenkamer",
  "operatiekamer",
  "onderzoekruimte",
  "eetruimte",
  "restaurant",
  "kantine",
  "vergaderruimte",
  "hotelkamer",
  "sportzaal",
  "verkoopruimte",
  "supermarkt",
  "warenhuis",
];

interface Isso53RoomFunctionCellProps {
  roomId: string;
}

export function Isso53RoomFunctionCell({ roomId }: Isso53RoomFunctionCellProps) {
  const { t } = useTranslation();
  const sidecar = useProjectStore((s) => s.isso53Rooms[roomId]);
  const updateIsso53Room = useProjectStore((s) => s.updateIsso53Room);

  const gebruiksFunctie: Isso53GebruiksFunctie =
    sidecar?.gebruiksFunctie ?? "kantoor";
  const ruimteType: Isso53RuimteType = sidecar?.ruimteType ?? "verblijfsruimte";

  const gfOptions: Record<string, string> = Object.fromEntries(
    GEBRUIKS_FUNCTIES.map((v) => [
      v,
      t(`isso53.room.gebruiksFunctieOptions.${v}`),
    ]),
  );
  const rtOptions: Record<string, string> = Object.fromEntries(
    RUIMTE_TYPES.map((v) => [v, t(`isso53.room.ruimteTypeOptions.${v}`)]),
  );

  const personen = sidecar?.personen ?? undefined;
  const zFactor = sidecar?.infiltrationReductionZ ?? 1.0;
  const ventilationEstablished = sidecar?.ventilationEstablished ?? undefined;
  const zOptions: Record<string, string> = {
    "1": t("isso53.room.zFactorOptions.1"),
    "0.7": t("isso53.room.zFactorOptions.0.7"),
    "0.5": t("isso53.room.zFactorOptions.0.5"),
  };

  return (
    <div className="flex flex-col gap-0.5">
      <EditableSelect
        value={gebruiksFunctie}
        onChange={(v) =>
          updateIsso53Room(roomId, {
            gebruiksFunctie: v as Isso53GebruiksFunctie,
          })
        }
        options={gfOptions}
      />
      <EditableSelect
        value={ruimteType}
        onChange={(v) =>
          updateIsso53Room(roomId, { ruimteType: v as Isso53RuimteType })
        }
        options={rtOptions}
      />
      <label className="flex items-center gap-1 text-xs text-on-surface-variant">
        <span className="shrink-0">{t("isso53.room.personenLabel")}</span>
        <input
          type="number"
          min={0}
          step={1}
          value={personen ?? ""}
          placeholder={t("isso53.room.personenPlaceholder")}
          onChange={(e) => {
            const raw = e.target.value;
            updateIsso53Room(roomId, {
              personen: raw === "" ? null : Number(raw),
            });
          }}
          className="w-full rounded border-none bg-transparent px-1 py-0.5 text-xs
            text-on-surface outline-none hover:bg-[var(--oaec-hover)]
            focus:bg-[var(--oaec-bg-input)] focus:ring-1 focus:ring-primary"
        />
      </label>
      <label className="flex items-center gap-1 text-xs text-on-surface-variant">
        <span className="shrink-0">{t("isso53.room.zFactorLabel")}</span>
        <EditableSelect
          value={String(zFactor)}
          onChange={(v) =>
            updateIsso53Room(roomId, { infiltrationReductionZ: Number(v) })
          }
          options={zOptions}
          className="text-xs"
        />
      </label>
      <label className="flex items-center gap-1 text-xs text-on-surface-variant">
        <span className="shrink-0">
          {t("isso53.room.ventilationEstablishedLabel")}
        </span>
        <input
          type="number"
          min={0}
          step={0.1}
          value={ventilationEstablished ?? ""}
          placeholder={t("isso53.room.ventilationEstablishedPlaceholder")}
          onChange={(e) => {
            const raw = e.target.value;
            updateIsso53Room(roomId, {
              ventilationEstablished: raw === "" ? undefined : Number(raw),
            });
          }}
          className="w-full rounded border-none bg-transparent px-1 py-0.5 text-xs
            text-on-surface outline-none hover:bg-[var(--oaec-hover)]
            focus:bg-[var(--oaec-bg-input)] focus:ring-1 focus:ring-primary"
        />
        <span className="shrink-0 text-[10px] text-on-surface-muted">dm³/s</span>
        {ventilationEstablished != null && ventilationEstablished > 0 && (
          <span className="shrink-0 text-[10px] text-on-surface-muted tabular-nums">
            ({(ventilationEstablished * 3.6).toFixed(1)} m³/h)
          </span>
        )}
      </label>
    </div>
  );
}
