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
    </div>
  );
}
