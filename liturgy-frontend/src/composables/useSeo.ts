import { computed, unref, type MaybeRef } from 'vue'
import { useHead } from '@vueuse/head'

type SeoOptions = {
  title?: MaybeRef<string | undefined>
  description?: MaybeRef<string | undefined>
  path?: MaybeRef<string | undefined>
  keywords?: MaybeRef<string | undefined>
  image?: MaybeRef<string | undefined>
}

const SITE_NAME = 'Liturgy'
const BASE_URL = 'https://liturgy.ericbreyer.com'

export function useSeo(options: SeoOptions) {
  const title = computed(() => {
    const t = unref(options.title)
    return t ? `${t} — ${SITE_NAME}` : SITE_NAME
  })

  const description = computed(() => unref(options.description) || '')
  const keywords = computed(() => unref(options.keywords) || '')
  const image = computed(() => unref(options.image) || '')
  const path = computed(() => unref(options.path) || '')

  const canonical = computed(() => {
    const p = path.value || ''
    // ensure leading slash
    const normalized = p.startsWith('/') || p === '' ? p : `/${p}`
    return `${BASE_URL}${normalized}`
  })

  // Build meta arrays with sensible defaults
  const meta = computed(() => {
    const m: Array<Record<string, string>> = []
    if (description.value) m.push({ name: 'description', content: description.value })
    if (keywords.value) m.push({ name: 'keywords', content: keywords.value })
    // Open Graph
    m.push({ property: 'og:title', content: title.value })
    if (description.value) m.push({ property: 'og:description', content: description.value })
    if (image.value) m.push({ property: 'og:image', content: image.value })
    m.push({ property: 'og:site_name', content: SITE_NAME })
    // Twitter
    m.push({ name: 'twitter:card', content: image.value ? 'summary_large_image' : 'summary' })
    m.push({ name: 'twitter:title', content: title.value })
    if (description.value) m.push({ name: 'twitter:description', content: description.value })
    if (image.value) m.push({ name: 'twitter:image', content: image.value })
    return m
  })

  // Apply head using @vueuse/head (accepts reactive objects)
  useHead({
    title: title,
    meta: meta,
    link: [{ rel: 'canonical', href: canonical }],
  })
}

export default useSeo
