// i18n dictionaries registry.
// en.ts is the source of truth; every other dictionary must contain the
// exact same keys (enforced by `satisfies Record<TranslationKey, string>`).

import { en, LOCALES, type TranslationKey, type LocaleCode } from './en';
import { de } from './de';
import { es } from './es';
import { fr } from './fr';
import { ptBR } from './pt-BR';
import { zh } from './zh';

export { en, de, es, fr, ptBR, zh, LOCALES };
export type { TranslationKey, LocaleCode };

export type Dictionary = Record<TranslationKey, string>;

export const dictionaries: Record<LocaleCode, Dictionary> = {
  en,
  de,
  es,
  fr,
  'pt-BR': ptBR,
  zh,
};
