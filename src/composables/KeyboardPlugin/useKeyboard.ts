import { useMagicKeys, whenever } from '@vueuse/core'
import { useNavigationStore } from './store'
import {onMounted, watch} from 'vue'

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
    tab,
    current
  } = useMagicKeys()

  const navigate = async (direction: 'up' | 'down' | 'left' | 'right') => {
    const activeZone = store.activeZone
    if (!activeZone) {
      console.debug('[Navigation] No active zone')
      return
    }
    store.navigate(direction)
  }

  const intervals = {
    up: null,
    down: null,
    left: null,
    right: null
  }

  // Fonction pour démarrer la répétition
  const startRepeat = (direction: 'up' | 'down' | 'left' | 'right') => {
    if (intervals[direction]) return
    intervals[direction] = setInterval(() => navigate(direction), 200) // Répéter toutes les 100 ms
  }

  // Fonction pour arrêter la répétition
  const stopRepeat = (direction: 'up' | 'down' | 'left' | 'right') => {
    if (intervals[direction]) {
      clearInterval(intervals[direction])
      intervals[direction] = null
    }
  }

  // Setup des watchers pour les touches
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

  watch(() => Array.from(current), (currentKeys) => {
    if (currentKeys.includes('arrowright')) {
      startRepeat('right')
    } else {
      stopRepeat('right')
    }

    if (currentKeys.includes('arrowleft')) {
      startRepeat('left')
    } else {
      stopRepeat('left')
    }

    if (currentKeys.includes('arrowup')) {
      startRepeat('up')
    } else {
      stopRepeat('up')
    }

    if (currentKeys.includes('arrowdown')) {
      startRepeat('down')
    } else {
      stopRepeat('down')
    }
  })

  return {
    navigate
  }
}