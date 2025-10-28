import './styles/main.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

// Import icon assets so Vite includes them in the build output. We'll update
// the <link> tags at runtime to point to the processed (hashed) asset URLs
// so favicons and manifest are available from the dist folder.
import favicon96 from './assets/icon/favicon-96x96.png'
import faviconSvg from './assets/icon/favicon.svg'
import appleTouch from './assets/icon/apple-touch-icon.png'
import siteManifest from './assets/icon/site.webmanifest'

function ensureLink(rel: string, href: string, attrs: Record<string, string> = {}) {
	let el = document.querySelector(`link[rel="${rel}"]`) as HTMLLinkElement | null
	if (!el) {
		el = document.createElement('link')
		el.rel = rel
		document.head.appendChild(el)
	}
	el.href = href
	for (const [k, v] of Object.entries(attrs)) {
		el.setAttribute(k, v)
	}
}

// Update favicon and manifest links to the processed asset URLs
try {
	ensureLink('icon', favicon96, { type: 'image/png', sizes: '96x96' })
	ensureLink('icon', faviconSvg, { type: 'image/svg+xml' })
	ensureLink('apple-touch-icon', appleTouch, { sizes: '180x180' })
	ensureLink('manifest', siteManifest)
} catch (e) {
	// non-fatal: if DOM isn't available or imports fail, the app will still run
	// and the default favicon (public/favicon.ico) will be used by browsers.
	// eslint-disable-next-line no-console
	console.warn('Failed to set favicons programmatically', e)
}

const app = createApp(App)

app.use(router)
app.mount('#app')
