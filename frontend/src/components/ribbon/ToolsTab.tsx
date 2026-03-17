import { useTranslation } from "react-i18next";
import RibbonGroup from "./RibbonGroup";
import RibbonButton from "./RibbonButton";
import RibbonButtonStack from "./RibbonButtonStack";
import { settingsIcon, shareIcon, printIcon, saveIcon } from "./icons";

const spellcheckIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>`;

const languageIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"/></svg>`;

const terminalIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>`;

const pluginIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z"/></svg>`;

const exportIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/></svg>`;

export default function ToolsTab() {
  const { t } = useTranslation("ribbon");

  return (
    <div className="ribbon-content active">
      <div className="ribbon-groups">
        <RibbonGroup label={t("tools.file")}>
          <RibbonButton icon={saveIcon} label={t("tools.save")} title="Ctrl+S" />
          <RibbonButton icon={exportIcon} label={t("tools.export")} />
          <RibbonButtonStack>
            <RibbonButton size="small" icon={printIcon} label={t("tools.print")} title="Ctrl+P" />
            <RibbonButton size="small" icon={shareIcon} label={t("tools.share")} />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("tools.proofing")}>
          <RibbonButtonStack>
            <RibbonButton size="small" icon={spellcheckIcon} label={t("tools.spellCheck")} />
            <RibbonButton size="small" icon={languageIcon} label={t("tools.language")} />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("tools.extensions")}>
          <RibbonButton icon={pluginIcon} label={t("tools.plugins")} />
          <RibbonButton icon={terminalIcon} label={t("tools.console")} />
        </RibbonGroup>

        <RibbonGroup label={t("tools.options")}>
          <RibbonButton icon={settingsIcon} label={t("tools.settings")} />
        </RibbonGroup>
      </div>
    </div>
  );
}
