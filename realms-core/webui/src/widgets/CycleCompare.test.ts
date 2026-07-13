import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import CycleCompare from './CycleCompare.vue';

const sample = [
  { cycleId: 'c1', cycleName: '周目1', npcId: 'ly', affinity: 60, trust: 80, intimacy: 70, currentType: 'close_friend', initialTemplate: 'tpl_childhood_friend' },
  { cycleId: 'c2', cycleName: '周目2', npcId: 'ly', affinity: -30, trust: 20, intimacy: 10, currentType: 'enemy', initialTemplate: 'tpl_stranger' },
];

describe('CycleCompare', () => {
  it('renders table rows', () => {
    const w = mount(CycleCompare, { props: { npcName: '林月如', comparisons: sample } });
    expect(w.text()).toContain('周目1');
    expect(w.text()).toContain('周目2');
    expect(w.text()).toContain('+60');
    expect(w.text()).toContain('-30');
  });

  it('shows empty state', () => {
    const w = mount(CycleCompare, { props: { npcName: 'NPC', comparisons: [] } });
    expect(w.text()).toContain('无数据');
  });

  it('shows relationship types', () => {
    const w = mount(CycleCompare, { props: { npcName: 'NPC', comparisons: sample } });
    expect(w.text()).toContain('close_friend');
    expect(w.text()).toContain('enemy');
  });
});
