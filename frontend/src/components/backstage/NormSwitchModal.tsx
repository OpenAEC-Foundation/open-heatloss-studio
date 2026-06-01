import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "../Modal";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import {
  LOSSY_51_TO_53_HEATING_KEYS,
  LOSSY_53_TO_51_HEATING_KEYS,
  MAP_51_TO_53,
  deriveIsso53BuildingFromIsso51,
  deriveIsso53RoomsFromIsso51,
  deriveIsso51BuildingTypeFromIsso53,
  isIsso51Heating,
  isIsso53Heating,
  mapHeatingSystem,
  mapRoom53To51,
  writeNormSwitchBackup,
} from "../../lib/normSwitch";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
} from "../../types/projectV2";
import type { ActiveNorm } from "../../types/projectV2";
import "./NormChoiceModal.css";
import "./NormSwitchModal.css";

interface NormSwitchModalProps {
  open: boolean;
  onClose: () => void;
}

/**
 * Fase 4: wissel-flow tussen ISSO 51 en ISSO 53.
 *
 * Trigger: klik op de norm-badge in `TitleBar`. Toont een waarschuwing met:
 *   - tekstuele uitleg over data-conversie
 *   - preview-tabel van de ruimte-functie mapping
 *   - "Annuleren" / "Wissel naar ISSO XX" actie
 *
 * Bij bevestiging:
 *   1. Schrijft een back-up van het huidige project naar disk (Tauri) of
 *      blob-download (web).
 *   2. Past data-mapping toe op `rooms[].function` / `isso53Rooms` en
 *      `building.building_type` / `isso53Building`.
 *   3. Flipt `store.norm`.
 *   4. Toont een toast met het back-up pad.
 */
export default function NormSwitchModal({ open, onClose }: NormSwitchModalProps) {
  const { t } = useTranslation("backstage");
  const addToast = useToastStore((s) => s.addToast);
  const [isSwitching, setIsSwitching] = useState(false);

  // Subscribe — bij elk open lezen we de huidige state via getState() in de
  // handler, maar voor de preview-tabel willen we live-updates bij b.v.
  // het toevoegen van een ruimte vlak voor open.
  const currentNorm = useProjectStore((s) => s.norm);
  const rooms = useProjectStore((s) => s.project.rooms);
  const isso53Rooms = useProjectStore((s) => s.isso53Rooms);

  const targetNorm: ActiveNorm = currentNorm === "isso51" ? "isso53" : "isso51";
  const currentLabel = currentNorm === "isso51" ? "ISSO 51" : "ISSO 53";
  const targetLabel = targetNorm === "isso51" ? "ISSO 51" : "ISSO 53";

  /**
   * Preview-rijen voor de mapping-tabel. Bij 51→53 tonen we per ruimte
   * de gekozen `RoomFunction` + de afgeleide `GebruiksFunctie.RuimteType`.
   * Bij 53→51 tonen we de omgekeerde mapping naar `living_room` —
   * informatief: alle ruimten worden default LivingRoom.
   */
  const previewRows = useMemo(() => {
    if (rooms.length === 0) return [];
    if (currentNorm === "isso51") {
      return rooms.map((r) => {
        const target = MAP_51_TO_53[r.function];
        return {
          id: r.id,
          name: r.name || t("normSwitch.roomFallback"),
          from: t(`normSwitch.roomFunction.${r.function}`, r.function),
          to: `${t(`normSwitch.gebruiksFunctie.${target.gebruiksFunctie}`, target.gebruiksFunctie)} · ${t(`normSwitch.ruimteType.${target.ruimteType}`, target.ruimteType)}`,
        };
      });
    }
    // 53 → 51
    return rooms.map((r) => {
      const sidecar = isso53Rooms[r.id];
      const fromLabel = sidecar
        ? `${t(`normSwitch.gebruiksFunctie.${sidecar.gebruiksFunctie}`, sidecar.gebruiksFunctie)} · ${t(`normSwitch.ruimteType.${sidecar.ruimteType}`, sidecar.ruimteType)}`
        : t("normSwitch.roomFallback");
      return {
        id: r.id,
        name: r.name || t("normSwitch.roomFallback"),
        from: fromLabel,
        to: t("normSwitch.roomFunction.living_room", "Woonkamer"),
      };
    });
  }, [currentNorm, rooms, isso53Rooms, t]);

  const handleConfirm = async () => {
    if (isSwitching) return;
    setIsSwitching(true);
    try {
      const state = useProjectStore.getState();

      // 1. Back-up
      const backupPath = await writeNormSwitchBackup({
        project: state.project,
        result: state.result,
        currentNorm: state.norm,
        currentLocalPath: state.currentLocalPath,
        isso53Building: state.isso53Building,
        isso53Rooms: state.isso53Rooms,
      });

      // 2. Mapping toepassen + norm-flip in één set-call om geen tussen-
      //    rendering te triggeren waar UI nog de oude norm met nieuwe rooms ziet.
      let lossyHeatingCount = 0;
      if (state.norm === "isso51") {
        // 51 → 53
        const newIsso53Rooms = deriveIsso53RoomsFromIsso51(state.project);
        const newIsso53Building = deriveIsso53BuildingFromIsso51(state.project);
        // Map verwarmingssystemen voor building-default + alle rooms naar de
        // ISSO 53-set. Tel lossy keys (geen 1-op-1 norm-equivalent) voor
        // de toast-waarschuwing.
        const oldDefault = state.project.building.default_heating_system;
        if (isIsso51Heating(oldDefault) && LOSSY_51_TO_53_HEATING_KEYS.has(oldDefault)) {
          lossyHeatingCount += 1;
        }
        const mappedDefault = mapHeatingSystem(oldDefault, "isso51", "isso53");
        const mappedRooms = state.project.rooms.map((r) => {
          if (isIsso51Heating(r.heating_system) && LOSSY_51_TO_53_HEATING_KEYS.has(r.heating_system)) {
            lossyHeatingCount += 1;
          }
          return {
            ...r,
            heating_system: mapHeatingSystem(r.heating_system, "isso51", "isso53"),
          };
        });
        useProjectStore.setState({
          norm: "isso53",
          project: {
            ...state.project,
            rooms: mappedRooms,
            building: {
              ...state.project.building,
              default_heating_system: mappedDefault,
            },
          },
          isso53Rooms: newIsso53Rooms,
          isso53Building: newIsso53Building,
          isDirty: true,
        });
      } else {
        // 53 → 51: alle rooms → living_room; building_type uit positie afleiden.
        const oldDefault = state.project.building.default_heating_system;
        if (isIsso53Heating(oldDefault) && LOSSY_53_TO_51_HEATING_KEYS.has(oldDefault)) {
          lossyHeatingCount += 1;
        }
        const mappedDefault = mapHeatingSystem(oldDefault, "isso53", "isso51");
        const updatedRooms = state.project.rooms.map((r) => {
          if (isIsso53Heating(r.heating_system) && LOSSY_53_TO_51_HEATING_KEYS.has(r.heating_system)) {
            lossyHeatingCount += 1;
          }
          return {
            ...r,
            function: mapRoom53To51(
              state.isso53Rooms[r.id] ?? { ...DEFAULT_ISSO53_ROOM },
            ),
            heating_system: mapHeatingSystem(r.heating_system, "isso53", "isso51"),
          };
        });
        const newBuildingType = deriveIsso51BuildingTypeFromIsso53(
          state.isso53Building,
        );
        useProjectStore.setState({
          norm: "isso51",
          project: {
            ...state.project,
            rooms: updatedRooms,
            building: {
              ...state.project.building,
              building_type: newBuildingType,
              default_heating_system: mappedDefault,
            },
          },
          // Reset sidecar — ISSO 53 sidecar is niet meer relevant na de wissel.
          isso53Building: { ...DEFAULT_ISSO53_BUILDING },
          isso53Rooms: {},
          isDirty: true,
        });
      }
      if (lossyHeatingCount > 0) {
        addToast(
          t("normSwitch.heatingLossy", {
            count: lossyHeatingCount,
            defaultValue: `${lossyHeatingCount} verwarmingssyst(e)em(en) zonder norm-equivalent — best-effort gemapt. Controleer per vertrek.`,
          }),
          "info",
          6000,
        );
      }

      // 3. Toast
      const newLabel = state.norm === "isso51" ? "ISSO 53" : "ISSO 51";
      const msg = backupPath
        ? t("normSwitch.successWithBackup", {
            norm: newLabel,
            path: backupPath,
            defaultValue: `Project gewisseld naar ${newLabel}. Back-up: ${backupPath}`,
          })
        : t("normSwitch.successWithoutBackup", {
            norm: newLabel,
            defaultValue: `Project gewisseld naar ${newLabel}. Back-up gedownload.`,
          });
      addToast(msg, "success", 5000);
      onClose();
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      addToast(
        t("normSwitch.error", {
          detail,
          defaultValue: `Wissel mislukt: ${detail}`,
        }),
        "error",
      );
    } finally {
      setIsSwitching(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t("normSwitch.title", { defaultValue: "Norm wisselen" })}
      width={560}
      className="norm-switch-modal"
      footer={
        <>
          <button
            type="button"
            className="norm-choice-btn norm-choice-btn-secondary"
            onClick={onClose}
            disabled={isSwitching}
          >
            {t("normSwitch.cancel", { defaultValue: "Annuleren" })}
          </button>
          <button
            type="button"
            className="norm-choice-btn norm-choice-btn-primary"
            onClick={() => void handleConfirm()}
            disabled={isSwitching}
          >
            {isSwitching
              ? t("normSwitch.switching", { defaultValue: "Bezig…" })
              : t("normSwitch.confirm", {
                  norm: targetLabel,
                  defaultValue: `Wissel naar ${targetLabel}`,
                })}
          </button>
        </>
      }
    >
      <div className="norm-switch-body">
        <p className="norm-switch-lede">
          {t("normSwitch.intro", {
            from: currentLabel,
            to: targetLabel,
            defaultValue: `Je staat op het punt dit project van ${currentLabel} naar ${targetLabel} te wisselen.`,
          })}
        </p>

        <div className="norm-switch-warning">
          <div className="norm-switch-warning-title">
            {t("normSwitch.warningTitle", {
              defaultValue: "Niet alle gegevens worden 1-op-1 overgenomen:",
            })}
          </div>
          <ul className="norm-switch-warning-list">
            <li>
              {t("normSwitch.warningRooms", {
                defaultValue:
                  "Ruimte-functies worden gemapt (zie hieronder) — controleer per ruimte.",
              })}
            </li>
            <li>
              {t("normSwitch.warningVentilation", {
                defaultValue:
                  "Ventilatie-eis verandert: per-m² (wonen) ↔ per-persoon × bezetting (utiliteit).",
              })}
            </li>
            <li>
              {t("normSwitch.warningOperation", {
                defaultValue:
                  "Bedrijfsbeperking-toeslag: main-room-percentage ↔ specifieke toeslag P [W/m²].",
              })}
            </li>
          </ul>
        </div>

        <p className="norm-switch-backup-note">
          {t("normSwitch.backupNote", {
            file: `<naam> (v ${currentLabel} backup).json`,
            defaultValue: `Een back-up wordt opgeslagen als '<naam> (v ${currentLabel} backup).json' in dezelfde map.`,
          })}
        </p>

        {previewRows.length > 0 && (
          <div className="norm-switch-preview">
            <div className="norm-switch-preview-title">
              {t("normSwitch.previewTitle", {
                defaultValue: "Mapping per ruimte (preview)",
              })}
            </div>
            <div className="norm-switch-preview-table" role="table">
              <div className="norm-switch-preview-row norm-switch-preview-head" role="row">
                <span role="columnheader">
                  {t("normSwitch.colRoom", { defaultValue: "Ruimte" })}
                </span>
                <span role="columnheader">
                  {t("normSwitch.colFrom", {
                    norm: currentLabel,
                    defaultValue: `Huidig (${currentLabel})`,
                  })}
                </span>
                <span role="columnheader" aria-hidden="true">→</span>
                <span role="columnheader">
                  {t("normSwitch.colTo", {
                    norm: targetLabel,
                    defaultValue: `Nieuw (${targetLabel})`,
                  })}
                </span>
              </div>
              {previewRows.map((row) => (
                <div key={row.id} className="norm-switch-preview-row" role="row">
                  <span role="cell" className="norm-switch-preview-name">{row.name}</span>
                  <span role="cell">{row.from}</span>
                  <span role="cell" aria-hidden="true">→</span>
                  <span role="cell">{row.to}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {previewRows.length === 0 && (
          <p className="norm-switch-empty">
            {t("normSwitch.noRooms", {
              defaultValue: "Geen ruimten in dit project — alleen building-velden worden gemapt.",
            })}
          </p>
        )}
      </div>
    </Modal>
  );
}
