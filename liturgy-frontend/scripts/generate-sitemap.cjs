#!/usr/bin/env node
const fs = require('fs')
const path = require('path')

const startYear = parseInt(process.argv[2], 10) || new Date().getFullYear()
const endYear = parseInt(process.argv[3], 10) || startYear

function pad(n) {
  return n < 10 ? `0${n}` : `${n}`
}

function datesForYear(year) {
  const dates = []
  const start = new Date(year, 0, 1)
  const end = new Date(year, 11, 31)
  for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
    const y = d.getFullYear()
    const m = pad(d.getMonth() + 1)
    const day = pad(d.getDate())
    dates.push({ date: `${y}-${m}-${day}`, month: `${y}-${m}` })
  }
  return dates
}

const BASE = 'https://liturgy.ericbreyer.com'
const urls = new Set()

for (let y = startYear; y <= endYear; y++) {
  const ds = datesForYear(y)
  ds.forEach((d) => {
    urls.add(`${BASE}/today?date=${d.date}`)
    urls.add(`${BASE}/month?date=${d.month}`)
  })
}

['/', '/today', '/week', '/month', '/search', '/about'].forEach((p) => urls.add(BASE + p))

const sitemap = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${[...urls]
  .map((u) => `  <url>\n    <loc>${u}</loc>\n  </url>`)
  .join('\n')}\n</urlset>`

const out = path.join(__dirname, '..', 'public', 'sitemap.xml')
fs.writeFileSync(out, sitemap, { encoding: 'utf8' })
console.log('Wrote', out, 'with', urls.size, 'entries')
