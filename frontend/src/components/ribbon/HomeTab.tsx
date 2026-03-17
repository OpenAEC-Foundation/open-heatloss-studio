import { useState } from "react";
import { useTranslation } from "react-i18next";
import RibbonGroup from "./RibbonGroup";
import RibbonButton from "./RibbonButton";
import RibbonButtonStack from "./RibbonButtonStack";
import {
  clipboardIcon, cutIcon, copyIcon,
  undoIcon, redoIcon,
  boldIcon, italicIcon, underlineIcon,
  alignLeftIcon, alignCenterIcon, alignRightIcon,
  imageIcon, tableIcon, linkIcon,
  lineIcon, rectangleIcon, circleIcon, pencilIcon, arrowIcon,
} from "./icons";

type DrawingTool = "line" | "rectangle" | "circle" | "pencil" | "arrow" | null;

export default function HomeTab() {
  const { t } = useTranslation("ribbon");
  const [activeTool, setActiveTool] = useState<DrawingTool>(null);

  const toggleTool = (tool: DrawingTool) => {
    setActiveTool((prev) => (prev === tool ? null : tool));
  };

  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label={t("home.clipboard")}>
          <RibbonButton icon={clipboardIcon} label={t("home.paste")} title="Ctrl+V" />
          <RibbonButtonStack>
            <RibbonButton size="small" icon={cutIcon} label={t("home.cut")} title="Ctrl+X" />
            <RibbonButton size="small" icon={copyIcon} label={t("home.copy")} title="Ctrl+C" />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("home.history")}>
          <RibbonButtonStack>
            <RibbonButton size="small" icon={undoIcon} label={t("home.undo")} title="Ctrl+Z" />
            <RibbonButton size="small" icon={redoIcon} label={t("home.redo")} title="Ctrl+Y" />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("home.font")}>
          <RibbonButtonStack>
            <RibbonButton size="small" icon={boldIcon} label={t("home.bold")} title="Ctrl+B" />
            <RibbonButton size="small" icon={italicIcon} label={t("home.italic")} title="Ctrl+I" />
            <RibbonButton size="small" icon={underlineIcon} label={t("home.underline")} title="Ctrl+U" />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("home.paragraph")}>
          <RibbonButtonStack>
            <RibbonButton size="small" icon={alignLeftIcon} label={t("home.alignLeft")} />
            <RibbonButton size="small" icon={alignCenterIcon} label={t("home.center")} />
            <RibbonButton size="small" icon={alignRightIcon} label={t("home.alignRight")} />
          </RibbonButtonStack>
        </RibbonGroup>

        <RibbonGroup label={t("home.drawing")}>
          <RibbonButton icon={pencilIcon} label={t("home.pencil")} active={activeTool === "pencil"} onClick={() => toggleTool("pencil")} />
          <RibbonButtonStack>
            <RibbonButton size="small" icon={lineIcon} label={t("home.line")} active={activeTool === "line"} onClick={() => toggleTool("line")} />
            <RibbonButton size="small" icon={arrowIcon} label={t("home.arrow")} active={activeTool === "arrow"} onClick={() => toggleTool("arrow")} />
            <RibbonButton size="small" icon={rectangleIcon} label={t("home.rectangle")} active={activeTool === "rectangle"} onClick={() => toggleTool("rectangle")} />
          </RibbonButtonStack>
          <RibbonButton icon={circleIcon} label={t("home.circle")} active={activeTool === "circle"} onClick={() => toggleTool("circle")} />
        </RibbonGroup>

        <RibbonGroup label={t("home.insert")}>
          <RibbonButton icon={imageIcon} label={t("home.image")} />
          <RibbonButton icon={tableIcon} label={t("home.table")} />
          <RibbonButton icon={linkIcon} label={t("home.link")} />
        </RibbonGroup>
      </div>
    </div>
  );
}
