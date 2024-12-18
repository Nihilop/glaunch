import {App} from 'vue'
import {useNavigationStore} from './store'
import {useKeyboard} from './useKeyboard'
import {useRegion} from './useRegion'
import {useZone} from './useZone'
import {useSound} from './useSound'

export * from './types'

export interface KeyboardNavigationOptions {
  debug?: boolean
  muted?: boolean
}

export const createKeyboardNavigation = (options: KeyboardNavigationOptions = {}) => {
  return {
    install(app: App) {
      // Vérifier si vue-router est installé
      const router = app.config.globalProperties.$router
      if (!router) {
        throw new Error('vue-router must be installed to use memory feature')
      }

      // Créer le store
      const store = useNavigationStore()

      // Configurer les options
      if (options.debug) {
        store.debug = true
      }

      // Configurer les hooks du router
      router.beforeEach((to, from) => {
        store.saveState(from.path)
        return true
      })

      router.beforeResolve((to) => {
        return true
      })

      router.afterEach((to) => {
        store.restoreState(to.path)
      })
    }
  }
}

export {
  useKeyboard,
  useRegion,
  useZone,
  useSound
}