import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import OrdoView from '../../views/OrdoView.vue'
import { api, type Vespers } from '../../services/api'
import { calendars, createVespers } from '../fixtures/ordo'

const selectedDate = ref('2025-09-13')

vi.mock('../../composables/useDateNavigation', () => ({
	useDateNavigation: () => ({ selectedDate }),
}))

vi.mock('../../services/api', () => ({
	api: {
		getCalendars: vi.fn(),
		getOrdoVespers: vi.fn(),
		getOrdoVespersSources: vi.fn(),
	},
}))

describe('OrdoView', () => {
	let wrapper: ReturnType<typeof mount<typeof OrdoView>> | undefined

	beforeEach(() => {
		vi.resetAllMocks()
		selectedDate.value = '2025-09-13'
		vi.mocked(api.getCalendars).mockResolvedValue(calendars)
		vi.mocked(api.getOrdoVespers).mockResolvedValue(createVespers())
		vi.mocked(api.getOrdoVespersSources).mockResolvedValue(['rules/vespers.toml'])
	})

	afterEach(() => {
		wrapper?.unmount()
		wrapper = undefined
	})

	it('loads the default calendar and renders the seven office components in order', async () => {
		wrapper = mount(OrdoView)
		await flushPromises()

		expect(api.getCalendars).toHaveBeenCalledOnce()
		expect(api.getOrdoVespers).toHaveBeenCalledExactlyOnceWith('ef', 2025, 9, 13)
		expect(api.getOrdoVespersSources).toHaveBeenCalledExactlyOnceWith('ef', 2025, 9, 13)
		expect(wrapper.get('select').element.value).toBe('ef')
		expect(wrapper.get('h2').text()).toBe('Test Vespers')
		expect(wrapper.findAll('.ordo-key').map((item) => item.text())).toEqual([
			'Antiphons', 'Psalms', 'Chapter', 'Hymn', 'Verse', 'Magnificat Antiphon', 'Collect',
		])
		expect(wrapper.findAll('.ordo-value').map((item) => item.text())).toEqual([
			'Common: Confessor', 'Psalter', 'Proper', 'Ordinary: Advent',
			'Sunday (Advent)', 'Octave: Christmas', 'Proper',
		])
		expect(wrapper.vm.sources).toEqual(['rules/vespers.toml'])
		expect(wrapper.find('.error').exists()).toBe(false)
	})

	it('falls back to the first available calendar when ef is absent', async () => {
		vi.mocked(api.getCalendars).mockResolvedValue([calendars[1]!])
		wrapper = mount(OrdoView)
		await flushPromises()

		expect(wrapper.get('select').element.value).toBe('of')
		expect(api.getOrdoVespers).toHaveBeenCalledWith('of', 2025, 9, 13)
		expect(api.getOrdoVespersSources).toHaveBeenCalledWith('of', 2025, 9, 13)
	})

	it('reloads both endpoints when the calendar or shared date changes', async () => {
		wrapper = mount(OrdoView)
		await flushPromises()
		vi.mocked(api.getOrdoVespers).mockClear()
		vi.mocked(api.getOrdoVespersSources).mockClear()

		await wrapper.get('select').setValue('of')
		await flushPromises()
		expect(api.getOrdoVespers).toHaveBeenLastCalledWith('of', 2025, 9, 13)
		expect(api.getOrdoVespersSources).toHaveBeenLastCalledWith('of', 2025, 9, 13)

		selectedDate.value = '2025-09-14'
		await nextTick()
		await flushPromises()
		expect(api.getOrdoVespers).toHaveBeenLastCalledWith('of', 2025, 9, 14)
		expect(api.getOrdoVespersSources).toHaveBeenLastCalledWith('of', 2025, 9, 14)
		expect(api.getOrdoVespers).toHaveBeenCalledTimes(2)
		expect(api.getOrdoVespersSources).toHaveBeenCalledTimes(2)
	})

	it('disables refresh while loading and restores it after the response', async () => {
		let resolveVespers!: (value: Vespers) => void
		vi.mocked(api.getOrdoVespers).mockReturnValue(new Promise((resolve) => {
			resolveVespers = resolve
		}))
		wrapper = mount(OrdoView)
		await flushPromises()

		expect(wrapper.get('button').element.disabled).toBe(true)
		expect(wrapper.get('button').text()).toContain('Loading')
		expect(wrapper.find('section').exists()).toBe(false)
		resolveVespers(createVespers())
		await flushPromises()
		expect(wrapper.get('button').element.disabled).toBe(false)
		expect(wrapper.get('button').text()).toBe('Refresh')
		expect(wrapper.get('h2').text()).toBe('Test Vespers')
	})

	it('shows a request error and recovers on refresh', async () => {
		vi.mocked(api.getOrdoVespers).mockRejectedValueOnce(new Error('Office unavailable'))
		wrapper = mount(OrdoView)
		await flushPromises()

		expect(wrapper.get('.error').text()).toBe('Office unavailable')
		expect(wrapper.find('section').exists()).toBe(false)
		expect(wrapper.get('button').element.disabled).toBe(false)
		expect(api.getOrdoVespersSources).not.toHaveBeenCalled()

		await wrapper.get('button').trigger('click')
		await flushPromises()
		expect(wrapper.find('.error').exists()).toBe(false)
		expect(wrapper.get('h2').text()).toBe('Test Vespers')
		expect(api.getOrdoVespers).toHaveBeenCalledTimes(2)
	})

	it('shows source request errors and stops loading', async () => {
		vi.mocked(api.getOrdoVespersSources).mockRejectedValue(new Error('Sources unavailable'))
		wrapper = mount(OrdoView)
		await flushPromises()

		expect(wrapper.get('.error').text()).toBe('Sources unavailable')
		expect(wrapper.get('button').element.disabled).toBe(false)
		expect(wrapper.vm.sources).toEqual([])
	})
})
