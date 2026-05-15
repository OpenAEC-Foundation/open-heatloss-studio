import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import Modal from "../Modal";
import "./FeedbackDialog.css";

/** GitHub repo waar issues naartoe gaan. */
const GITHUB_REPO = "OpenAEC-Foundation/open-heatloss-studio";

async function getAppVersion(): Promise<string> {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    return await getVersion();
  } catch {
    return "";
  }
}

async function buildUserAgent(appVer: string): Promise<string> {
  try {
    const os = await import("@tauri-apps/plugin-os");
    const osType = os.type() || "Unknown";
    const osVer = os.version() || "";
    const arch = os.arch() || "";
    return `Warmteverlies/${appVer || "0.0.0"} (${osType} ${osVer}; ${arch})`
      .replace(/\s+/g, " ")
      .trim();
  } catch {
    return `Warmteverlies/${appVer || "0.0.0"}`;
  }
}

const CATEGORIES = ["general", "bug", "feature"] as const;
type Category = (typeof CATEGORIES)[number];

/** GitHub-label per categorie. Labels moeten in de repo bestaan; "feedback"
 * en "bug" / "enhancement" zijn standaard. */
const CATEGORY_LABEL: Record<Category, string> = {
  general: "feedback",
  bug: "bug",
  feature: "enhancement",
};

/** Title-prefix per categorie. "general" krijgt geen prefix. */
const CATEGORY_PREFIX: Record<Category, string> = {
  general: "",
  bug: "[Bug] ",
  feature: "[Feature] ",
};

const SENTIMENTS = [
  { id: "frustrated", emoji: "\u{1F61E}" },
  { id: "neutral", emoji: "\u{1F610}" },
  { id: "happy", emoji: "\u{1F60A}" },
] as const;
type Sentiment = (typeof SENTIMENTS)[number]["id"] | null;

const MAX_CHARS = 5000;
const MIN_MESSAGE_CHARS = 10;
const MIN_TITLE_CHARS = 5;

/** Build markdown-body voor de GitHub Issue. */
function buildIssueBody(args: {
  category: Category;
  message: string;
  sentiment: Sentiment;
  appVersion: string;
  userAgent: string;
}): string {
  const categoryLabel = ({
    general: "Algemeen",
    bug: "Bug",
    feature: "Functie",
  } as const)[args.category];

  const sentimentLabel = args.sentiment
    ? ({
        frustrated: "😞 Gefrustreerd",
        neutral: "😐 Neutraal",
        happy: "😊 Blij",
      } as const)[args.sentiment]
    : "—";

  return [
    "## Beschrijving",
    "",
    args.message.trim(),
    "",
    "---",
    "",
    `**Categorie:** ${categoryLabel}`,
    `**Sentiment:** ${sentimentLabel}`,
    `**App-versie:** ${args.appVersion || "onbekend"}`,
    `**User-Agent:** ${args.userAgent || "onbekend"}`,
    "",
    "<sub>Aangemaakt via Feedback-dialog in Open Heatloss Studio.</sub>",
  ].join("\n");
}

/** Bouw de pre-filled GitHub Issue URL. */
function buildIssueUrl(args: {
  category: Category;
  title: string;
  message: string;
  sentiment: Sentiment;
  appVersion: string;
  userAgent: string;
}): string {
  const fullTitle = CATEGORY_PREFIX[args.category] + args.title.trim();
  const body = buildIssueBody({
    category: args.category,
    message: args.message,
    sentiment: args.sentiment,
    appVersion: args.appVersion,
    userAgent: args.userAgent,
  });
  const params = new URLSearchParams({
    title: fullTitle,
    body,
    labels: CATEGORY_LABEL[args.category],
  });
  return `https://github.com/${GITHUB_REPO}/issues/new?${params.toString()}`;
}

/** Open een URL in de default browser. Tauri: via plugin-shell.
 *  Web: via window.open(). */
async function openUrl(url: string): Promise<void> {
  try {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } catch {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

interface FeedbackDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function FeedbackDialog({ open, onClose }: FeedbackDialogProps) {
  const { t } = useTranslation("feedback");
  const { t: tCommon } = useTranslation("common");

  const [title, setTitle] = useState("");
  const [category, setCategory] = useState<Category>("general");
  const [message, setMessage] = useState("");
  const [sentiment, setSentiment] = useState<Sentiment>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState("");
  const [appVersion, setAppVersion] = useState("");
  const [userAgent, setUserAgent] = useState("");

  // Reset state when dialog opens.
  useEffect(() => {
    if (open) {
      setTitle("");
      setCategory("general");
      setMessage("");
      setSentiment(null);
      setSubmitting(false);
      setSubmitted(false);
      setError("");
      getAppVersion().then((ver) => {
        setAppVersion(ver);
        buildUserAgent(ver).then(setUserAgent);
      });
    }
  }, [open]);

  const titleOk = title.trim().length >= MIN_TITLE_CHARS;
  const messageOk = message.trim().length >= MIN_MESSAGE_CHARS;
  const canSubmit = titleOk && messageOk && !submitting;
  const charCount = message.length;
  const charWarning = charCount >= MAX_CHARS - 500;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    setSubmitting(true);
    setError("");
    try {
      const url = buildIssueUrl({
        category,
        title,
        message,
        sentiment,
        appVersion,
        userAgent,
      });
      await openUrl(url);
      setSubmitted(true);
    } catch {
      setError(t("errorGeneric"));
    } finally {
      setSubmitting(false);
    }
  };

  const handleSendAnother = () => {
    setTitle("");
    setCategory("general");
    setMessage("");
    setSentiment(null);
    setSubmitted(false);
    setError("");
  };

  const footer = !submitted ? (
    <>
      <button className="feedback-btn feedback-btn-secondary" onClick={onClose}>
        {tCommon("cancel")}
      </button>
      <button
        className="feedback-btn feedback-btn-primary"
        onClick={handleSubmit}
        disabled={!canSubmit}
      >
        {submitting ? t("submitting") : t("submit")}
      </button>
    </>
  ) : undefined;

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t("title")}
      width={480}
      className="feedback-dialog"
      footer={footer}
    >
      {submitted ? (
        <div className="feedback-success">
          <div className="feedback-success-emoji">{"\u{2705}"}</div>
          <h3>{t("successTitle")}</h3>
          <p>{t("successMessage")}</p>
          <button
            className="feedback-btn feedback-btn-primary"
            onClick={handleSendAnother}
          >
            {t("sendAnother")}
          </button>
        </div>
      ) : (
        <div className="feedback-content">
          {/* Intro */}
          <p className="feedback-intro">{t("intro")}</p>

          {/* Categorie */}
          <div className="feedback-section">
            <div className="feedback-categories">
              {CATEGORIES.map((cat) => (
                <button
                  key={cat}
                  className={`feedback-category${
                    category === cat ? " active" : ""
                  }`}
                  onClick={() => setCategory(cat)}
                >
                  {t(`category${cat.charAt(0).toUpperCase() + cat.slice(1)}`)}
                </button>
              ))}
            </div>
          </div>

          {/* Titel */}
          <div className="feedback-section">
            <div className="feedback-field-row">
              <label className="feedback-field-label">
                {t("issueTitle")}{" "}
                <span className="feedback-required">*</span>
              </label>
              <input
                type="text"
                className="feedback-input"
                placeholder={t("issueTitlePlaceholder")}
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                maxLength={120}
              />
            </div>
          </div>

          {/* Bericht */}
          <div className="feedback-section">
            <textarea
              className="feedback-textarea"
              placeholder={t("messagePlaceholder")}
              value={message}
              onChange={(e) =>
                setMessage(e.target.value.slice(0, MAX_CHARS))
              }
              rows={6}
            />
            <div
              className={`feedback-char-count${
                charWarning ? " warning" : ""
              }`}
            >
              {charCount}/{MAX_CHARS}
            </div>
          </div>

          {/* Sentiment */}
          <div className="feedback-section">
            <div className="feedback-sentiment-label">{t("sentiment")}</div>
            <div className="feedback-sentiments">
              {SENTIMENTS.map((s) => (
                <button
                  key={s.id}
                  className={`feedback-sentiment${
                    sentiment === s.id ? " active" : ""
                  }`}
                  onClick={() =>
                    setSentiment(sentiment === s.id ? null : s.id)
                  }
                  title={t(
                    `sentiment${s.id.charAt(0).toUpperCase() + s.id.slice(1)}`,
                  )}
                >
                  <span className="feedback-sentiment-emoji">{s.emoji}</span>
                  <span className="feedback-sentiment-text">
                    {t(
                      `sentiment${s.id.charAt(0).toUpperCase() + s.id.slice(1)}`,
                    )}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {error && <div className="feedback-error">{error}</div>}
        </div>
      )}
    </Modal>
  );
}
