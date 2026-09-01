import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { ReactNode } from "react";
import { en } from "./en";
import type { TranslationKey } from "./en";
import { zhTW } from "./zh-TW";

export type Locale = "en" | "zh-TW";

export const LOCALES: { code: Locale; label: string }[] = [
  { code: "en", label: "EN" },
  { code: "zh-TW", label: "中文" },
];

const DICTIONARIES: Record<Locale, Record<TranslationKey, string>> = {
  en,
  "zh-TW": zhTW,
};

const STORAGE_KEY = "locale";

function initialLocale(): Locale {
  const saved = window.localStorage.getItem(STORAGE_KEY);
  if (saved === "en" || saved === "zh-TW") {
    return saved;
  }
  return window.navigator.language.toLowerCase().startsWith("zh")
    ? "zh-TW"
    : "en";
}

type Vars = Record<string, string | number>;

interface I18n {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  /** Translate a key, filling any `{name}` placeholders from `vars`. */
  t: (key: TranslationKey, vars?: Vars) => string;
}

const I18nContext = createContext<I18n | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(initialLocale);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const setLocale = useCallback((next: Locale) => {
    window.localStorage.setItem(STORAGE_KEY, next);
    setLocaleState(next);
  }, []);

  const t = useCallback(
    (key: TranslationKey, vars?: Vars) => {
      const template = DICTIONARIES[locale][key] ?? en[key];
      if (!vars) {
        return template;
      }
      return template.replace(/\{(\w+)\}/g, (_, name: string) =>
        name in vars ? String(vars[name]) : `{${name}}`,
      );
    },
    [locale],
  );

  const value = useMemo<I18n>(() => ({ locale, setLocale, t }), [
    locale,
    setLocale,
    t,
  ]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18n {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return ctx;
}
