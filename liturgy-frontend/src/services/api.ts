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

  private async fetch<T>(url: string): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseURL}${url}`)
      const data = await response.json()
      return data
    } catch (error) {
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

  async getDayInfo(name: string, year: number, month: number, day: number): Promise<DayInfo> {
    const response = await this.fetch<DayInfo>(`/calendars/${name}/day/${year}/${month}/${day}`)
    if (response.success && response.data) {
      return response.data
    }
    throw new Error(response.error || 'Failed to fetch day info')
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
