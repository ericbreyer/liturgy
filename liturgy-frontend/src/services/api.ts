// Types matching your Rust API
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export interface CalendarInfo {
  name: string
  display_name: string
  description: string
  commemoration_interpretation: string
}

export interface CalendarDetails {
  name: string
  display_name: string
  description: string
}

export interface YearCalendarData {
  calendar_name: string
  year: number
  csv_data: string
  total_days: number
}

export interface LitugicalUnit {
  desc: string
  rank: string
  date: string
  color: string
}

export interface DayInfo {
  desc: {
    date: string
    day_in_season: string
    day_rank: string
    day: LitugicalUnit
    commemorations: LitugicalUnit[]
  }
}

export interface SearchResult {
  name: string
  description: string
  date?: string
  rank: string
  score: number
  color: string
}

export interface CalendarStats {
  year: number
  total_days: number
  feast_days: number
  seasons: SeasonStats[]
}

export interface SeasonStats {
  name: string
  days: number
  color: string
}

// Simple API client without axios dependency
class ApiClient {
  private baseURL = '/api'

  // Dev-only in-memory metrics for debugging network/race issues.
  // Not persisted; useful during local dev or in-browser debugging.
  private _metrics = {
    requests: 0,
    aborted: 0,
    errors: 0,
    totalDurationMs: 0,
    lastRequests: [] as Array<{ url: string; durationMs: number; aborted?: boolean }>,
  }

  // Accept an optional AbortSignal so callers can cancel in-flight requests.
  private async fetch<T>(url: string, signal?: AbortSignal): Promise<ApiResponse<T>> {
    const start = Date.now()
    this._metrics.requests += 1
    try {
      const response = await fetch(`${this.baseURL}${url}`, { signal })
      const data = await response.json()
      const duration = Date.now() - start
      this._metrics.totalDurationMs += duration
      this._metrics.lastRequests.unshift({ url, durationMs: duration })
      if (this._metrics.lastRequests.length > 50) this._metrics.lastRequests.pop()
      return data
    } catch (error: any) {
      const duration = Date.now() - start
      this._metrics.totalDurationMs += duration
      this._metrics.lastRequests.unshift({ url, durationMs: duration, aborted: error?.name === 'AbortError' })
      if (this._metrics.lastRequests.length > 50) this._metrics.lastRequests.pop()
      if (error?.name === 'AbortError') {
        this._metrics.aborted += 1
      } else {
        this._metrics.errors += 1
      }
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }
    }
  }

  async getCalendars(): Promise<CalendarInfo[]> {
    const response = await this.fetch<CalendarInfo[]>('/calendars')
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch calendars')
  }

  async getCalendar(name: string): Promise<CalendarDetails> {
    const response = await this.fetch<CalendarDetails>(`/calendars/${name}`)
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch calendar')
  }

  async getYearCalendar(name: string, year: number): Promise<YearCalendarData> {
    const response = await this.fetch<YearCalendarData>(`/calendars/${name}/year/${year}`)
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch year calendar')
  }

  // Accept an optional AbortSignal so callers can cancel the request.
  async getDayInfo(
    name: string,
    year: number,
    month: number,
    day: number,
    signal?: AbortSignal,
  ): Promise<DayInfo> {
    const url = `/calendars/${name}/day/${year}/${month}/${day}`
    const start = Date.now()
    const response = await this.fetch<DayInfo>(url, signal)
    const duration = Date.now() - start
    // additional metrics specific to getDayInfo
    this._metrics.lastRequests.unshift({ url, durationMs: duration })
    if (this._metrics.lastRequests.length > 50) this._metrics.lastRequests.pop()

    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch day info')
  }

  // Expose metrics for dev debugging (not for production monitoring)
  _getDebugMetrics() {
    return { ...this._metrics }
  }

  async searchFeasts(name: string, query: string): Promise<SearchResult[]> {
    const response = await this.fetch<SearchResult[]>(
      `/calendars/${name}/search?q=${encodeURIComponent(query)}`,
    )
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to search feasts')
  }

  async getCalendarStats(name: string, year: number): Promise<CalendarStats> {
    const response = await this.fetch<CalendarStats>(`/calendars/${name}/stats/${year}`)
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch calendar stats')
  }
}

export const api = new ApiClient()
