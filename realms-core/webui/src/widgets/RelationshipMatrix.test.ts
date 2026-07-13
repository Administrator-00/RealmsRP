import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import RelationshipMatrix from './RelationshipMatrix.vue';

const sample = [
  { npcId: 'ly', npcName: '林月如', affinity: 60, trust: 80, intimacy: 70, currentType: 'close_friend' },
  { npcId: 'xl', npcName: '李逍遥', affinity: -30, trust: 15, intimacy: 5, currentType: 'enemy' },
];

describe('RelationshipMatrix', () => {
  it('renders NPC cards', () => {
    const w = mount(RelationshipMatrix, { props: { cycleName: '周目1', rows: sample } });
    expect(w.text()).toContain('林月如');
    expect(w.text()).toContain('李逍遥');
    expect(w.text()).toContain('+60');
    expect(w.text()).toContain('-30');
  });

  it('shows empty state', () => {
    const w = mount(RelationshipMatrix, { props: { cycleName: 'empty', rows: [] } });
    expect(w.text()).toContain('暂无关系数据');
  });

  it('shows cycle name', () => {
    const w = mount(RelationshipMatrix, { props: { cycleName: '周目1', rows: sample } });
    expect(w.text()).toContain('周目1');
  });
});
