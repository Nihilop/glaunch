import { useMagicKeys, whenever } from '@vueuse/core'
import { useNavigationStore } from './store'
import { onMounted } from 'vue'

export function useKeyboard() {
  const store = useNavigationStore()

  const {
    arrowUp,
    arrowDown,
    arrowLeft,
    arrowRight,
    enter,
    space,
    escape,
    tab
  } = useMagicKeys()

  const navigate = async (direction: 'up' | 'down' | 'left' | 'right') => {
    const activeZone = store.activeZone
    if (!activeZone) {
      console.debug('[Navigation] No active zone')
      return
    }

    console.debug(`[Navigation] Navigating ${direction} in zone ${activeZone.id}`)
    // Appeler la méthode de navigation du store
    store.navigate(direction)
  }

  // Setup des watchers pour les touches
  whenever(arrowUp, () => navigate('up'))
  whenever(arrowDown, () => navigate('down'))
  whenever(arrowLeft, () => navigate('left'))
  whenever(arrowRight, () => navigate('right'))

  whenever(() => enter.value || space.value, () => {
    store.handleSelect()
  })

  whenever(tab, (pressed) => {
    if (pressed) {
      document.addEventListener('keydown', (e) => {
        if (e.key === 'Tab') {
          e.preventDefault()
        }
      }, { once: true })
    }
  })

  whenever(escape, () => {
    const { activeRegion } = store
    if (!activeRegion?.defaultZone) return

    store.setActiveZone(activeRegion.defaultZone)
  })

  onMounted(() => {
    console.debug('[Navigation] Keyboard navigation initialized')
  })

  return {
    navigate
  }
}