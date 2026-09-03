// Theme design token presets.
// Each preset maps to CSS custom properties on :root.

export interface ThemeTokens {
  /** Human-readable name shown in the picker. */
  name: string;
  /** Accent / primary colour. */
  accent: string;
  /** Page background. */
  bg: string;
  /** Card / surface background. */
  surface: string;
  /** Primary text colour. */
  text: string;
  /** Muted / secondary text. */
  muted: string;
  /** Corner radius for cards and inputs. */
  radius: string;
  /** UI font family. */
  font: string;
  /** Spacing density scale. */
  density: 'compact' | 'comfortable' | 'spacious';
  /** Font family used in the reader view. */
  readerFont: string;
  /** Max width of the reader column. */
  readerWidth: string;
  /** Line height inside the reader. */
  readerLineHeight: string;
}

export const defaultDark: ThemeTokens = {
  name: 'Default Dark',
  accent: '#6366f1',
  bg: '#0f1117',
  surface: '#171a23',
  text: '#e8eaf0',
  muted: '#9aa3b2',
  radius: '10px',
  font: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
  density: 'comfortable',
  readerFont: 'Georgia',
  readerWidth: '700px',
  readerLineHeight: '1.8',
};

export const defaultLight: ThemeTokens = {
  name: 'Default Light',
  accent: '#6366f1',
  bg: '#f8f9fc',
  surface: '#ffffff',
  text: '#1a1d27',
  muted: '#6b7280',
  radius: '10px',
  font: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
  density: 'comfortable',
  readerFont: 'Georgia',
  readerWidth: '700px',
  readerLineHeight: '1.8',
};

export const highContrast: ThemeTokens = {
  name: 'High Contrast',
  accent: '#0055ff',
  bg: '#000000',
  surface: '#0a0a0a',
  text: '#ffffff',
  muted: '#cccccc',
  radius: '4px',
  font: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
  density: 'comfortable',
  readerFont: 'Arial',
  readerWidth: '650px',
  readerLineHeight: '1.9',
};

export const sepia: ThemeTokens = {
  name: 'Sepia',
  accent: '#a0522d',
  bg: '#f4ecd8',
  surface: '#faf6ee',
  text: '#3e2723',
  muted: '#8d6e63',
  radius: '8px',
  font: "Georgia, 'Times New Roman', serif",
  density: 'comfortable',
  readerFont: 'Georgia',
  readerWidth: '680px',
  readerLineHeight: '1.9',
};

export const dyslexia: ThemeTokens = {
  name: 'Dyslexia Friendly',
  accent: '#4a6cf7',
  bg: '#fefefe',
  surface: '#ffffff',
  text: '#1a1a1a',
  muted: '#666666',
  radius: '12px',
  font: "OpenDyslexic, 'Comic Sans MS', Verdana, sans-serif",
  density: 'spacious',
  readerFont: "OpenDyslexic, 'Comic Sans MS', Verdana, sans-serif",
  readerWidth: '600px',
  readerLineHeight: '2.0',
};

export const archiveClassic: ThemeTokens = {
  name: 'Archive Classic',
  accent: '#990000',
  bg: '#ffffff',
  surface: '#f5f5f5',
  text: '#2a2a2a',
  muted: '#666666',
  radius: '2px',
  font: "Georgia, 'Times New Roman', serif",
  density: 'compact',
  readerFont: "Georgia, 'Times New Roman', serif",
  readerWidth: '680px',
  readerLineHeight: '1.8',
};

export const archiveNoir: ThemeTokens = {
  name: 'Archive Noir',
  accent: '#990000',
  bg: '#1e1e1e',
  surface: '#262626',
  text: '#e0e0e0',
  muted: '#999999',
  radius: '2px',
  font: "Georgia, 'Times New Roman', serif",
  density: 'compact',
  readerFont: "Georgia, 'Times New Roman', serif",
  readerWidth: '680px',
  readerLineHeight: '1.8',
};

/** All built-in presets indexed by slug. */
export const presets: Record<string, ThemeTokens> = {
  'default-dark': defaultDark,
  'default-light': defaultLight,
  'high-contrast': highContrast,
  'sepia': sepia,
  'dyslexia': dyslexia,
  'archive-classic': archiveClassic,
  'archive-noir': archiveNoir,
};
