import { useState, useEffect, useRef, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { LANGUAGES, changeLanguage } from "../../i18n/config";
import { getSetting, setSetting } from "../../tauriStore";
import Modal from "../Modal";
import ThemedSelect from "../ThemedSelect";
import "../ThemedSelect.css";
import "./SettingsDialog.css";

const THEME_OPTIONS = [
  { value: "light", labelKey: "appearance.light", swatches: ["#3E3636", "#D97706", "#F5F0EB", "#B45309"] },
  { value: "openaec", labelKey: "appearance.dark", swatches: ["#1a1a2e", "#D97706", "#C4B199", "#B45309"] },
];

const TAB_IDS = ["general", "appearance", "editor", "files", "shortcuts", "plugins", "about"] as const;

export function applyTheme(theme?: string) {
  document.documentElement.setAttribute("data-theme", theme || "light");
}

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
  theme: string;
  onThemeChange: (theme: string) => void;
}

export default function SettingsDialog({
  open,
  onClose,
  theme,
  onThemeChange,
}: SettingsDialogProps) {
  const { t } = useTranslation("settings");
  const { t: tCommon } = useTranslation("common");
  const [activeTab, setActiveTab] = useState("general");

  // Draft state — only committed on Save
  const [draftTheme, setDraftTheme] = useState(theme);
  const [draftLang, setDraftLang] = useState("auto");
  const [confirmResetOpen, setConfirmResetOpen] = useState(false);

  // Snapshot of original values when dialog opens, for reverting on Cancel
  const originalTheme = useRef(theme);
  const originalLang = useRef("");

  // Reset draft to current values when dialog opens
  useEffect(() => {
    if (open) {
      originalTheme.current = theme;
      setDraftTheme(theme);
      getSetting("language", "auto").then((lang) => {
        originalLang.current = lang;
        setDraftLang(lang);
      });
    }
  }, [open, theme]);

  // Cancel — discard all draft changes, restore originals
  const handleCancel = () => {
    setDraftTheme(originalTheme.current);
    setDraftLang(originalLang.current);
    onClose();
  };

  // Save — commit all draft changes
  const handleSave = () => {
    onThemeChange(draftTheme);
    applyTheme(draftTheme);
    setSetting("theme", draftTheme);

    setSetting("language", draftLang);
    changeLanguage(draftLang);

    onClose();
  };

  // Reset to defaults — resets draft values (still requires Save to apply)
  const handleReset = () => {
    setConfirmResetOpen(true);
  };

  const handleConfirmReset = () => {
    setDraftTheme("light");
    setDraftLang("auto");
    setConfirmResetOpen(false);
  };

  const footer = (
    <>
      <button className="settings-btn settings-btn-secondary" onClick={handleReset}>
        {t("resetToDefaults")}
      </button>
      <div className="settings-footer-right">
        <button className="settings-btn settings-btn-secondary" onClick={handleCancel}>
          {tCommon("cancel")}
        </button>
        <button className="settings-btn settings-btn-primary" onClick={handleSave}>
          {tCommon("save")}
        </button>
      </div>
    </>
  );

  return (
    <>
    <Modal open={open} onClose={handleCancel} title={t("title")} width={560} height={500} className="settings-dialog" footer={footer}>
      <div className="settings-body">
        <div className="settings-sidebar">
          {TAB_IDS.map((id) => (
            <button
              key={id}
              className={`settings-tab${activeTab === id ? " active" : ""}`}
              onClick={() => setActiveTab(id)}
            >
              {t(`tabs.${id}`)}
            </button>
          ))}
        </div>

        <div className="settings-content">
          {activeTab === "general" && (
            <GeneralTabContent lang={draftLang} onLangChange={setDraftLang} />
          )}
          {activeTab === "appearance" && (
            <AppearanceTabContent theme={draftTheme} onThemeSelect={setDraftTheme} />
          )}
          {activeTab === "editor" && <EditorTabContent />}
          {activeTab === "files" && <FilesTabContent />}
          {activeTab === "shortcuts" && <ShortcutsTabContent />}
          {activeTab === "plugins" && <PluginsTabContent />}
          {activeTab === "about" && <AboutTabContent />}
        </div>
      </div>
    </Modal>

    <Modal
      open={confirmResetOpen}
      onClose={() => setConfirmResetOpen(false)}
      title={t("resetToDefaults")}
      width={340}
      footer={
        <>
          <button className="settings-btn settings-btn-secondary" onClick={() => setConfirmResetOpen(false)}>
            {tCommon("cancel")}
          </button>
          <button className="settings-btn settings-btn-primary" onClick={handleConfirmReset}>
            {t("resetToDefaults")}
          </button>
        </>
      }
    >
      <div style={{ padding: 12, fontSize: 12 }}>{t("resetConfirm")}</div>
    </Modal>
    </>
  );
}

function GeneralTabContent({
  lang,
  onLangChange,
}: {
  lang: string;
  onLangChange: (value: string) => void;
}) {
  const { t } = useTranslation("settings");

  return (
    <>
      <div className="settings-section">
        <h3>{t("general.application")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("general.language")}</span>
          <ThemedSelect
            value={lang}
            options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
            onChange={onLangChange}
            style={{ width: 180 }}
          />
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("general.startup")}</h3>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="restore-session" />
          <label htmlFor="restore-session">{t("general.restoreSession")}</label>
        </div>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="check-updates" defaultChecked />
          <label htmlFor="check-updates">{t("general.checkUpdates")}</label>
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("general.author")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("general.authorName")}</span>
          <input className="settings-input" type="text" placeholder={t("general.authorName")} />
        </div>
      </div>
    </>
  );
}

function AppearanceTabContent({
  theme,
  onThemeSelect,
}: {
  theme: string;
  onThemeSelect: (value: string) => void;
}) {
  const { t } = useTranslation("settings");
  const [fontSize, setFontSize] = useState("14");
  return (
    <>
      <div className="settings-section">
        <h3>{t("appearance.theme")}</h3>
        <ThemeDropdown theme={theme} onThemeSelect={onThemeSelect} />
      </div>

      <div className="settings-section">
        <h3>{t("appearance.font")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("appearance.uiFontSize")}</span>
          <ThemedSelect
            value={fontSize}
            options={[
              { value: "12", label: "12px" },
              { value: "13", label: "13px" },
              { value: "14", label: "14px" },
              { value: "15", label: "15px" },
              { value: "16", label: "16px" },
            ]}
            onChange={setFontSize}
          />
        </div>
      </div>
    </>
  );
}

function ThemeDropdown({
  theme,
  onThemeSelect,
}: {
  theme: string;
  onThemeSelect: (value: string) => void;
}) {
  const { t } = useTranslation("settings");
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [open]);

  // THEME_OPTIONS is a non-empty const array so fallback is always valid
  const selected = THEME_OPTIONS.find((o) => o.value === theme) ?? THEME_OPTIONS[0]!;

  const swatchRow = (swatches: string[]) => (
    <div className="theme-dropdown-swatches">
      {swatches.map((color, i) => (
        <span key={i} className="theme-dropdown-swatch" style={{ backgroundColor: color } as CSSProperties} />
      ))}
    </div>
  );

  return (
    <div className="theme-dropdown" ref={ref}>
      <button className="theme-dropdown-trigger" onClick={() => setOpen(!open)}>
        {swatchRow(selected.swatches)}
        <span className="theme-dropdown-label">{t(selected.labelKey)}</span>
        <svg className="theme-dropdown-chevron" width="10" height="10" viewBox="0 0 10 10" fill="none">
          <path d="M2.5 4L5 6.5L7.5 4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>
      {open && (
        <div className="theme-dropdown-menu">
          {THEME_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              className={`theme-dropdown-item${theme === opt.value ? " active" : ""}`}
              onClick={() => { onThemeSelect(opt.value); setOpen(false); }}
            >
              {swatchRow(opt.swatches)}
              <span className="theme-dropdown-label">{t(opt.labelKey)}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function EditorTabContent() {
  const { t } = useTranslation("settings");
  const [tabSize, setTabSize] = useState("4");
  return (
    <>
      <div className="settings-section">
        <h3>{t("editor.general")}</h3>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="word-wrap" defaultChecked />
          <label htmlFor="word-wrap">{t("editor.wordWrap")}</label>
        </div>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="line-numbers" defaultChecked />
          <label htmlFor="line-numbers">{t("editor.lineNumbers")}</label>
        </div>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="minimap" />
          <label htmlFor="minimap">{t("editor.minimap")}</label>
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("editor.indentation")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("editor.tabSize")}</span>
          <ThemedSelect
            value={tabSize}
            options={[
              { value: "2", label: t("editor.nSpaces", { count: 2 }) },
              { value: "4", label: t("editor.nSpaces", { count: 4 }) },
              { value: "8", label: t("editor.nSpaces", { count: 8 }) },
            ]}
            onChange={setTabSize}
          />
        </div>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="insert-spaces" defaultChecked />
          <label htmlFor="insert-spaces">{t("editor.insertSpaces")}</label>
        </div>
      </div>
    </>
  );
}

function FilesTabContent() {
  const { t } = useTranslation("settings");
  const [autoSave, setAutoSave] = useState("off");
  const [encoding, setEncoding] = useState("utf-8");
  return (
    <>
      <div className="settings-section">
        <h3>{t("files.autoSave")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("files.autoSaveOption")}</span>
          <ThemedSelect
            value={autoSave}
            options={[
              { value: "off", label: t("files.off") },
              { value: "afterDelay", label: t("files.afterDelay") },
              { value: "onFocusChange", label: t("files.onFocusChange") },
              { value: "onWindowChange", label: t("files.onWindowChange") },
            ]}
            onChange={setAutoSave}
          />
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("files.encoding")}</h3>
        <div className="settings-row">
          <span className="settings-label">{t("files.defaultEncoding")}</span>
          <ThemedSelect
            value={encoding}
            options={[
              { value: "utf-8", label: "UTF-8" },
              { value: "utf-16", label: "UTF-16" },
              { value: "ascii", label: "ASCII" },
            ]}
            onChange={setEncoding}
          />
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("files.backup")}</h3>
        <div className="settings-checkbox-row">
          <input type="checkbox" id="auto-backup" defaultChecked />
          <label htmlFor="auto-backup">{t("files.autoBackup")}</label>
        </div>
      </div>
    </>
  );
}

function ShortcutsTabContent() {
  const { t } = useTranslation("settings");
  return (
    <div className="settings-placeholder">
      <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
      </svg>
      <p>{t("shortcuts.placeholder")}</p>
    </div>
  );
}

function PluginsTabContent() {
  const { t } = useTranslation("settings");
  return (
    <div className="settings-placeholder">
      <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
      </svg>
      <p>{t("plugins.placeholder")}</p>
    </div>
  );
}

function AboutTabContent() {
  const { t } = useTranslation("settings");
  return (
    <div className="settings-section">
      <h3>ISSO 51 Warmteverliesberekening</h3>
      <div style={{ fontSize: 11, lineHeight: 1.8 }}>
        <p><strong>{t("about.version")}:</strong> 0.1.0</p>
        <p><strong>{t("about.framework")}:</strong> Tauri + React + TypeScript + Rust</p>
        <p><strong>{t("about.license")}:</strong> MIT</p>
        <p style={{ marginTop: 8, color: "var(--theme-dialog-content-secondary)" }}>
          Warmteverliesberekening volgens ISSO 51:2023.
        </p>
      </div>
    </div>
  );
}
