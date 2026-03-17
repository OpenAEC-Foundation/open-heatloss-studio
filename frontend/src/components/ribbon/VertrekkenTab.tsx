import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { plusIcon, calculatorIcon } from "./icons";
import { useProjectStore } from "../../store/projectStore";
import { createRoom } from "../../lib/roomDefaults";
import { createBackend } from "../../lib/backend";
import { useToastStore } from "../../store/toastStore";

const backend = createBackend();

export default function VertrekkenTab() {
  const { t } = useTranslation("ribbon");
  const { t: tc } = useTranslation("common");
  const navigate = useNavigate();
  const addRoom = useProjectStore((s) => s.addRoom);
  const project = useProjectStore((s) => s.project);
  const setResult = useProjectStore((s) => s.setResult);
  const setError = useProjectStore((s) => s.setError);
  const setCalculating = useProjectStore((s) => s.setCalculating);
  const isCalculating = useProjectStore((s) => s.isCalculating);
  const addToast = useToastStore((s) => s.addToast);
  const hasRooms = project.rooms.length > 0;

  const handleAddRoom = () => {
    addRoom(createRoom());
    navigate("/rooms");
  };

  const handleCalculate = async () => {
    setCalculating(true);
    try {
      const result = await backend.calculate(project);
      setResult(result);
      addToast(tc("calculationComplete"), "success");
      navigate("/results");
    } catch (err) {
      const msg = err instanceof Error ? err.message : tc("calculationFailed");
      setError(msg);
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
