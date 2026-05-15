/**
 * Rapport opmaak instellingen — modale dialoog (OpenAEC huisstijl).
 *
 * V1: alleen footer-afbeelding. Modal-skelet staat klaar om uit te breiden
 * met header-image / marges / lettertype / accent-kleur in V2.
 */
import { useCallback, useRef } from "react";

import Modal from "../Modal";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import type { CoverImage } from "../../types";
import "./RapportOpmaakDialog.css";

const MAX_IMAGE_SIZE = 2 * 1024 * 1024; // 2 MB

interface RapportOpmaakDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function RapportOpmaakDialog({
  open,
  onClose,
}: RapportOpmaakDialogProps) {
  const project = useProjectStore((s) => s.project);
  const updateProject = useProjectStore((s) => s.updateProject);
  const addToast = useToastStore((s) => s.addToast);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const footerImage = project.info.footer_image ?? null;

  const updateProjectInfo = useCallback(
    (partial: Partial<typeof project.info>) => {
      updateProject({ info: { ...project.info, ...partial } });
    },
    [updateProject, project.info],
  );

  const handleFooterChange = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      if (file.size > MAX_IMAGE_SIZE) {
        addToast("Afbeelding is groter dan 2 MB.", "error");
        e.target.value = "";
        return;
      }
      if (file.type !== "image/png" && file.type !== "image/jpeg") {
        addToast("Alleen PNG of JPEG worden ondersteund.", "error");
        e.target.value = "";
        return;
      }
      try {
        const dataUrl: string = await new Promise((resolve, reject) => {
          const fr = new FileReader();
          fr.onload = () => resolve(String(fr.result));
          fr.onerror = () => reject(fr.error);
          fr.readAsDataURL(file);
        });
        const base64 = dataUrl.replace(/^data:[^;]+;base64,/, "");
        const image: CoverImage = {
          data: base64,
          media_type: file.type as "image/png" | "image/jpeg",
          filename: file.name,
        };
        updateProjectInfo({ footer_image: image });
        addToast("Footer-afbeelding opgeslagen.", "success");
      } catch (err) {
        addToast(
          `Inlezen mislukt: ${err instanceof Error ? err.message : String(err)}`,
          "error",
        );
      }
      e.target.value = "";
    },
    [updateProjectInfo, addToast],
  );

  const handleFooterClear = useCallback(() => {
    updateProjectInfo({ footer_image: null });
    addToast("Footer-afbeelding verwijderd.", "info");
  }, [updateProjectInfo, addToast]);

  const footer = (
    <button
      className="rapport-opmaak-btn rapport-opmaak-btn-primary"
      onClick={onClose}
    >
      Sluiten
    </button>
  );

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Rapport opmaak"
      width={520}
      footer={footer}
    >
      <div className="rapport-opmaak-content">
        <p className="rapport-opmaak-intro">
          Opmaak-instellingen voor het PDF rapport. Wijzigingen worden direct
          opgeslagen in het project en toegepast bij de volgende "Genereren".
        </p>

        <section className="rapport-opmaak-section">
          <h3 className="rapport-opmaak-section-title">Footer-afbeelding</h3>
          <p className="rapport-opmaak-section-desc">
            Verschijnt onderaan elke content-pagina (boven het paginanummer).
            PNG of JPEG, max. 2 MB. Aanbevolen breedte 1500 px voor scherpe weergave.
          </p>

          {footerImage ? (
            <div className="rapport-opmaak-image-block">
              <img
                src={`data:${footerImage.media_type};base64,${footerImage.data}`}
                alt="Footer"
                className="rapport-opmaak-preview"
              />
              <div className="rapport-opmaak-image-row">
                <span className="rapport-opmaak-filename">
                  {footerImage.filename ?? "afbeelding"}
                </span>
                <div className="rapport-opmaak-image-actions">
                  <button
                    type="button"
                    onClick={() => fileInputRef.current?.click()}
                    className="rapport-opmaak-btn rapport-opmaak-btn-secondary"
                  >
                    Andere kiezen…
                  </button>
                  <button
                    type="button"
                    onClick={handleFooterClear}
                    className="rapport-opmaak-btn rapport-opmaak-btn-danger"
                  >
                    Verwijderen
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <button
              type="button"
              onClick={() => fileInputRef.current?.click()}
              className="rapport-opmaak-drop"
            >
              Afbeelding kiezen…
            </button>
          )}

          <input
            ref={fileInputRef}
            type="file"
            accept="image/png,image/jpeg"
            onChange={handleFooterChange}
            className="rapport-opmaak-hidden"
          />
        </section>

        <p className="rapport-opmaak-roadmap">
          Binnenkort uitbreidbaar: header-afbeelding, marges, lettertype en
          accent-kleur.
        </p>
      </div>
    </Modal>
  );
}
