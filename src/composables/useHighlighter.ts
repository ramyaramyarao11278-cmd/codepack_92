import { ref, shallowRef } from "vue";
import { createHighlighter, type Highlighter } from "shiki";

const highlighterRef = shallowRef<Highlighter | null>(null);
const loading = ref(false);
const loadedLangs = new Set<string>();

const THEME = "vitesse-dark";

// Map file extensions to shiki language IDs
const EXT_TO_LANG: Record<string, string> = {
  ts: "typescript", tsx: "tsx", js: "javascript", jsx: "jsx",
  vue: "vue", rs: "rust", py: "python", rb: "ruby", go: "go",
  java: "java", kt: "kotlin", swift: "swift", dart: "dart",
  c: "c", cpp: "cpp", h: "c", hpp: "cpp",
  cs: "csharp", php: "php", sh: "bash", bash: "bash", zsh: "bash",
  bat: "bat", ps1: "powershell",
  html: "html", css: "css", scss: "scss", less: "less",
  json: "json", yaml: "yaml", yml: "yaml", toml: "toml",
  xml: "xml", svg: "xml",
  md: "markdown", sql: "sql",
  dockerfile: "dockerfile", docker: "dockerfile",
  graphql: "graphql", gql: "graphql",
  lua: "lua", r: "r", perl: "perl",
  makefile: "makefile", cmake: "cmake",
  gradle: "groovy", groovy: "groovy",
};

function getLangFromPath(filePath: string): string {
  const name = filePath.replace(/\\/g, "/").split("/").pop() || "";
  const lower = name.toLowerCase();
  // Special filenames
  if (lower === "dockerfile") return "dockerfile";
  if (lower === "makefile" || lower === "gnumakefile") return "makefile";
  if (lower === "cmakelists.txt") return "cmake";
  const ext = lower.split(".").pop() || "";
  return EXT_TO_LANG[ext] || "";
}

async function ensureHighlighter(): Promise<Highlighter> {
  if (highlighterRef.value) return highlighterRef.value;
  loading.value = true;
  try {
    const h = await createHighlighter({ themes: [THEME], langs: [] });
    highlighterRef.value = h;
    return h;
  } finally {
    loading.value = false;
  }
}

async function ensureLang(h: Highlighter, lang: string): Promise<void> {
  if (!lang || loadedLangs.has(lang)) return;
  try {
    await h.loadLanguage(lang as any);
    loadedLangs.add(lang);
  } catch {
    // Language not available in shiki, will fall back to plain text
  }
}

export function useHighlighter() {
  const highlightedHtml = ref("");
  const isHighlighting = ref(false);

  async function highlight(code: string, filePath: string): Promise<string> {
    if (!code) { highlightedHtml.value = ""; return ""; }
    isHighlighting.value = true;
    try {
      const h = await ensureHighlighter();
      const lang = getLangFromPath(filePath);
      if (lang) {
        await ensureLang(h, lang);
      }
      const effectiveLang = lang && loadedLangs.has(lang) ? lang : "text";
      // Load text lang if needed
      if (effectiveLang === "text" && !loadedLangs.has("text")) {
        try {
          await h.loadLanguage("text" as any);
          loadedLangs.add("text");
        } catch { /* ignore */ }
      }
      const html = h.codeToHtml(code, {
        lang: loadedLangs.has(effectiveLang) ? effectiveLang : "text",
        theme: THEME,
      });
      highlightedHtml.value = html;
      return html;
    } catch {
      highlightedHtml.value = "";
      return "";
    } finally {
      isHighlighting.value = false;
    }
  }

  return { highlightedHtml, isHighlighting, highlight, loading };
}
