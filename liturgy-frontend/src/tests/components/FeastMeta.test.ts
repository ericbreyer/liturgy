import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import FeastMeta from '../../components/FeastMeta.vue'

describe('FeastMeta', () => {
  it('renders with rank only', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'solemnity',
      },
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.find('.feast-rank').text()).toBe('solemnity')
    expect(wrapper.find('.feast-calendars').exists()).toBe(false)
    expect(wrapper.find('.commemoration-count').exists()).toBe(false)
  })

  it('renders with calendars as string', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'feast',
        calendars: 'grc',
      },
    })

    expect(wrapper.find('.feast-rank').text()).toBe('feast')
    expect(wrapper.find('.feast-calendars').text()).toBe('(GRC)')
  })

  it('renders with calendars as array', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'memorial',
        calendars: ['grc', 'grc-us'],
      },
    })

    expect(wrapper.find('.feast-rank').text()).toBe('memorial')
    expect(wrapper.find('.feast-calendars').text()).toBe('(GRC, GRC-US)')
  })

  it('renders with commemoration count', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'optional memorial',
        commemorationCount: 3,
      },
    })

    expect(wrapper.find('.feast-rank').text()).toBe('optional memorial')
    expect(wrapper.find('.commemoration-count').text()).toBe('+3')
  })

  it('renders with all properties', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'solemnity',
        calendars: ['grc', 'grc-us'],
        commemorationCount: 2,
      },
    })

    expect(wrapper.find('.feast-rank').text()).toBe('solemnity')
    expect(wrapper.find('.feast-calendars').text()).toBe('(GRC, GRC-US)')
    expect(wrapper.find('.commemoration-count').text()).toBe('+2')
  })

  it('applies correct size classes', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'feast',
        size: 'small',
      },
    })

    expect(wrapper.classes()).toContain('feast-meta--small')
  })

  it('applies default medium size when no size specified', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'feast',
      },
    })

    expect(wrapper.classes()).toContain('feast-meta--medium')
  })

  it('handles large size', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'feast',
        size: 'large',
      },
    })

    expect(wrapper.classes()).toContain('feast-meta--large')
  })

  it('applies correct base classes', () => {
    const wrapper = mount(FeastMeta, {
      props: {
        rank: 'feast',
      },
    })

    expect(wrapper.classes()).toContain('feast-meta')
  })
})
