import { createRouter, createWebHistory } from 'vue-router'
import TodayView from '../views/TodayView.vue'
import WeekView from '../views/WeekView.vue'
import MonthView from '../views/MonthView.vue'
import SearchView from '../views/SearchView.vue'
import NerdView from '../views/NerdView.vue'
import NovenaView from '../views/NovenaView.vue'
import AboutView from '../views/AboutView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/today',
    },
    {
      path: '/today',
      name: 'Today',
      component: TodayView,
      meta: {
        title: 'Daily Liturgy',
      },
    },
    {
      path: '/week',
      name: 'Week',
      component: WeekView,
      meta: {
        title: 'Week View',
      },
    },
    {
      path: '/month',
      name: 'Month',
      component: MonthView,
      meta: {
        title: 'Month View',
      },
    },
    {
      path: '/search',
      name: 'Search',
      component: SearchView,
      meta: {
        title: 'Search',
      },
    },
    {
      path: '/nerd',
      name: 'Nerd',
      component: NerdView,
      meta: {
        title: 'Advanced View',
      },
    },
    {
      path: '/novena',
      name: 'Novena',
      component: NovenaView,
      meta: {
        title: 'Upcoming Novenas',
      },
    },
    {
      path: '/about',
      name: 'About',
      component: AboutView,
      meta: {
        title: 'About',
      },
    },
  ],
})

// Optional: Update document title based on route
router.beforeEach((to, from, next) => {
  if (to.meta?.title) {
    document.title = `${to.meta.title} - Liturgy Calendar`
  }

  // Set CSS variables based on route
  const app = document.documentElement
  switch (to.name) {
    case 'Today':
      app.style.setProperty('--accent-color', 'rgb(141, 83, 255)')
      break
    case 'Week':
      app.style.setProperty('--accent-color', 'rgb(41, 214, 185)')
      break
    case 'Month':
      app.style.setProperty('--accent-color', 'rgb(141, 83, 255)')
      break
    case 'Search':
      app.style.setProperty('--accent-color', 'rgb(41, 214, 185)')
      break
    case 'Nerd':
      app.style.setProperty('--accent-color', 'rgb(141, 83, 255)')
      break
    case 'Novena':
      app.style.setProperty('--accent-color', 'rgb(141, 83, 255)')
      break
    case 'About':
      app.style.setProperty('--accent-color', 'rgb(41, 214, 185)')
      break
    default:
      app.style.setProperty('--accent-color', 'rgb(141, 83, 255)')
  }

  next()
})

export default router
