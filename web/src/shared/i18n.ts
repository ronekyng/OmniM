// i18n kernel — I18N_L10N contract (docs/specification/ui/I18N_L10N.zh-CN.md):
// §2 词典真值 (`i18n/<locale>.ts` + `t()`/`useI18n()`），§3 语言选择与回退
// （默认 zh-CN、en-US 回退、立即切换不重载、`omnim.locale` 持久化、
// `<html lang>` 同步），§3 格式随切换重渲染，§4 ICU 复数（Intl.PluralRules）。
import { useSyncExternalStore } from 'react';
import { zhCN, type MessageKey } from './i18n/zh-CN';
import { enUS } from './i18n/en-US';

export type Locale = 'zh-CN' | 'en-US';

const SUPPORTED: Locale[] = ['zh-CN', 'en-US'];
const FALLBACK: Locale = 'en-US';
const DEFAULT: Locale = 'zh-CN';
const STORAGE_KEY = 'omnim.locale';

const dictionaries: Record<Locale, Record<string, string>> = {
  'zh-CN': zhCN as Record<string, string>,
  'en-US': enUS as Record<string, string>,
};

type SubKey = string;

// ———————————————————————————— 外部 store ————————————————————————————
let listeners = new Set<() => void>();
let current: Locale | null = null; // lazy initialise（读 storage/浏览器）

function isLocale(v: unknown): v is Locale {
  return typeof v === 'string' && (SUPPORTED as string[]).includes(v);
}

/** §3 检测顺序：用户已存选择 → navigator.languages 交集 → zh-CN。 */
export function detectLocale(): Locale {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (isLocale(stored)) return stored;
  } catch {
    /* private mode etc. — fall through */
  }
  const langs =
    typeof navigator !== 'undefined' ? navigator.languages ?? [navigator.language] : [];
  for (const lang of langs) {
    if (!lang) continue;
    if (isLocale(lang)) return lang;
    // Same primary subtag wins ('zh-TW' → 'zh-CN', 'en-GB' → 'en-US').
    const primary = lang.split('-')[0];
    const prefixed = SUPPORTED.find((s) => s.split('-')[0] === primary);
    if (prefixed) return prefixed;
  }
  return DEFAULT;
}

function locale(): Locale {
  if (current === null) {
    current = detectLocale();
    // §3 文档属性：初始检测同样同步 `<html lang>`（切换时不重载）。
    if (typeof document !== 'undefined' && document.documentElement) {
      document.documentElement.lang = current;
    }
  }
  return current;
}

function emit() {
  for (const fn of listeners) fn();
}

function subscribe(fn: () => void) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

function getSnapshot(): Locale {
  return locale();
}

/** §3 切换：立即生效、不重载、存 `omnim.locale`、更新 `<html lang>`。 */
export function setLocale(next: Locale) {
  if (!isLocale(next) || next === locale()) return;
  current = next;
  try {
    window.localStorage.setItem(STORAGE_KEY, next);
  } catch {
    /* storage unavailable — switch still applies in-session */
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = next;
  }
  emit();
}

export function supportedLocales(): Locale[] {
  return [...SUPPORTED];
}

// ———————————————————————————— 插值 ————————————————————————————
/** `{count}` → value（§4.4：动态文本走占位符，绝不做字符串运算）。 */
function interpolate(template: string, vars: Record<string, string | number> | undefined): string {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (whole, name: string) =>
    Object.prototype.hasOwnProperty.call(vars, name) ? String(vars[name]) : whole,
  );
}

/** Lookup with §3 回退链：当前词典 → en-US → 开发环境键名（生产空 + 诊断日志）。 */
export function t(key: MessageKey, vars?: Record<string, string | number>): string {
  const loc = locale();
  const dict = dictionaries[loc];
  const hit = dict[key] ?? dictionaries[FALLBACK][key];
  if (hit !== undefined) return interpolate(hit, vars);
  // 缺失于两种词典：开发环境直接渲染键名（§3"开发环境…"）；生产渲染 en-US
  // 值（同样缺失 → 空串）并给出诊断日志。
  if (import.meta.env?.DEV) return key;
  console.warn(`[i18n] missing message key: ${key} (${loc})`);
  return '';
}

/** §4 复数：`Intl.PluralRules` 选择类别（禁止手写复数）。键 `….<category>`。 */
export function tPlural(
  // 复数键是“基键 + 类别后缀”，不属于词典扁平键集合本身。
  base: string,
  count: number,
  vars?: Record<string, string | number>,
): string {
  const loc = locale();
  const rest = vars && 'count' in vars ? { ...vars, count } : { count, ...vars };
  const category = Intl.PluralRules.supportedLocalesOf([loc]).length > 0
    ? new Intl.PluralRules(loc).select(count)
    : 'other';
  const tryKey = (cat: string) =>
    dictionaries[loc][`${base}.${cat}`] ?? dictionaries[FALLBACK][`${base}.${cat}`];
  const hit = tryKey(category) ?? tryKey('other');
  if (hit === undefined) {
    if (import.meta.env?.DEV) return `${base}.${category}`;
    console.warn(`[i18n] missing plural key: ${base} (${loc}/${category})`);
    return '';
  }
  return interpolate(hit, rest);
}

// ———————————————————————————— 格式化（§5） ————————————————————————————
/** 坐标：3 位小数（存储值不变，仅显示）。 */
export function formatCoordinate(value: number, localeOverride?: Locale): string {
  return new Intl.NumberFormat(localeOverride ?? locale(), {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3,
  }).format(value);
}

/** 大数字：按语言环境分组。 */
export function formatCount(value: number, localeOverride?: Locale): string {
  return new Intl.NumberFormat(localeOverride ?? locale()).format(value);
}

/** 日期：契约 §5 固定件序——zh `YYYY/MM/DD` · en `MM/DD/YYYY`（timeStyle 不用，
 * 部分运行时短年份会缩成 2 位）。 */
export function formatDate(date: Date | number, localeOverride?: Locale): string {
  const loc = localeOverride ?? locale();
  const parts = new Intl.DateTimeFormat('en-CA', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(date);
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? '';
  const y = get('year');
  const m = get('month');
  const d = get('day');
  return loc === 'zh-CN' ? `${y}/${m}/${d}` : `${m}/${d}/${y}`;
}

/** 时间：zh 24 小时制；en 按语言默认（2:05:09 PM）。 */
export function formatTime(date: Date | number, localeOverride?: Locale): string {
  return new Intl.DateTimeFormat(localeOverride ?? locale(), {
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: (localeOverride ?? locale()) !== 'zh-CN',
  }).format(date);
}

/** 角度：度符号，1 位小数（45.0°）。 */
export function formatAngle(degrees: number, localeOverride?: Locale): string {
  return `${new Intl.NumberFormat(localeOverride ?? locale(), {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(degrees)}\u00B0`;
}

// ———————————————————————————— React hook ————————————————————————————
/** `useI18n()`：语言切换时整棵使用树重渲染（外部 store 订阅）。 */
export function useI18n() {
  const loc = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  return {
    locale: loc,
    setLocale,
    t: (key: MessageKey, vars?: Record<string, string | number>) => t(key, vars),
  };
}
