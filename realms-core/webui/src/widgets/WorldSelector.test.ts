import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import WorldSelector from './WorldSelector.vue';

const worlds = [
  { worldId: 'w1', name: '仙剑奇侠传一', description: '神州浩土，仙魔纷争', npcCount: 4 },
  { worldId: 'w2', name: '仙剑二', description: '续写传奇', npcCount: 5 },
];

describe('WorldSelector', () => {
  it('renders worlds', () => {
    const w = mount(WorldSelector, { props: { worlds } });
    expect(w.text()).toContain('仙剑奇侠传一');
    expect(w.text()).toContain('仙剑二');
  });

  it('emits select on click', async () => {
    const w = mount(WorldSelector, { props: { worlds } });
    await w.findAll('.ws-card')[0].trigger('click');
    expect(w.emitted('select')?.[0]).toEqual(['w1']);
  });

  it('shows empty state', () => {
    const w = mount(WorldSelector, { props: { worlds: [] } });
    expect(w.text()).toContain('暂无可用世界');
  });

  it('marks active selection', async () => {
    const w = mount(WorldSelector, { props: { worlds } });
    await w.findAll('.ws-card')[1].trigger('click');
    expect(w.findAll('.ws-card')[1].classes()).toContain('active');
  });
});
