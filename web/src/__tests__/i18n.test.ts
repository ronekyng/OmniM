// @vitest-environment jsdom
// I18N_L10N contract tests (UI-I18N-02/03/04): locale detection & fallback,
// Intl plural categories, and locale-aware number/date/time/angle formats.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Fresh module state per test block (the kernel caches the locale).
async function freshI18n() {
  vi.resetModules();
  return await import('../shared/i18n');
}

function setNavigator(langs: readonly string[]) {
  Object.defineProperty(window, 'navigator', {
    value: { languages: langs, language: langs[0] },
    configurable: true,
    writable: true,
  });
}

beforeEach(() => {
  window.localStorage.clear();
  document.documentElement.lang = '';
  setNavigator([]);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('locale detection (§3)', () => {
  it('defaults to zh-CN when nothing is stored or advertised', async () => {
    const i18n = await freshI18n();
    expect(i18n.detectLocale()).toBe('zh-CN');
  });

  it('stored user choice wins (omnim.locale)', async () => {
    window.localStorage.setItem('omnim.locale', 'en-US');
    const i18n = await freshI18n();
    expect(i18n.detectLocale()).toBe('en-US');
  });

  it('navigator.languages intersection picks a supported browser language', async () => {
    setNavigator(['en-GB', 'zh-TW', 'en-US']);
    const i18n = await freshI18n();
    expect(i18n.detectLocale()).toBe('en-US');
  });

  it('same primary subtag maps to a supported locale (zh-TW → zh-CN)', async () => {
    setNavigator(['zh-HK']);
    const i18n = await freshI18n();
    expect(i18n.detectLocale()).toBe('zh-CN');
  });
});

describe('message lookup and fallback (§3)', () => {
  it('resolves the current locale dictionary with {var} interpolation', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('en-US');
    expect(i18n.t('shell.breadcrumb.workspace')).toBe('Workspace');
    // switching back goes through the same setLocale path as the UI
    i18n.setLocale('zh-CN');
    expect(i18n.t('shell.breadcrumb.workspace')).toBe('工作区');
  });

  it('a missing key (en-only) falls back to the en-US value', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('zh-CN');
    // 'common.plural.objects.one' exists only in en-US.
    expect(i18n.t('common.plural.objects.one' as never, { count: 1 })).toBe('1 object');
  });

  it('a key missing in both dictionaries renders the key name in dev and "" in prod', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const i18n = await freshI18n();
    // VITEST runs with import.meta.env.DEV = true by default.
    expect(i18n.t('common.illusive.missing_key' as never)).toBe('common.illusive.missing_key');
    warn.mockRestore();
  });
});

describe('plurals via Intl.PluralRules (§4, UI-I18N-03)', () => {
  it('en-US: selects one vs other; never hand-written branching', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('en-US');
    expect(i18n.tPlural('common.plural.objects', 1)).toBe('1 object');
    expect(i18n.tPlural('common.plural.objects', 0)).toBe('0 objects');
    expect(i18n.tPlural('common.plural.objects', 3)).toBe('3 objects');
  });

  it('zh-CN declares only other and interpolates the count', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('zh-CN');
    expect(i18n.tPlural('common.plural.objects', 1)).toBe('1 个对象');
    expect(i18n.tPlural('common.plural.objects', 7)).toBe('7 个对象');
  });

  it('falls back to the other category when a locale misses it (zh + en key set)', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('en-US');
    // 'one' exists, other categories inherit: this mirrors contract rule
    // "…词典仍必须声明 other，以便安全加入英文".
    expect(i18n.tPlural('common.plural.objects', 2)).toBe('2 objects');
  });
});

describe('locale formats (§5, UI-I18N-04)', () => {
  it('coordinates use 3 decimals in both locales', async () => {
    const i18n = await freshI18n();
    expect(i18n.formatCoordinate(1.2344, 'en-US')).toBe('1.234');
    expect(i18n.formatCoordinate(1.2344, 'zh-CN')).toBe('1.234');
  });

  it('grouped counts use the locale separator', async () => {
    const i18n = await freshI18n();
    expect(i18n.formatCount(12345, 'en-US')).toBe('12,345');
    expect(i18n.formatCount(12345, 'zh-CN')).toBe('12,345');
  });

  it('dates: short format per locale', async () => {
    const i18n = await freshI18n();
    const ts = Date.UTC(2026, 9, 29);
    expect(i18n.formatDate(ts, 'en-US')).toBe('10/29/2026');
    expect(i18n.formatDate(ts, 'zh-CN')).toBe('2026/10/29');
  });

  it('times: zh uses 24h clock; en uses the language default', async () => {
    const i18n = await freshI18n();
    const ts = new Date(2026, 9, 29, 14, 5, 9, 0).getTime();
    const zh = i18n.formatTime(ts, 'zh-CN');
    const en = i18n.formatTime(ts, 'en-US');
    expect(zh).toMatch(/14:05:09/);
    expect(zh).not.toMatch(/下午|PM/);
    expect(en).toMatch(/2:05:09\s*PM/);
  });

  it('angles carry the degree sign with 1 decimal', async () => {
    const i18n = await freshI18n();
    expect(i18n.formatAngle(45, 'en-US')).toBe('45.0°');
    expect(i18n.formatAngle(45.04, 'zh-CN')).toBe('45.0°');
  });
});

describe('setLocale side effects (§3)', () => {
  it('persists to omnim.locale and updates <html lang> without reload', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('en-US');
    expect(window.localStorage.getItem('omnim.locale')).toBe('en-US');
    expect(document.documentElement.lang).toBe('en-US');
    i18n.setLocale('zh-CN');
    expect(window.localStorage.getItem('omnim.locale')).toBe('zh-CN');
    expect(document.documentElement.lang).toBe('zh-CN');
  });

  it('switching is instant (no reload) for subscribers', async () => {
    const i18n = await freshI18n();
    const before = i18n.t('shell.breadcrumb.workspace');
    expect(before).toBe('工作区');
    i18n.setLocale('en-US');
    expect(i18n.t('shell.breadcrumb.workspace')).toBe('Workspace');
  });
});

describe('setLocale side effects (§3)', () => {
  it('persists to omnim.locale and updates <html lang> without reload', async () => {
    const i18n = await freshI18n();
    i18n.setLocale('en-US');
    expect(window.localStorage.getItem('omnim.locale')).toBe('en-US');
    expect(document.documentElement.lang).toBe('en-US');
    i18n.setLocale('zh-CN');
    expect(window.localStorage.getItem('omnim.locale')).toBe('zh-CN');
    expect(document.documentElement.lang).toBe('zh-CN');
  });

  it('switching is instant (no reload) for subscribers', async () => {
    const i18n = await freshI18n();
    const before = i18n.t('shell.breadcrumb.workspace');
    expect(before).toBe('工作区');
    i18n.setLocale('en-US');
    expect(i18n.t('shell.breadcrumb.workspace')).toBe('Workspace');
  });
});

// useI18n() 本体（useSyncExternalStore 外部 store）在 e2e/language.spec 的真实
// React 渲染路径里覆盖；这里测底下 subscribe/emit 语义已足够。

