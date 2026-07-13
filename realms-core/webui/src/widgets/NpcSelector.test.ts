import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import NpcSelector from './NpcSelector.vue';

const npcs = [
  { npcId: 'ly', name: '林月如', description: '林家堡大小姐', source: 'npc' as const },
  { npcId: 'xl', name: '李逍遥', description: '余杭镇少年', source: 'npc' as const },
  { npcId: 'oc1', name: '师兄', description: '原创角色', source: 'oc' as const },
];

describe('NpcSelector', () => {
  it('renders NPCs', () => {
    const w = mount(NpcSelector, { props: { npcs } });
    expect(w.text()).toContain('林月如');
    expect(w.text()).toContain('师兄');
  });

  it('emits select', async () => {
    const w = mount(NpcSelector, { props: { npcs } });
    await w.findAll('.ns-card')[0].trigger('click');
    expect(w.emitted('select')?.[0]).toEqual(['ly']);
  });

  it('shows source badges', () => {
    const w = mount(NpcSelector, { props: { npcs } });
    expect(w.text()).toContain('内置');
    expect(w.text()).toContain('OC');
  });
});
