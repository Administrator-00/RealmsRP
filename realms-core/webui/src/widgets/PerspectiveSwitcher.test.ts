import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PerspectiveSwitcher from './PerspectiveSwitcher.vue';

const perspectives = [
  { id: 'pov_npc_ly', name: '林月如', source: 'npc' as const },
  { id: 'pov_oc_shixiong', name: '师兄', source: 'oc' as const },
];

describe('PerspectiveSwitcher', () => {
  it('renders perspectives', () => {
    const w = mount(PerspectiveSwitcher, { props: { perspectives } });
    expect(w.text()).toContain('林月如');
    expect(w.text()).toContain('师兄');
  });

  it('emits select', async () => {
    const w = mount(PerspectiveSwitcher, { props: { perspectives } });
    await w.findAll('.ps-item')[1].trigger('click');
    expect(w.emitted('select')?.[0]).toEqual(['pov_oc_shixiong']);
  });

  it('shows source badges', () => {
    const w = mount(PerspectiveSwitcher, { props: { perspectives } });
    expect(w.text()).toContain('内置');
    expect(w.text()).toContain('OC');
  });
});
