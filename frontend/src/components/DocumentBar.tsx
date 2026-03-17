import { useState } from "react";
import "./DocumentBar.css";

interface DocTab {
  id: string;
  title: string;
  modified?: boolean;
}

const INITIAL_DOCS: DocTab[] = [
  { id: "1", title: "Project Overview.oaec", modified: false },
  { id: "2", title: "Floor Plan - Level 1.oaec", modified: true },
  { id: "3", title: "Structural Analysis.oaec", modified: false },
];

export default function DocumentBar() {
  const [docs, setDocs] = useState<DocTab[]>(INITIAL_DOCS);
  const [activeId, setActiveId] = useState("1");

  const closeDoc = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const remaining = docs.filter((d) => d.id !== id);
    setDocs(remaining);
    if (activeId === id && remaining.length > 0 && remaining[0]) {
      setActiveId(remaining[0].id);
    }
  };

  return (
    <div className="document-bar">
      <div className="document-tabs">
        {docs.map((doc) => (
          <button
            key={doc.id}
            className={`document-tab${activeId === doc.id ? " active" : ""}`}
            onClick={() => setActiveId(doc.id)}
          >
<span className="document-tab-title">{doc.title}</span>
            {doc.modified && <span className="document-tab-modified" />}
            <span
              className="document-tab-close"
              onClick={(e) => closeDoc(doc.id, e)}
            >
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <path d="M2.5 2.5l5 5M7.5 2.5l-5 5" />
              </svg>
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
