import type {Ref} from 'vue'
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {useNavigationStore} from './store'
import type {Bounds, ZoneConfig} from './types'
import {useResizeObserver} from '@vueuse/core'
import {useRouter} from "vue-router";

export function useZone(elementRef: Ref<HTMLElement | null>, config: Omit<ZoneConfig, 'regionId'>) {
  const store = useNavigationStore()
  const debug = computed(() => store.debug)
  const router = useRouter()
  const initialized = ref(false)
  const lastActivationSource = ref<'keyboard' | 'mouse'>('keyboard')
  const isMouseMoving = ref(false)
  const keyboardCooldown = ref(false)
  let cooldownTimer: number | null = null

  // Créer un ID unique pour la zone si non fourni
  const zoneId = config.id || `zone-${Math.random().toString(36).slice(2, 11)}`

  // État local
  const items = ref<HTMLElement[]>([])
  const bounds = ref<Bounds | null>(null)

  // Computed
  const isActive = computed(() => store.activeZone?.id === zoneId)
  const activeIndex = computed(() => isActive.value ? store.activeIndex : -1)

  // Mettre à jour les bounds de la zone
  const updateBounds = () => {
    if (!elementRef.value) return

    const rect = elementRef.value.getBoundingClientRect()
    bounds.value = {
      top: rect.top,
      right: rect.right,
      bottom: rect.bottom,
      left: rect.left
    }

    // Mettre à jour la liste des éléments
    items.value = Array.from(elementRef.value.children) as HTMLElement[]

    // Mettre à jour la zone dans le store
    if (store.zones.has(zoneId)) {
      const zone = store.zones.get(zoneId)!
      zone.bounds = bounds.value
      zone.items = items.value
      zone.ref = elementRef.value
    }

    if (config.hoverable) {
      cleanupHoverListeners()
      setupHoverListeners()
    }
  }

  // Initialiser la zone
  const initializeZone = () => {
    if (initialized.value || !elementRef.value) return

    // Déterminer la région parente
    const activeRegion = store.activeRegion
    if (!activeRegion) {
      console.warn(`[Navigation] Waiting for active region for zone ${zoneId}`)
      return
    }

    store.registerZone({
      ...config,
      id: zoneId,
      regionId: activeRegion.id,
      ref: elementRef
    })

    // Initial bounds update
    updateBounds()
    initialized.value = true
  }

  // Observer les changements de taille
  useResizeObserver(elementRef, () => {
    if (initialized.value) {
      updateBounds()
    }
  })

  // Watch pour l'élément ref
  watch(() => elementRef.value, (newEl) => {
    if (newEl && !initialized.value) {
      initializeZone()
    }
  }, { immediate: true })

  const scrollItemIntoView = (index: number) => {
    if (!elementRef.value || !items.value[index]) return

    const zone = store.zones.get(zoneId)
    // Ne pas scroller si l'activation vient de la souris
    if (zone?._lastActivationSource === 'mouse') return

    const container = elementRef.value
    const item = items.value[index]

    if (config.type === 'horizontal') {
      const firstItemLeft = items.value[0]?.offsetLeft || 0
      const selectedItemLeft = item.offsetLeft
      const scrollOffset = selectedItemLeft - firstItemLeft

      container.scrollTo({
        left: scrollOffset,
        behavior: 'smooth'
      })
    } else if (config.type === 'grid' || config.type === 'vertical') {
      const firstItemTop = items.value[0]?.offsetTop || 0
      const selectedItemTop = item.offsetTop
      const scrollOffset = selectedItemTop - firstItemTop

      container.scrollTo({
        top: scrollOffset,
        behavior: 'smooth'
      })
    }
  }

  const updateZoneState = () => {
    if (!elementRef.value) return

    if (debug.value) {
      console.log('[Navigation Debug] Updating zone state', {
        id: config.id,
        memory: config.memory,
        items: elementRef.value.children.length
      })
    }

    updateBounds()

    // Si cette zone a memory activé, le store tentera de restaurer son état
    if (config.memory) {
      const route = router?.currentRoute.value
      if (route) {
        store.restoreState(route.path)
      }
    }
  }

  const setActiveElement = (index: number) => {
    if (!initialized.value) return false
    return store.setActiveElement(zoneId, index)
  }

  const handleItemHover = (event: MouseEvent) => {
    if (!config.hoverable || !elementRef.value || keyboardCooldown.value) return

    const target = event.target as HTMLElement
    const itemIndex = Array.from(elementRef.value.children).indexOf(target)

    if (itemIndex !== -1) {
      const zone = store.zones.get(zoneId)
      if (zone) {
        zone._lastActivationSource = 'mouse'
        setActiveElement(itemIndex)
      }
    }
  }

  // Fonction pour ajouter les listeners de hover
  const setupHoverListeners = () => {
    if (!config.hoverable || !elementRef.value) return

    const children = Array.from(elementRef.value.children) as HTMLElement[]
    children.forEach((child) => {
      child.addEventListener('mouseenter', handleItemHover)
    })
  }

  // Fonction pour nettoyer les listeners
  const cleanupHoverListeners = () => {
    if (!config.hoverable || !elementRef.value) return

    const children = Array.from(elementRef.value.children) as HTMLElement[]
    children.forEach((child) => {
      child.removeEventListener('mouseenter', handleItemHover)
    })
  }

  // Watch pour le scroll automatique
  watch(activeIndex, (newIndex) => {
    if (isActive.value && newIndex >= 0) {
      scrollItemIntoView(newIndex)
    }
  })

  watch([
    () => store.activeZone?.id === zoneId ? store.activeIndex : -1,
    () => store.activeZone?._lastActivationSource
  ], ([newIndex, source]) => {
    if (newIndex >= 0 && source === 'keyboard') {
      // Activer le cooldown
      keyboardCooldown.value = true

      // Nettoyer le timer précédent si existe
      if (cooldownTimer) {
        clearTimeout(cooldownTimer)
      }

      // Réactiver le hover après 1 seconde
      cooldownTimer = window.setTimeout(() => {
        keyboardCooldown.value = false
      }, 1000) // 1 seconde de cooldown
    }
  })

  onMounted(() => {
    initializeZone()
  })

  onUnmounted(() => {
    if (cooldownTimer) {
      clearTimeout(cooldownTimer)
    }
    if (initialized.value) {
      cleanupHoverListeners()
      store.unregisterZone(zoneId)
    }
  })

  return {
    zoneId,
    isActive,
    activeIndex,
    items,
    bounds,
    updateBounds,
    updateZoneState,
    setActiveElement,
    lastActivationSource
  }
}