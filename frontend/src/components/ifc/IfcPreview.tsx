/**
 * IFC Preview — split-pane viewer voor IFC4X3 (STEP) en IFCX (JSON).
 *
 * Mirror van Open Calc Studio's `components/report/IfcPreview.tsx`. Toont
 * de **gegenereerde** IFC representaties van het huidige project — niet
 * een geladen externe file. Linker paneel: STEP file met syntax highlighting
 * + line numbers. Rechter paneel: IFCX JSON als collapsible tree.
 *
 * Verschillen met OCS:
 * - OCS bron-data = cost schedule + items; OHS bron-data = project + result
 *   + modeller (via buildIfcEnergyDocument).
 * - OCS gebruikt `generateIfcCostFile` voor STEP; OHS gebruikt
 *   `generateIfcStepFromProject` (eigen implementatie, IfcProject + Site +
 *   Building + Spaces + Walls/Slabs/Roofs met isso51:: PropertySets).
 * - Cross-link STEP↔JSON via ifcGuid is voorbereid maar niet gewireed —
 *   onze STEP en IFCX gebruiken (nog) niet dezelfde GUIDs. Een toekomstige
 *   PR kan beide registries delen voor click-to-sync.
 */
import "./ifc-preview.css";
import React, { useCallback, useMemo, useState } from "react";

import { useProjectStore } from "../../store/projectStore";
import { useModellerStore } from "../modeller/modellerStore";
import { generateIfcStepFromProject } from "../../lib/ifcStepGenerator";
import {
  buildIfcEnergyDocument,
  serializeIfcEnergy,
} from "../../lib/ifcenergy";

// ───────────────────────────────────────────────────────────────────
// Syntax highlighting for STEP lines (verbatim van OCS)
// ───────────────────────────────────────────────────────────────────

function highlightStepLine(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let remaining = text;
  let key = 0;

  if (/^(ISO-10303-21|HEADER|ENDSEC|DATA|END-ISO-10303-21);?\s*$/.test(remaining.trim())) {
    return <span key={key} className="step-keyword">{text}</span>;
  }
  if (/^FILE_(DESCRIPTION|NAME|SCHEMA)\b/.test(remaining.trim())) {
    const match = remaining.match(/^(FILE_\w+)/);
    if (match) {
      parts.push(<span key={key++} className="step-keyword">{match[1]}</span>);
      remaining = remaining.slice(match[1]!.length);
    }
  }

  let i = 0;
  let buf = "";
  const flushBuf = () => {
    if (buf) {
      parts.push(<span key={key++}>{buf}</span>);
      buf = "";
    }
  };

  while (i < remaining.length) {
    const ch = remaining[i]!;
    if (ch === "#" && i + 1 < remaining.length && /\d/.test(remaining[i + 1]!)) {
      flushBuf();
      let ref = "#";
      i++;
      while (i < remaining.length && /\d/.test(remaining[i]!)) ref += remaining[i++];
      parts.push(<span key={key++} className="step-entity-ref">{ref}</span>);
      continue;
    }
    if (ch === "'") {
      flushBuf();
      let str = "'";
      i++;
      while (i < remaining.length && remaining[i] !== "'") str += remaining[i++];
      if (i < remaining.length) str += remaining[i++];
      parts.push(<span key={key++} className="step-string">{str}</span>);
      continue;
    }
    if (ch === "=" && i + 1 < remaining.length && /[A-Z]/.test(remaining[i + 1]!)) {
      flushBuf();
      i++;
      let entityType = "";
      while (i < remaining.length && /[A-Z0-9_]/.test(remaining[i]!)) entityType += remaining[i++];
      parts.push(<span key={key++}>=</span>);
      parts.push(<span key={key++} className="step-entity-type">{entityType}</span>);
      continue;
    }
    if (ch === "." && i + 1 < remaining.length && /[A-Z$_]/.test(remaining[i + 1]!)) {
      flushBuf();
      let enumVal = ".";
      i++;
      while (i < remaining.length && remaining[i] !== "." && /[A-Z0-9$_]/.test(remaining[i]!)) {
        enumVal += remaining[i++];
      }
      if (i < remaining.length && remaining[i] === ".") {
        enumVal += ".";
        i++;
      }
      parts.push(<span key={key++} className="step-enum">{enumVal}</span>);
      continue;
    }
    if (/\d/.test(ch) && (i === 0 || /[=,(]/.test(remaining[i - 1]!))) {
      flushBuf();
      let num = "";
      while (i < remaining.length && /[\d.eE+-]/.test(remaining[i]!)) num += remaining[i++];
      parts.push(<span key={key++} className="step-number">{num}</span>);
      continue;
    }
    buf += ch;
    i++;
  }
  flushBuf();
  return <>{parts}</>;
}

// ───────────────────────────────────────────────────────────────────
// JSON tree renderer (verbatim van OCS)
// ───────────────────────────────────────────────────────────────────

interface JsonTreeNode {
  type: "object" | "array" | "string" | "number" | "boolean" | "null";
  key?: string;
  value?: unknown;
  children?: JsonTreeNode[];
  itemCount?: number;
  nodeId?: string;
}

function parseJsonTree(value: unknown, key?: string, pathPrefix?: string): JsonTreeNode {
  const nodePath = pathPrefix ? (key ? `${pathPrefix}.${key}` : pathPrefix) : key || "root";
  if (value === null) return { type: "null", key, value: null, nodeId: nodePath };
  if (typeof value === "string") return { type: "string", key, value, nodeId: nodePath };
  if (typeof value === "number") return { type: "number", key, value, nodeId: nodePath };
  if (typeof value === "boolean") return { type: "boolean", key, value, nodeId: nodePath };
  if (Array.isArray(value)) {
    const children = value.map((item, i) => parseJsonTree(item, String(i), nodePath));
    return { type: "array", key, children, itemCount: value.length, nodeId: nodePath };
  }
  if (typeof value === "object") {
    const obj = value as Record<string, unknown>;
    const children = Object.entries(obj).map(([k, v]) => parseJsonTree(v, k, nodePath));
    return {
      type: "object",
      key,
      children,
      itemCount: Object.keys(obj).length,
      nodeId: nodePath,
    };
  }
  return { type: "null", key, value, nodeId: nodePath };
}

function renderJsonValue(value: unknown): React.ReactNode {
  if (value === null) return <span className="step-keyword">null</span>;
  if (typeof value === "boolean") return <span className="step-keyword">{String(value)}</span>;
  if (typeof value === "number") return <span className="step-number">{value}</span>;
  if (typeof value === "string") {
    if (/^Ifc\w+$/.test(value)) return <span className="step-entity-type">"{value}"</span>;
    return <span className="step-string">"{value}"</span>;
  }
  return <span>{String(value)}</span>;
}

interface JsonTreeRowProps {
  node: JsonTreeNode;
  depth: number;
  collapsed: Set<string>;
  onToggle: (nodeId: string) => void;
  isLast: boolean;
}

const JsonTreeRow: React.FC<JsonTreeRowProps> = ({
  node, depth, collapsed, onToggle, isLast,
}) => {
  const nodeId = node.nodeId || "";
  const isCollapsible = node.type === "object" || node.type === "array";
  const isCollapsed = collapsed.has(nodeId);
  const indent = depth * 16;
  const comma = isLast ? "" : ",";
  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isCollapsible) onToggle(nodeId);
  };

  const rows: React.ReactNode[] = [];

  if (!isCollapsible) {
    rows.push(
      <div key={nodeId} className="json-tree-row" style={{ paddingLeft: indent }}>
        {node.key !== undefined && <><span className="json-key">"{node.key}"</span>: </>}
        {renderJsonValue(node.value)}{comma}
      </div>,
    );
  } else {
    const openBrace = node.type === "object" ? "{" : "[";
    const closeBrace = node.type === "object" ? "}" : "]";

    rows.push(
      <div
        key={`${nodeId}-open`}
        className="json-tree-row json-tree-collapsible"
        style={{ paddingLeft: indent }}
      >
        <span className="json-tree-toggle" onClick={handleToggle}>
          {isCollapsed ? "▶" : "▼"}
        </span>
        {node.key !== undefined && <><span className="json-key">"{node.key}"</span>: </>}
        {isCollapsed ? (
          <span className="json-tree-collapsed-preview">
            {openBrace}<span className="step-comment">
              {node.type === "object" ? `…${node.itemCount} props` : `…${node.itemCount} items`}
            </span>{closeBrace}{comma}
          </span>
        ) : (
          <span>{openBrace}</span>
        )}
      </div>,
    );

    if (!isCollapsed && node.children) {
      for (let i = 0; i < node.children.length; i++) {
        rows.push(
          <JsonTreeRow
            key={node.children[i]!.nodeId || `${nodeId}-${i}`}
            node={node.children[i]!}
            depth={depth + 1}
            collapsed={collapsed}
            onToggle={onToggle}
            isLast={i === node.children.length - 1}
          />,
        );
      }
      rows.push(
        <div key={`${nodeId}-close`} className="json-tree-row" style={{ paddingLeft: indent }}>
          {closeBrace}{comma}
        </div>,
      );
    }
  }

  return <>{rows}</>;
};

// ───────────────────────────────────────────────────────────────────
// Main component
// ───────────────────────────────────────────────────────────────────

interface StepLine {
  lineNumber: number;
  text: string;
}

export const IfcPreview: React.FC = () => {
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const modellerState = useModellerStore();
  const [splitPos, setSplitPos] = useState(50);
  const [isDragging, setIsDragging] = useState(false);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

  // --- Generated content ---
  const stepContent = useMemo(() => generateIfcStepFromProject(project), [project]);
  const ifcxContent = useMemo(() => {
    const doc = buildIfcEnergyDocument({
      project,
      result,
      modeller: {
        rooms: modellerState.rooms,
        windows: modellerState.windows,
        doors: modellerState.doors,
        projectConstructions: modellerState.projectConstructions,
        wallConstructions: modellerState.wallConstructions,
        floorConstructions: modellerState.floorConstructions,
        roofConstructions: modellerState.roofConstructions,
        wallBoundaryTypes: modellerState.wallBoundaryTypes,
        underlay: modellerState.underlay,
      },
    });
    return serializeIfcEnergy(doc);
  }, [project, result, modellerState]);

  const stepLines: StepLine[] = useMemo(
    () => stepContent.split("\n").map((text, i) => ({ lineNumber: i + 1, text })),
    [stepContent],
  );

  const jsonTree = useMemo(() => {
    try {
      return parseJsonTree(JSON.parse(ifcxContent));
    } catch {
      return null;
    }
  }, [ifcxContent]);

  // Initial collapse: depth > 3
  const [collapsedInitialized, setCollapsedInitialized] = useState(false);
  React.useEffect(() => {
    if (collapsedInitialized || !jsonTree) return;
    const set = new Set<string>();
    function walk(n: JsonTreeNode, depth: number) {
      if ((n.type === "object" || n.type === "array") && n.nodeId && depth > 3) {
        set.add(n.nodeId);
      }
      if (n.children) for (const c of n.children) walk(c, depth + 1);
    }
    walk(jsonTree, 0);
    setCollapsed(set);
    setCollapsedInitialized(true);
  }, [jsonTree, collapsedInitialized]);

  const handleToggle = useCallback((nodeId: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(nodeId)) next.delete(nodeId);
      else next.add(nodeId);
      return next;
    });
  }, []);

  const stepFileSize = useMemo(() => {
    const bytes = new Blob([stepContent]).size;
    return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`;
  }, [stepContent]);
  const ifcxFileSize = useMemo(() => {
    const bytes = new Blob([ifcxContent]).size;
    return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`;
  }, [ifcxContent]);
  const ifcxLineCount = useMemo(() => ifcxContent.split("\n").length, [ifcxContent]);

  const projectName = project.info.name || "project";
  const safeName = projectName.replace(/[^a-zA-Z0-9_\-\s]/g, "").trim() || "project";

  const handleDownloadStep = () => {
    const blob = new Blob([stepContent], { type: "application/x-step" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${safeName}.ifc`;
    a.click();
    URL.revokeObjectURL(url);
  };
  const handleDownloadIfcx = () => {
    const blob = new Blob([ifcxContent], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${safeName}.ifcenergy`;
    a.click();
    URL.revokeObjectURL(url);
  };
  const handleCopyStep = () => navigator.clipboard.writeText(stepContent);
  const handleCopyIfcx = () => navigator.clipboard.writeText(ifcxContent);

  // Splitter drag
  const handleSplitterMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    const container = (e.target as HTMLElement).parentElement!;
    const rect = container.getBoundingClientRect();
    const onMouseMove = (ev: MouseEvent) => {
      const pct = ((ev.clientX - rect.left) / rect.width) * 100;
      setSplitPos(Math.min(80, Math.max(20, pct)));
    };
    const onMouseUp = () => {
      setIsDragging(false);
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }, []);

  return (
    <div className={`ifc-preview ifc-split${isDragging ? " ifc-dragging" : ""}`}>
      {/* Left: STEP (IFC4X3) */}
      <div className="ifc-panel" style={{ width: `${splitPos}%` }}>
        <div className="ifc-toolbar">
          <div className="ifc-toolbar-info">
            <span className="ifc-toolbar-label">IFC4X3 (STEP)</span>
            <span className="ifc-toolbar-meta">{stepLines.length} regels</span>
            <span className="ifc-toolbar-meta">{stepFileSize}</span>
          </div>
          <div className="ifc-toolbar-actions">
            <button className="ifc-toolbar-btn" onClick={handleCopyStep}>Kopieer</button>
            <button className="ifc-toolbar-btn" onClick={handleDownloadStep}>Download .ifc</button>
          </div>
        </div>
        <div className="ifc-code">
          <table className="ifc-step-table">
            <tbody>
              {stepLines.map((line) => (
                <tr key={line.lineNumber}>
                  <td className="ifc-step-linenum">{line.lineNumber}</td>
                  <td className="ifc-step-text">{highlightStepLine(line.text)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <div className="ifc-splitter" onMouseDown={handleSplitterMouseDown} />

      {/* Right: IFCX JSON */}
      <div className="ifc-panel" style={{ width: `${100 - splitPos}%` }}>
        <div className="ifc-toolbar">
          <div className="ifc-toolbar-info">
            <span className="ifc-toolbar-label">IFCX (.ifcenergy)</span>
            <span className="ifc-toolbar-meta">{ifcxLineCount} regels</span>
            <span className="ifc-toolbar-meta">{ifcxFileSize}</span>
          </div>
          <div className="ifc-toolbar-actions">
            <button className="ifc-toolbar-btn" onClick={handleCopyIfcx}>Kopieer</button>
            <button className="ifc-toolbar-btn" onClick={handleDownloadIfcx}>Download .ifcenergy</button>
          </div>
        </div>
        <div className="ifc-code ifc-json-tree">
          {jsonTree ? (
            <JsonTreeRow
              node={jsonTree}
              depth={0}
              collapsed={collapsed}
              onToggle={handleToggle}
              isLast={true}
            />
          ) : (
            <div className="ifc-json-error">Failed to parse JSON</div>
          )}
        </div>
      </div>
    </div>
  );
};
