import { describe, expect, it } from 'vitest';
import en from './en.json';
import zhCN from './zh-CN.json';
import ja from './ja.json';
import ko from './ko.json';

type JsonTree = Record<string, unknown>;

function flattenKeys(tree: JsonTree, prefix = ''): string[] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === 'object' && value !== null
      ? flattenKeys(value as JsonTree, `${prefix}${key}.`)
      : [`${prefix}${key}`]
  );
}

describe('locale files', () => {
  it('keeps all locales key-par with en', () => {
    const enKeys = flattenKeys(en).sort();
    expect(flattenKeys(zhCN).sort()).toEqual(enKeys);
    expect(flattenKeys(ja).sort()).toEqual(enKeys);
    expect(flattenKeys(ko).sort()).toEqual(enKeys);
  });
});
