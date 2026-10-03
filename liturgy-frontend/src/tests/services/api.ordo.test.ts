import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { api } from '../../services/api'
import { createVespers } from '../fixtures/ordo'

describe('Ordo API', () => {
	const fetchMock = vi.fn<typeof fetch>()
	const endpoints = [
		{
			name: 'vespers',
			url: '/api/ordo/vespers/ef/2025/9/13',
			request: () => api.getOrdoVespers('ef', 2025, 9, 13),
			data: createVespers(),
			fallback: 'Failed to fetch Ordo vespers',
		},
		{
			name: 'sources',
			url: '/api/ordo/vespers/ef/sources/2025/9/13',
			request: () => api.getOrdoVespersSources('ef', 2025, 9, 13),
			data: ['rules/vespers.toml'],
			fallback: 'Failed to fetch Ordo vespers sources',
		},
	]

	beforeEach(() => {
		fetchMock.mockReset()
		vi.stubGlobal('fetch', fetchMock)
	})

	afterEach(() => {
		vi.unstubAllGlobals()
	})

	it.each(endpoints)('returns $name data from the correct endpoint', async ({ request, url, data }) => {
		fetchMock.mockResolvedValue(new Response(JSON.stringify({ success: true, data })))
		await expect(request()).resolves.toEqual(data)
		expect(fetchMock).toHaveBeenCalledExactlyOnceWith(url, { signal: undefined })
	})

	it('accepts an empty sources list', async () => {
		fetchMock.mockResolvedValue(new Response(JSON.stringify({ success: true, data: [] })))
		await expect(api.getOrdoVespersSources('ef', 2025, 9, 13)).resolves.toEqual([])
	})

	it.each(endpoints)('propagates backend errors for $name', async ({ request }) => {
		fetchMock.mockResolvedValue(new Response(JSON.stringify({ success: false, error: 'No office found' })))
		await expect(request()).rejects.toThrow('No office found')
	})

	it.each(endpoints)('rejects missing $name data with the endpoint fallback', async ({ request, fallback }) => {
		fetchMock.mockResolvedValue(new Response(JSON.stringify({ success: true })))
		await expect(request()).rejects.toThrow(fallback)
	})

	it.each(endpoints)('propagates network failures for $name', async ({ request }) => {
		fetchMock.mockRejectedValue(new Error('Network unavailable'))
		await expect(request()).rejects.toThrow('Network unavailable')
	})
})
