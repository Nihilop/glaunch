import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import type { Ref } from 'vue'
import { useNavigationStore } from './store'
import type { ZoneConfig, Bounds } from './types'
import { useResizeObserver, useElementBounding, useElementVisibility } from '@vueuse/core'
import {useRouter} from "vue-router";

export function useZone(elementRef: Ref<HTMLElement | null>, config: Omit<ZoneConfig, 'regionId'>) {
  const store = useNavigationStore()
  const debug = computed(() => store.debug)
  const router = useRouter()
  const initialized = ref(false)

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

  // Méthodes d'aide
  // useZone.ts
  const scrollItemIntoView = (index: number) => {
    if (!elementRef.value || !items.value[index]) return

    const container = elementRef.value
    const item = items.value[index]

    // Pour un scroll horizontal, nous voulons que l'élément sélectionné
    // soit toujours à la position du premier élément
    if (config.type === 'horizontal') {
      const firstItemLeft = items.value[0]?.offsetLeft || 0
      const selectedItemLeft = item.offsetLeft

      // Calculer le décalage pour aligner l'élément sélectionné
      // avec la position du premier élément
      const scrollOffset = selectedItemLeft - firstItemLeft

      container.scrollTo({
        left: scrollOffset,
        behavior: 'smooth'
      })
    }
    // Pour les grilles et listes verticales
    else if (config.type === 'grid' || config.type === 'vertical') {
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

  // Watch pour le scroll automatique
  watch(activeIndex, (newIndex) => {
    if (isActive.value && newIndex >= 0) {
      scrollItemIntoView(newIndex)
    }
  })

  onMounted(() => {
    initializeZone()
  })

  onUnmounted(() => {
    if (initialized.value) {
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
  }
}