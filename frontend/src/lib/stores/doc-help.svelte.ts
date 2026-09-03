// doc-help.svelte — in-app help: fetch a docs section, render it in a modal,
// cache per session. Also owns the "first-visit coachmark" state for help
// spots (roadmap arena, requests answer form, search advanced filters).

// ─── Doc section loading ──────────────────────────────────────────────
interface DocSection {
  html: string;
  page: string;
  anchor: string | null;
  title: string;
}

const sectionCache = new Map<string, DocSection>();

let active = $state<DocSection | null>(null);
let loading = $state(false);
let error = $state('');

// Parse a single section (heading + siblings until next same-or-higher
// heading) out of an mdBook HTML page.
function extractSection(doc: Document, anchor: string | null): string {
  if (!anchor) {
    // whole page: main content
    const main = doc.querySelector('main');
    return main ? main.innerHTML : doc.body.innerHTML;
  }
  const heading = doc.querySelector(`[id="${CSS.escape(anchor)}"]`);
  if (!heading) return '';
  const out: HTMLElement[] = [heading as HTMLElement];
  let el = (heading as HTMLElement).nextElementSibling;
  const level = parseInt((heading as HTMLElement).tagName[1] ?? '2', 10);
  while (el) {
    const tag = el.tagName.toLowerCase();
    if (/^h[1-6]$/.test(tag)) {
      const l = parseInt(tag[1], 10);
      if (l <= level) break; // next same-or-higher heading ends the section
    }
    out.push(el as HTMLElement);
    el = el.nextElementSibling;
  }
  return out.map((e) => e.outerHTML).join('\n');
}

export async function loadDocSection(slug: string): Promise<DocSection | null> {
  if (sectionCache.has(slug)) {
    active = sectionCache.get(slug)!;
    return active;
  }
  const [pagePart, anchor] = slug.split('#');
  const page = pagePart.endsWith('.html') ? pagePart : `${pagePart}.html`;
  loading = true;
  error = '';
  try {
    const res = await fetch(`/docs/${page}`);
    if (!res.ok) throw new Error(`Docs fetch failed (HTTP ${res.status})`);
    const dom = new DOMParser().parseFromString(await res.text(), 'text/html');
    const sectionHtml = extractSection(dom, anchor ?? null);
    if (!sectionHtml) throw new Error(`Section not found: ${slug}`);
    const title = anchor
      ? (dom.querySelector(`[id="${CSS.escape(anchor)}"]`)?.textContent?.trim() ?? pagePart)
      : (dom.querySelector('title')?.textContent?.replace(' - FicNexus Docs', '').trim() ?? pagePart);
    const sec: DocSection = { html: sectionHtml, page, anchor, title };
    sectionCache.set(slug, sec);
    active = sec;
    return sec;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    return null;
  } finally {
    loading = false;
  }
}

export function openDoc(slug: string) {
  void loadDocSection(slug);
}

export function closeDoc() {
  active = null;
}

// Single reactive state object for components to read ($state proxies).
export const docHelpState = {
  get active() { return active; },
  get loading() { return loading; },
  get error() { return error; },
};

// ─── Coachmarks (first-visit help, localStorage-dismissed) ─────────────
const COACHMARK_KEY = 'fichub_coachmarks_v1';

export function coachmarkSeen(name: string): boolean {
  try {
    const raw = localStorage.getItem(COACHMARK_KEY);
    if (!raw) return false;
    const seen = JSON.parse(raw) as string[];
    return seen.includes(name);
  } catch {
    return false;
  }
}

export function markCoachmarkSeen(name: string) {
  try {
    const raw = localStorage.getItem(COACHMARK_KEY);
    const seen: string[] = raw ? (JSON.parse(raw) as string[]) : [];
    if (!seen.includes(name)) {
      seen.push(name);
      localStorage.setItem(COACHMARK_KEY, JSON.stringify(seen));
    }
  } catch {
    /* storage unavailable — non-fatal */
  }
}

// Re-export for type usage elsewhere (kept tiny on purpose).
export type { DocSection };
