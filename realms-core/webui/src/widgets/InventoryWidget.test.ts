import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import InventoryWidget from './InventoryWidget.vue';
import type { InventoryItem } from '../types';

const sample: InventoryItem[] = [
  { id: '1', name: '长剑', type: 'weapon', quantity: 1 },
  { id: '2', name: '金疮药', type: 'potion', quantity: 3 },
];

describe('InventoryWidget', () => {
  it('renders items', () => {
    const w = mount(InventoryWidget, { props: { items: sample } });
    expect(w.text()).toContain('长剑');
    expect(w.text()).toContain('金疮药');
  });

  it('shows quantity for stacked items', () => {
    const w = mount(InventoryWidget, { props: { items: sample } });
    expect(w.text()).toContain('×3');
  });

  it('shows empty state', () => {
    const w = mount(InventoryWidget, { props: { items: [] } });
    expect(w.text()).toContain('背包空空');
  });

  it('groups by type', () => {
    const w = mount(InventoryWidget, { props: { items: sample } });
    expect(w.text()).toContain('武器');
    expect(w.text()).toContain('药品');
  });
});
