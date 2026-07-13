import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import CycleList from './CycleList.vue';

const cycles = [
  { cycleId: 'c1', cycleName: '周目1', worldId: 'w1', perspectiveName: '李逍遥', gmName: '林月如文风', totalEvents: 12, lastPlayedAt: '2026-07-13' },
  { cycleId: 'c2', cycleName: '周目2', worldId: 'w1', perspectiveName: '师兄', gmName: '古龙武侠', totalEvents: 5 },
];

describe('CycleList', () => {
  it('renders cycles', () => {
    const w = mount(CycleList, { props: { cycles, worldName: '仙剑一' } });
    expect(w.text()).toContain('周目1');
    expect(w.text()).toContain('周目2');
    expect(w.text()).toContain('李逍遥');
  });

  it('emits select', async () => {
    const w = mount(CycleList, { props: { cycles, worldName: '仙剑一' } });
    await w.findAll('.cl-btn-primary')[0].trigger('click');
    expect(w.emitted('select')?.[0]).toEqual(['c1']);
  });

  it('emits duplicate', async () => {
    const w = mount(CycleList, { props: { cycles, worldName: '仙剑一' } });
    // 三个按钮: continue (.cl-btn-primary) / duplicate (.cl-btn) / delete (.cl-btn-danger)
    // findAll('.cl-btn') 匹配所有含 cl-btn 的 class, 用 index 1 取 duplicate
    const btns = w.findAll('.cl-btn');
    await btns[1].trigger('click');
    expect(w.emitted('duplicate')?.[0]).toEqual(['c1']);
  });

  it('shows empty state', () => {
    const w = mount(CycleList, { props: { cycles: [], worldName: '空' } });
    expect(w.text()).toContain('暂无周目');
  });
});
