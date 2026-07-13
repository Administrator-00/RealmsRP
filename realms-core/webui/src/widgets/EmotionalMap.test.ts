import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import EmotionalMap from './EmotionalMap.vue';
import type { EmotionalNode } from '../types';

const sample: EmotionalNode[] = [
  { npcId: 'ly', npcName: '林月如', emotionType: 'love', intensity: 0.7 },
  { npcId: 'xl', npcName: '李逍遥', emotionType: 'respect', intensity: 0.3 },
];

describe('EmotionalMap', () => {
  it('renders emotion nodes', () => {
    const w = mount(EmotionalMap, { props: { nodes: sample } });
    expect(w.text()).toContain('林月如');
    expect(w.text()).toContain('李逍遥');
    expect(w.text()).toContain('爱慕');
  });

  it('shows empty state', () => {
    const w = mount(EmotionalMap, { props: { nodes: [] } });
    expect(w.text()).toContain('无情感数据');
  });
});
