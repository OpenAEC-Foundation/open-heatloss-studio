import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { plusIcon, calculatorIcon } from "./icons";
import { useProjectStore } from "../../store/projectStore";
import { createRoom } from "../../lib/roomDefaults";
import { useRunCalculation } from "../../hooks/useRunCalculation";
import { useToastStore } from "../../store/toastStore";

export default function VertrekkenTab() {
  const { t } = useTranslation("ribbon");
  const { t: tc } = useTranslation("common");
  const navigate = useNavigate();
  const addRoom = useProjectStore((s) => s.addRoom);
  const project = useProjectStore((s) => s.project);
  const isCalculating = useProjectStore((s) => s.isCalculating);
  const addToast = useToastStore((s) => s.addToast);
  const runCalculation = useRunCalculation();
  const hasRooms = project.rooms.length > 0;

  const handleAddRoom = () => {
    addRoom(createRoom(project.building.default_heating_system));
    navigate("/rooms");
  };

  const handleCalculate = async () => {
    const ok = await runCalculation();
    if (ok) {
      addToast(tc("calculationComplete"), "success");
      navigate("/results");
    } else {
      const msg = useProjectStore.getState().error ?? tc("calculationFailed");
      addToast(msg, "error");
    }
  };

  return (
    <>
      <RibbonGroup label={t("vertrekken.rooms")}>
        <RibbonButton
          icon={plusIcon}
          label={t("vertrekken.addRoom")}
          onClick={handleAddRoom}
        />
      </RibbonGroup>
      <RibbonGroup label={t("vertrekken.calculation")}>
        <RibbonButton
          icon={calculatorIcon}
          label={t("vertrekken.calculate")}
          disabled={!hasRooms || isCalculating}
          onClick={handleCalculate}
        />
      </RibbonGroup>
    </>
  );
}
