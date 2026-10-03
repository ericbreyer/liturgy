import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import OrdoView from '../../views/OrdoView.vue'
import { calendars, createVespers } from '../fixtures/ordo'

vi.mock('../../composables/useDateNavigation', () => ({
	useDateNavigation: () => ({ selectedDate: ref('2025-09-13') }),
}))

vi.mock('../../services/api', () => ({
	api: {
		getCalendars: async () => calendars,
		getOrdoVespers: async () => createVespers(),
		getOrdoVespersSources: async () => [],
	},
}))

describe('Ordo location formatting', () => {
	let wrapper: ReturnType<typeof mount<typeof OrdoView>>

	beforeEach(async () => {
		wrapper = mount(OrdoView)
		await flushPromises()
	})

	afterEach(() => {
		wrapper.unmount()
	})

	it.each([
		[{ type: 'Common', name: 'Confessor' }, 'Common: Confessor'],
		[{ type: 'Common' }, 'Common'],
		[{ type: 'Proper' }, 'Proper'],
		[{ type: 'Ordinary', source: 'Advent' }, 'Ordinary: Advent'],
		[{ type: 'Ordinary' }, 'Ordinary'],
		[{ type: 'Octave', source: 'Christmas' }, 'Octave: Christmas'],
		[{ type: 'Octave' }, 'Octave'],
		[{ type: 'Psalter' }, 'Psalter'],
		[{ type: 'Sunday', source: 'Advent' }, 'Sunday (Advent)'],
		[{ type: 'Sunday' }, 'Sunday'],
		[{ Common: 'Confessor' }, 'Common of Confessor'],
		[{ Common: '' }, 'Common'],
		[{ Proper: null }, 'Proper'],
		[{ Ordinary: 'Advent' }, 'Ordinary of Advent'],
		[{ Octave: 'Christmas' }, 'Octave of Christmas'],
		[{ Psalter: null }, 'Psalter'],
		[{ Sunday: 'Advent' }, 'Sunday (Advent)'],
		['Proper', 'Proper'],
		[{ type: 'Unknown', source: 'Advent' }, '{"type":"Unknown","source":"Advent"}'],
		[{ Other: 'Source' }, 'Other: Source'],
		[null, 'null'],
	])('formats %j as %s', (location, expected) => {
		expect(wrapper.vm.formatLocation(location)).toBe(expected)
	})
})
