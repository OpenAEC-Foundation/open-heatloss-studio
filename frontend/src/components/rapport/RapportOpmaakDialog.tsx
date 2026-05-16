/**
 * Rapport opmaak instellingen — modale dialoog (OpenAEC huisstijl).
 *
 * V2: footer-afbeelding + header-afbeelding + marges + accent-kleur.
 * Lettertype is V3 (vereist extra TTF files in resources/fonts/).
 */
import { useCallback, useMemo, useRef } from "react";

import Modal from "../Modal";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import type { CoverImage, ReportStyle } from "../../types";
import "./RapportOpmaakDialog.css";

const MAX_IMAGE_SIZE = 2 * 1024 * 1024; // 2 MB

/** Defaults — gespiegeld op Rust-side wanneer style.* ontbreekt. */
const STYLE_DEFAULTS = {
  margin_top_mm: 20,
  margin_bottom_mm: 28,
  margin_horizontal_mm: 15,
  accent_color_hex: "0F766E",
} as const;

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
  const headerInputRef = useRef<HTMLInputElement>(null);
  const footerInputRef = useRef<HTMLInputElement>(null);

  const footerImage = project.info.footer_image ?? null;
  const headerImage = project.info.header_image ?? null;
  const style = useMemo<ReportStyle>(
    () => project.info.report_style ?? {},
    [project.info.report_style],
  );

  const updateProjectInfo = useCallback(
    (partial: Partial<typeof project.info>) => {
      updateProject({ info: { ...project.info, ...partial } });
    },
    [updateProject, project.info],
  );

  const updateStyle = useCallback(
    (partial: Partial<ReportStyle>) => {
      updateProjectInfo({ report_style: { ...style, ...partial } });
    },
    [updateProjectInfo, style],
  );

  /** Generic image-upload handler — sets project.info[field] = {data, media_type, filename}. */
  const handleImageUpload = useCallback(
    async (
      e: React.ChangeEvent<HTMLInputElement>,
      field: "header_image" | "footer_image",
      successMsg: string,
    ) => {
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
        updateProjectInfo({ [field]: image });
        addToast(successMsg, "success");
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

  const footerCB = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) =>
      handleImageUpload(e, "footer_image", "Footer-afbeelding opgeslagen."),
    [handleImageUpload],
  );
  const headerCB = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) =>
      handleImageUpload(e, "header_image", "Header-afbeelding opgeslagen."),
    [handleImageUpload],
  );

  const handleFooterClear = useCallback(() => {
    updateProjectInfo({ footer_image: null });
    addToast("Footer-afbeelding verwijderd.", "info");
  }, [updateProjectInfo, addToast]);

  const handleHeaderClear = useCallback(() => {
    updateProjectInfo({ header_image: null });
    addToast("Header-afbeelding verwijderd.", "info");
  }, [updateProjectInfo, addToast]);

  const handleStyleReset = useCallback(() => {
    updateProjectInfo({ report_style: null });
    addToast("Opmaak teruggezet naar defaults.", "info");
  }, [updateProjectInfo, addToast]);

  const renderImageField = (
    label: string,
    desc: string,
    image: CoverImage | null,
    inputRef: React.RefObject<HTMLInputElement | null>,
    onChange: (e: React.ChangeEvent<HTMLInputElement>) => void,
    onClear: () => void,
    altText: string,
  ) => (
    <section className="rapport-opmaak-section">
      <h3 className="rapport-opmaak-section-title">{label}</h3>
      <p className="rapport-opmaak-section-desc">{desc}</p>

      {image ? (
        <div className="rapport-opmaak-image-block">
          <img
            src={`data:${image.media_type};base64,${image.data}`}
            alt={altText}
            className="rapport-opmaak-preview"
          />
          <div className="rapport-opmaak-image-row">
            <span className="rapport-opmaak-filename">
              {image.filename ?? "afbeelding"}
            </span>
            <div className="rapport-opmaak-image-actions">
              <button
                type="button"
                onClick={() => inputRef.current?.click()}
                className="rapport-opmaak-btn rapport-opmaak-btn-secondary"
              >
                Andere kiezen…
              </button>
              <button
                type="button"
                onClick={onClear}
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
          onClick={() => inputRef.current?.click()}
          className="rapport-opmaak-drop"
        >
          Afbeelding kiezen…
        </button>
      )}

      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg"
        onChange={onChange}
        className="rapport-opmaak-hidden"
      />
    </section>
  );

  const dialogFooter = (
    <>
      <button
        className="rapport-opmaak-btn rapport-opmaak-btn-secondary"
        onClick={handleStyleReset}
      >
        Reset opmaak
      </button>
      <button
        className="rapport-opmaak-btn rapport-opmaak-btn-primary"
        onClick={onClose}
      >
        Sluiten
      </button>
    </>
  );

  // Voor de hex-color input — normaliseert "#0F766E" → "0F766E" en valideert
  // dat het 6 hex-digits is voordat 'ie wordt opgeslagen.
  const accentValue = style.accent_color_hex ?? STYLE_DEFAULTS.accent_color_hex;
  const accentPreviewColor = /^[0-9a-fA-F]{6}$/.test(accentValue)
    ? `#${accentValue}`
    : `#${STYLE_DEFAULTS.accent_color_hex}`;

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Rapport opmaak"
      width={560}
      footer={dialogFooter}
    >
      <div className="rapport-opmaak-content">
        <p className="rapport-opmaak-intro">
          Opmaak-instellingen voor het PDF rapport. Wijzigingen worden direct
          opgeslagen in het project en toegepast bij de volgende "Genereren".
        </p>

        {renderImageField(
          "Header-afbeelding",
          "Bedrijfslogo of beeldmerk — rechtsboven op elke content-pagina. Max. 14mm hoog, aspect-ratio behouden. PNG of JPEG, max. 2 MB.",
          headerImage,
          headerInputRef,
          headerCB,
          handleHeaderClear,
          "Header",
        )}

        {renderImageField(
          "Footer-afbeelding",
          "Verschijnt onderaan elke content-pagina (boven het paginanummer). PNG of JPEG, max. 2 MB. Aanbevolen breedte 1500 px voor scherpe weergave.",
          footerImage,
          footerInputRef,
          footerCB,
          handleFooterClear,
          "Footer",
        )}

        <section className="rapport-opmaak-section">
          <h3 className="rapport-opmaak-section-title">Marges</h3>
          <p className="rapport-opmaak-section-desc">
            Witruimte rond de content (in mm). Defaults: 20 / 28 / 15.
            Toegestaan bereik: 5 – 80 mm.
          </p>
          <div className="rapport-opmaak-margins-grid">
            <label className="rapport-opmaak-margin-field">
              <span>Boven</span>
              <input
                type="number"
                min={5}
                max={80}
                step={1}
                value={style.margin_top_mm ?? STYLE_DEFAULTS.margin_top_mm}
                onChange={(e) =>
                  updateStyle({ margin_top_mm: Number(e.target.value) })
                }
              />
              <small>mm</small>
            </label>
            <label className="rapport-opmaak-margin-field">
              <span>Onder</span>
              <input
                type="number"
                min={5}
                max={80}
                step={1}
                value={style.margin_bottom_mm ?? STYLE_DEFAULTS.margin_bottom_mm}
                onChange={(e) =>
                  updateStyle({ margin_bottom_mm: Number(e.target.value) })
                }
              />
              <small>mm</small>
            </label>
            <label className="rapport-opmaak-margin-field">
              <span>Links / rechts</span>
              <input
                type="number"
                min={5}
                max={80}
                step={1}
                value={
                  style.margin_horizontal_mm ??
                  STYLE_DEFAULTS.margin_horizontal_mm
                }
                onChange={(e) =>
                  updateStyle({ margin_horizontal_mm: Number(e.target.value) })
                }
              />
              <small>mm</small>
            </label>
          </div>
        </section>

        <section className="rapport-opmaak-section">
          <h3 className="rapport-opmaak-section-title">Accent-kleur</h3>
          <p className="rapport-opmaak-section-desc">
            Gebruikt voor de header-lijn, tabelkop en accent-elementen.
            Default: <code>0F766E</code> (OpenAEC teal).
          </p>
          <div className="rapport-opmaak-accent-row">
            <input
              type="color"
              className="rapport-opmaak-color"
              value={accentPreviewColor}
              onChange={(e) =>
                updateStyle({
                  accent_color_hex: e.target.value.replace(/^#/, "").toUpperCase(),
                })
              }
            />
            <input
              type="text"
              className="rapport-opmaak-color-text"
              value={accentValue}
              maxLength={7}
              placeholder="0F766E"
              onChange={(e) => {
                const v = e.target.value.replace(/^#/, "").toUpperCase();
                if (v.length === 0 || /^[0-9A-F]{1,6}$/.test(v)) {
                  updateStyle({ accent_color_hex: v });
                }
              }}
            />
            <span
              className="rapport-opmaak-color-preview"
              style={{ background: accentPreviewColor }}
              title={`Preview: ${accentPreviewColor}`}
            />
          </div>
        </section>

        <p className="rapport-opmaak-roadmap">
          Volgende uitbreiding: lettertype-keuze (extra TTF-files vereist in
          resources/fonts/).
        </p>
      </div>
    </Modal>
  );
}
