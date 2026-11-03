import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';


/**
 * UI-DS-01/UI-DS-04 guard: the token file is the only place allowed to hold
 * literal colours, and every `var(--token)` used by a component stylesheet
 * must be defined there. Mirrors scripts/check_ui_contracts.py so a broken
 * reference fails at unit-test speed as well.
 */

const SRC = join(process.cwd(), 'src');
const TOKEN_FILE = join(SRC, 'globals.css');

function cssFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name: string) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return cssFiles(path);
    return path.endsWith('.css') ? [path] : [];
  });
}

const files = cssFiles(SRC);
const tokenText = readFileSync(TOKEN_FILE, 'utf8');
const defined = new Set(
  [...tokenText.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]),
);

const LITERAL =
  /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?)\s*\(|:\s*(?:white|black|red|green|blue|gray|grey|silver|maroon|navy|teal|olive|purple|orange|yellow)\s*(?:!important\s*)?;/;

describe('design tokens', () => {
  it('defines the semantic aliases from DESIGN_SYSTEM §3.3', () => {
    for (const name of [
      '--surface-app',
      '--surface-panel',
      '--surface-sunken',
      '--border-default',
      '--text-primary',
      '--text-muted',
      '--accent-default',
      '--status-danger',
      '--status-warning',
      '--status-success',
      '--status-info',
    ]) {
      expect(defined.has(name), name).toBe(true);
    }
  });

  it('has no literal colours outside the token file', () => {
    const offenders = files
      .filter((path) => path !== TOKEN_FILE)
      .filter((path) => LITERAL.test(readFileSync(path, 'utf8')))
      .map((path) => relative(SRC, path));
    expect(offenders).toEqual([]);
  });

  it('defines every token referenced by a component stylesheet', () => {
    const missing = new Set<string>();
    for (const path of files) {
      const text = readFileSync(path, 'utf8');
      for (const match of text.matchAll(/var\((--[a-z0-9-]+)/g)) {
        if (!defined.has(match[1])) missing.add(`${relative(SRC, path)}: ${match[1]}`);
      }
    }
    expect([...missing]).toEqual([]);
  });


  it('keeps z-index values on the DESIGN_SYSTEM §7 scale', () => {
    const bands = [[0, 0], [10, 19], [20, 29], [30, 39], [40, 49], [60, 69], [90, 99]];
    const zTokens = [...tokenText.matchAll(/(--z-[a-z-]+)\s*:\s*(\d+);/g)];
    expect(zTokens.length).toBeGreaterThan(0);
    for (const [, name, raw] of zTokens) {
      const value = Number(raw);
      expect(bands.some(([low, high]) => value >= low && value <= high), `${name}=${value}`).toBe(true);
    }
    for (const path of files.filter((file) => file !== TOKEN_FILE)) {
      for (const [, raw] of readFileSync(path, 'utf8').matchAll(/z-index\s*:\s*([^;]+);/g)) {
        const value = raw.trim();
        const allowed = value.startsWith('var(--z-') || ['auto', 'inherit', 'initial', 'unset', 'revert'].includes(value);
        expect(allowed, `${relative(SRC, path)}: z-index ${value}`).toBe(true);
      }
    }
  });

  it('keeps the light theme a complete override of colour tokens', () => {
    const darkBlock = tokenText.match(/:root\s*\{([\s\S]*?)\n\}/);
    const lightBlock = tokenText.match(/:root\[data-theme='light'\]\s*\{([\s\S]*?)\n\}/);
    expect(darkBlock).not.toBeNull();
    expect(lightBlock).not.toBeNull();
    const pairs = [...(darkBlock?.[1] ?? '').matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)];
    const lightDefined = new Set(
      [...(lightBlock?.[1] ?? '').matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]),
    );
    const colourTokens = pairs
      .filter((m) => /^--(bg|border|text|accent|button|danger|success|warning|scrim|syntax|viewport)/.test(m[1]))
      .filter((m) => !m[2].includes('var('))
      .map((m) => m[1]);
    const missing = colourTokens.filter((name) => !lightDefined.has(name));
    expect(missing).toEqual([]);
  });
});
