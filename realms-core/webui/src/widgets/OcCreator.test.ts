import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import OcCreator from './OcCreator.vue';

const npcs = [{ npcId: 'ly', name: '林月如' }];
const templates = [
  { templateId: 'tpl_stranger', name: '萍水相逢', affinity: 0, trust: 0, intimacy: 0, typeLabel: 'stranger' },
];

describe('OcCreator', () => {
  it('renders step 1 with input fields', () => {
    const w = mount(OcCreator, { props: { npcs, templates } });
    expect(w.find('input').exists()).toBe(true);
    expect(w.text()).toContain('OC 名称');
  });

  it('shows progress steps', () => {
    const w = mount(OcCreator, { props: { npcs, templates } });
    expect(w.text()).toContain('基本信息');
    expect(w.text()).toContain('初始关系');
    expect(w.text()).toContain('确认');
  });

  it('has next button visible at step 1', () => {
    const w = mount(OcCreator, { props: { npcs, templates } });
    const btns = w.findAll('button');
    expect(btns.some(b => b.text().includes('下一步'))).toBe(true);
  });
});
