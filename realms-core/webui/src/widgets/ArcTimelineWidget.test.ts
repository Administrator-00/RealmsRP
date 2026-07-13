import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ArcTimelineWidget from './ArcTimelineWidget.vue';
import type { ArcTimelineNode } from '../types';

const sample: ArcTimelineNode[] = [
  { worldTime: '三月一日', dimension: 'trust_user', value: 20, label: '初识', summary: '' },
  { worldTime: '三月三日', dimension: 'trust_user', value: 55, label: '朋友', summary: '' },
];

describe('ArcTimelineWidget', () => {
  it('renders timeline nodes', () => {
    const w = mount(ArcTimelineWidget, { props: { timeline: sample } });
    expect(w.text()).toContain('初识');
    expect(w.text()).toContain('朋友');
  });

  it('filters by dimension', () => {
    const w = mount(ArcTimelineWidget, { props: { timeline: sample, dimension: 'trust_user' } });
    expect(w.text()).toContain('信任');
  });

  it('shows empty state', () => {
    const w = mount(ArcTimelineWidget, { props: { timeline: [] } });
    expect(w.text()).toContain('无弧光数据');
  });
});
