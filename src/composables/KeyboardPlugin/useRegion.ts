import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useNavigationStore } from './store'
import type { RegionConfig } from './types'

export function useRegion(config: Omit<RegionConfig, 'id'>) {
  const store = useNavigationStore()
  const initialized = ref(false)

  // Créer un ID unique pour la région
  const regionId = `region-${Math.random().toString(36).slice(2, 11)}`

  // Computed
  const isActive = computed(() => store.activeRegion?.id === regionId)
  const activeZone = computed(() => {
    if (!isActive.value) return null
    return store.activeZone
  })

  const zones = computed(() => {
    const region = store.regions.get(regionId)
    return region ? Array.from(region.zones.values()) : []
  })

  // Initialiser la région
  const initializeRegion = () => {
    if (initialized.value) return

    store.registerRegion({
      ...config,
      id: regionId,
      priority: config.priority || 0,
      persistent: config.persistent || false
    })

    initialized.value = true

    // Activer automatiquement si nécessaire
    if (store.regions.size === 1 || config.persistent) {
      activate()
    }
  }

  // Activer la région
  const activate = () => {
    if (!initialized.value) {
      initializeRegion()
    }

    store.setActiveRegion(regionId)

    // Si une zone par défaut est définie et qu'elle existe, l'activer
    if (config.defaultZone) {
      const region = store.regions.get(regionId)
      if (region?.zones.has(config.defaultZone)) {
        store.setActiveZone(config.defaultZone)
      }
    }
  }

  // Désactiver la région
  const deactivate = () => {
    if (isActive.value) {
      store.setActiveRegion(null)
    }
  }

  // Obtenir la région suivante/précédente
  const getNextRegion = (direction: 'prev' | 'next'): string | null => {
    const regions = Array.from(store.regions.values())
      .filter(r => !r.persistent || r.id === regionId) // Filtrer les régions persistantes sauf la courante
      .sort((a, b) => a.priority - b.priority)

    const currentIndex = regions.findIndex(r => r.id === regionId)
    if (currentIndex === -1) return null

    const nextIndex = direction === 'next'
      ? (currentIndex + 1) % regions.length
      : (currentIndex - 1 + regions.length) % regions.length

    return regions[nextIndex].id
  }

  // Watch pour les changements de zones
  watch(zones, (newZones) => {
    // Si la région est active et qu'une zone par défaut est définie
    if (isActive.value && config.defaultZone && !store.activeZone) {
      if (newZones.some(zone => zone.id === config.defaultZone)) {
        store.setActiveZone(config.defaultZone)
      }
    }
  })

  onMounted(() => {
    initializeRegion()
  })

  onUnmounted(() => {
    if (initialized.value && !config.persistent) {
      store.unregisterRegion(regionId)
    }
  })

  return {
    regionId,
    isActive,
    activeZone,
    zones,
    activate,
    deactivate,
    getNextRegion,
  }
}