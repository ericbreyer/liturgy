import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import LiturgicalColorBar from '../../components/LiturgicalColorBar.vue'

describe('LiturgicalColorBar', () => {
  it('renders with default props', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'red'
      }
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.classes()).toContain('liturgical-color-bar')
    
    // Check inline styles instead of classes for size/orientation
    const style = wrapper.attributes('style')
    expect(style).toContain('background-color: rgb(220, 38, 38)')
    expect(style).toContain('width: 4px') // medium default
    expect(style).toContain('height: 3rem') // medium default
  })

  it('applies correct size styles for small', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'white',
        size: 'small'
      }
    })

    const style = wrapper.attributes('style')
    expect(style).toContain('width: 3px')
    expect(style).toContain('height: 1.5rem')
  })

  it('applies correct orientation styles for horizontal', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'green',
        orientation: 'horizontal'
      }
    })

    const style = wrapper.attributes('style')
    // For horizontal, width and height are swapped
    expect(style).toContain('width: 3rem') // was height in vertical
    expect(style).toContain('height: 4px') // was width in vertical
  })

  it('applies correct background color style', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'violet'
      }
    })

    const style = wrapper.attributes('style')
    expect(style).toContain('background-color: rgb(124, 58, 237)')
  })

  it('handles unknown colors with default', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'unknown'
      }
    })

    const style = wrapper.attributes('style')
    expect(style).toContain('background-color: rgb(107, 114, 128)')
  })

  it('combines size and orientation props correctly', () => {
    const wrapper = mount(LiturgicalColorBar, {
      props: {
        color: 'gold',
        size: 'large',
        orientation: 'horizontal'
      }
    })

    const style = wrapper.attributes('style')
    expect(style).toContain('background-color: rgb(234, 179, 8)')
    // Large + horizontal: width becomes height (4rem), height becomes width (6px)
    expect(style).toContain('width: 4rem')
    expect(style).toContain('height: 6px')
  })
})
