import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export function useDateNavigation(routeName: 'Today' | 'Week' | 'Month' | 'Nerd' | 'Ordo') {
  const route = useRoute()
  const router = useRouter()

  // Get selected date from query param or default to today
  const selectedDate = computed(() => {
    const dateParam = route.query.date as string
    if (dateParam && /^\d{4}-\d{2}-\d{2}$/.test(dateParam)) {
      return dateParam
    }
    return stringFromDate(new Date())
  })

  // Get formatted date for display
  const formattedDate = computed(() => {
    const [year, month, day] = selectedDate.value.split('-').map(Number)
    const date = new Date(year, month - 1, day) // month is 0-indexed

    if (routeName === 'Month') {
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
      })
    }

    return date.toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    })
  })

  function updateSelectedDate(newDate: string) {
    // keep other query params intact
    // but update the date param
    const newQuery = { ...route.query }
    newQuery.date = newDate
    router.push({ name: routeName, query: newQuery })
  }

  function stringFromDate(date: Date): string {
    return date.getFullYear() + '-' +
      String(date.getMonth() + 1).padStart(2, '0') + '-' +
      String(date.getDate()).padStart(2, '0')
  }

  function goToToday() {
    const today = new Date()
    const todayString = stringFromDate(today)
    updateSelectedDate(todayString)
  }

  function goToPrevious() {
    const [year, month, day] = selectedDate.value.split('-').map(Number)
    let date: Date

    if (routeName === 'Today' || routeName === 'Nerd' || routeName === 'Ordo' || routeName == 'Week') {
      date = new Date(year, month - 1, day - 1)
    // } else if (routeName === 'Week') {
    //   date = new Date(year, month - 1, day - 7)
    } else {
      // Month
      date = new Date(year, month - 2, day) // Go to previous month
    }

    const dateString = stringFromDate(date)
    updateSelectedDate(dateString)
  }

  function goToNext() {
    const [year, month, day] = selectedDate.value.split('-').map(Number)
    let date: Date

    if (routeName === 'Today' || routeName === 'Nerd' || routeName === 'Ordo' || routeName == 'Week') {
      date = new Date(year, month - 1, day + 1)
    // } else if (routeName === 'Week') {
    //   date = new Date(year, month - 1, day + 7)
    } else {
      // Month
      date = new Date(year, month, day) // Go to next month
    }

    const dateString = stringFromDate(date)
    updateSelectedDate(dateString)
  }

  return {
    selectedDate,
    formattedDate,
    updateSelectedDate,
    goToToday,
    goToPrevious,
    goToNext,
    route,
  }
}
