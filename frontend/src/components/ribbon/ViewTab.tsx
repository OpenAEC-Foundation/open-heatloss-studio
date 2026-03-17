import { useTranslation } from "react-i18next";
import RibbonGroup from "./RibbonGroup";
import RibbonButton from "./RibbonButton";
import RibbonButtonStack from "./RibbonButtonStack";
import { zoomInIcon, zoomOutIcon, gridIcon, settingsIcon } from "./icons";

export default function ViewTab() {
  const { t } = useTranslation("ribbon");

  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label={t("view.zoom")}>
          <RibbonButtonStack>
            <RibbonButton size="small" icon={zoomInIcon} label={t("view.zoomIn")} />
            <RibbonButton size="small" icon={zoomOutIcon} label={t("view.zoomOut")} />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("view.layout")}>
          <RibbonButton icon={gridIcon} label={t("view.grid")} />
        </RibbonGroup>

        <RibbonGroup label={t("view.preferences")}>
          <RibbonButton icon={settingsIcon} label={t("view.settings")} />
        </RibbonGroup>
      </div>
    </div>
  );
}
