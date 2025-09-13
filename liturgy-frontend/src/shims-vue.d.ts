// TypeScript declaration for Vue SFCs so they can be imported with default exports
import { DefineComponent } from 'vue'

declare module '*.vue' {
  const component: DefineComponent<{}, {}, any>
  export default component
}
