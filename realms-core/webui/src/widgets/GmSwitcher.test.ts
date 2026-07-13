import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import GmSwitcher from './GmSwitcher.vue';

const gms = [
  { gmId: 'style_linyueru', name: '林月如文风', preview: '细腻情感向叙事者…', worldId: 'xiatian_qixia_1' },
  { gmId: 'style_gulong', name: '古龙武侠', preview: '短句凌厉，大量留白…' },
];

describe('GmSwitcher', () => {
  it('renders templates', () => {
    const w = mount(GmSwitcher, { props: { gms } });
    expect(w.text()).toContain('林月如文风');
    expect(w.text()).toContain('古龙武侠');
  });

  it('emits select', async () => {
    const w = mount(GmSwitcher, { props: { gms } });
    await w.findAll('.gs-item')[0].trigger('click');
    expect(w.emitted('select')?.[0]).toEqual(['style_linyueru']);
  });

  it('shows preview on select', async () => {
    const w = mount(GmSwitcher, { props: { gms } });
    await w.findAll('.gs-item')[0].trigger('click');
    expect(w.text()).toContain('细腻情感向叙事者');
  });
});
