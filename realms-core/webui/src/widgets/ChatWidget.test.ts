import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ChatWidget from './ChatWidget.vue';

const sample = [
  { id: '1', role: 'npc' as const, senderName: 'lin_yueru', content: '客官里面请！' },
  { id: '2', role: 'user' as const, senderName: 'user', content: '我住店' },
];

describe('ChatWidget', () => {
  it('renders messages', () => {
    const w = mount(ChatWidget, { props: { messages: sample } });
    expect(w.text()).toContain('客官里面请！');
    expect(w.text()).toContain('我住店');
  });

  it('shows empty state', () => {
    const w = mount(ChatWidget, { props: { messages: [] } });
    expect(w.text()).toContain('等待对话');
  });

  it('emits send on button click', async () => {
    const w = mount(ChatWidget, { props: { messages: sample } });
    const input = w.find('input');
    await input.setValue('你好');
    await w.find('form').trigger('submit.prevent');
    expect(w.emitted('send')).toBeTruthy();
    expect(w.emitted('send')![0]).toEqual(['你好']);
  });

  it('clears input after send', async () => {
    const w = mount(ChatWidget, { props: { messages: [] } });
    const input = w.find('input');
    await input.setValue('测试');
    await w.find('form').trigger('submit.prevent');
    expect((input.element as HTMLInputElement).value).toBe('');
  });

  it('uses npc names from prop', () => {
    const w = mount(ChatWidget, {
      props: { messages: sample, npcNames: { lin_yueru: '林月如' } },
    });
    expect(w.text()).toContain('林月如');
  });
});
