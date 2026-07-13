import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import AffinityWidget from './AffinityWidget.vue';
import type { NpcRelationship } from '../types';

const sample: NpcRelationship[] = [
  { cycleId: 'c1', npcId: 'lin_yueru', name: '林月如', affinity: 60, trust: 80, intimacy: 70, currentType: 'close_friend' },
  { cycleId: 'c1', npcId: 'li_xiaoyao', name: '李逍遥', affinity: 0, trust: 0, intimacy: 0, currentType: 'stranger' },
  { cycleId: 'c1', npcId: 'zhao_linger', name: '赵灵儿', affinity: -50, trust: 10, intimacy: 0, currentType: 'enemy' },
];

describe('AffinityWidget', () => {
  it('renders all NPCs', () => {
    const w = mount(AffinityWidget, { props: { relationships: sample } });
    expect(w.text()).toContain('林月如');
    expect(w.text()).toContain('李逍遥');
    expect(w.text()).toContain('赵灵儿');
  });

  it('displays affinity values', () => {
    const w = mount(AffinityWidget, { props: { relationships: sample } });
    expect(w.text()).toContain('+60');
    expect(w.text()).toContain('-50');
  });

  it('shows empty state', () => {
    const w = mount(AffinityWidget, { props: { relationships: [] } });
    expect(w.text()).toContain('暂无关系');
  });

  it('displays relationship types', () => {
    const w = mount(AffinityWidget, { props: { relationships: sample } });
    expect(w.text()).toContain('close_friend');
    expect(w.text()).toContain('enemy');
  });
});
