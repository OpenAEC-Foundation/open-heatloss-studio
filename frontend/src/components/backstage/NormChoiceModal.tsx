import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import Modal from "../Modal";
import type { ActiveNorm } from "../../types/projectV2";
import "./NormChoiceModal.css";

interface NormChoiceModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (norm: ActiveNorm) => void;
  /** Voorgeselecteerde optie. Default: `"isso51"`. */
  defaultNorm?: ActiveNorm;
}

/**
 * Modal die getoond wordt bij "Bestand → Nieuw" en de gebruiker dwingt
 * te kiezen tussen ISSO 51 (wonen) en ISSO 53 (utiliteit ≤ 4m).
 *
 * Fase 2 van het ISSO 53 UI-werkpakket: norm-keuze is permanent in dit
 * scherm — wisselen volgt in fase 4 via de topbar-badge.
 */
export default function NormChoiceModal({
  open,
  onClose,
  onConfirm,
  defaultNorm = "isso51",
}: NormChoiceModalProps) {
  const { t } = useTranslation("backstage");
  const [selected, setSelected] = useState<ActiveNorm>(defaultNorm);

  // Reset selectie bij elke opening — anders blijft een eerdere keuze
  // staan en is de default visueel niet meer kloppend.
  useEffect(() => {
    if (open) setSelected(defaultNorm);
  }, [open, defaultNorm]);

  const handleConfirm = () => {
    onConfirm(selected);
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t("normChoice.title")}
      width={420}
      className="norm-choice-modal"
      footer={
        <>
          <button
            type="button"
            className="norm-choice-btn norm-choice-btn-secondary"
            onClick={onClose}
          >
            {t("normChoice.cancel")}
          </button>
          <button
            type="button"
            className="norm-choice-btn norm-choice-btn-primary"
            onClick={handleConfirm}
          >
            {t("normChoice.continue")}
          </button>
        </>
      }
    >
      <div className="norm-choice-body">
        <p className="norm-choice-question">{t("normChoice.question")}</p>

        <label className="norm-choice-option">
          <input
            type="radio"
            name="norm-choice"
            value="isso51"
            checked={selected === "isso51"}
            onChange={() => setSelected("isso51")}
          />
          <span className="norm-choice-label">
            <strong>{t("normChoice.wonen")}</strong>
            <span className="norm-choice-sub">{t("normChoice.wonenSub")}</span>
          </span>
        </label>

        <label className="norm-choice-option">
          <input
            type="radio"
            name="norm-choice"
            value="isso53"
            checked={selected === "isso53"}
            onChange={() => setSelected("isso53")}
          />
          <span className="norm-choice-label">
            <strong>{t("normChoice.utiliteit")}</strong>
            <span className="norm-choice-sub">
              {t("normChoice.utiliteitSub")}
            </span>
          </span>
        </label>
      </div>
    </Modal>
  );
}
