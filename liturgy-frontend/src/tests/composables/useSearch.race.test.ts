import { ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { api, type SearchResult } from '../../services/api'
import { useSearch } from '../../composables/useSearch'

vi.mock('../../services/api', () => ({
  api: { searchFeasts: vi.fn() },
}))

vi.mock('../../composables/useCalendarSelection', () => ({
  useCalendarSelection: () => ({ selectedCalendars: ref(['calendar']) }),
}))

function deferredResults() {
  let resolve!: (results: SearchResult[]) => void
  let reject!: (error: Error) => void
  const promise = new Promise<SearchResult[]>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

describe('useSearch request ownership', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    useSearch().clearSearch()
    vi.spyOn(console, 'log').mockImplementation(() => {})
    vi.spyOn(console, 'warn').mockImplementation(() => {})
  })

  afterEach(() => {
    useSearch().clearSearch()
    vi.restoreAllMocks()
  })

  it('ignores an older response while the newer search is loading', async () => {
    const older = deferredResults()
    const newer = deferredResults()
    vi.mocked(api.searchFeasts)
      .mockReturnValueOnce(older.promise)
      .mockReturnValueOnce(newer.promise)
    const search = useSearch()
    search.searchQuery.value = 'older'
    const olderSearch = search.performSearch()
    const olderSignal = vi.mocked(api.searchFeasts).mock.calls[0][2]
    search.searchQuery.value = 'newer'
    const newerSearch = search.performSearch()

    expect(olderSignal).toBeInstanceOf(AbortSignal)
    expect(olderSignal?.aborted).toBe(true)
    expect(vi.mocked(api.searchFeasts).mock.calls[1][2]?.aborted).toBe(false)
    older.resolve([{ name: 'Older feast', score: 1 } as SearchResult])
    await olderSearch
    expect(search.searchResults.value).toEqual([])
    expect(search.isLoading.value).toBe(true)

    newer.resolve([{ name: 'Newer feast', score: 2 } as SearchResult])
    await newerSearch
    expect(search.searchResults.value.map((result) => result.name)).toEqual(['Newer feast'])
    expect(search.isLoading.value).toBe(false)
  })

  it('preserves newer results when an older response resolves last', async () => {
    const older = deferredResults()
    const newer = deferredResults()
    vi.mocked(api.searchFeasts)
      .mockReturnValueOnce(older.promise)
      .mockReturnValueOnce(newer.promise)
    const search = useSearch()
    search.searchQuery.value = 'older'
    const olderSearch = search.performSearch()
    search.searchQuery.value = 'newer'
    const newerSearch = search.performSearch()
    newer.resolve([{ name: 'Newer feast', score: 2 } as SearchResult])
    await newerSearch
    older.resolve([{ name: 'Older feast', score: 1 } as SearchResult])
    await olderSearch

    expect(search.searchResults.value.map((result) => result.name)).toEqual(['Newer feast'])
    expect(search.isLoading.value).toBe(false)
    expect(search.error.value).toBeNull()
  })

  it.each(['resolve', 'reject'] as const)(
    'clears loading immediately and ignores a late %s after clearResults',
    async (completion) => {
      const pending = deferredResults()
      vi.mocked(api.searchFeasts).mockReturnValueOnce(pending.promise)
      const search = useSearch()
      search.searchQuery.value = 'pending'
      const pendingSearch = search.performSearch()
      const signal = vi.mocked(api.searchFeasts).mock.calls[0][2]

      search.clearResults()
      expect(signal?.aborted).toBe(true)
      expect(search.isLoading.value).toBe(false)
      expect(search.hasSearched.value).toBe(false)
      if (completion === 'resolve') {
        pending.resolve([{ name: 'Late feast', score: 1 } as SearchResult])
      } else {
        pending.reject(new Error('Late failure'))
      }
      await pendingSearch

      expect(search.searchResults.value).toEqual([])
      expect(search.error.value).toBeNull()
      expect(search.isLoading.value).toBe(false)
      expect(search.hasSearched.value).toBe(false)
    },
  )
})