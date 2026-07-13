import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StateWidget from './StateWidget.vue';
import type { CharacterState } from '../types';

const sample: CharacterState = {
  cycleId: 'c1',
  npcId: 'lin_yueru',
  hp: 75, hpMax: 100,
  mp: 30, mpMax: 50,
  locationId: '余杭镇',
  statusFlags: { poisoned: true },
  arcs: { trust_user: 55 },
  arcPhase: '朋友',
  perceivedMood: 'happy',
};

describe('StateWidget', () => {
  it('renders HP/MP bars', () => {
    const w = mount(StateWidget, { props: { state: sample } });
    expect(w.text()).toContain('75/100');
    expect(w.text()).toContain('30/50');
  });

  it('renders location', () => {
    const w = mount(StateWidget, { props: { state: sample } });
    expect(w.text()).toContain('余杭镇');
  });

  it('renders arc phase', () => {
    const w = mount(StateWidget, { props: { state: sample } });
    expect(w.text()).toContain('朋友');
  });

  it('renders mood', () => {
    const w = mount(StateWidget, { props: { state: sample } });
    expect(w.text()).toContain('happy');
  });

  it('renders status flags', () => {
    const w = mount(StateWidget, { props: { state: sample } });
    expect(w.text()).toContain('poisoned');
  });

  it('handles zero max values gracefully', () => {
    const zero: CharacterState = { ...sample, hp: 0, hpMax: 0, mp: 0, mpMax: 0 };
    const w = mount(StateWidget, { props: { state: zero } });
    expect(w.text()).toContain('0/0');
  });
});
