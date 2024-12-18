import { defineStore } from 'pinia'
import { getCurrentInstance, type App } from 'vue'
import { type Router } from 'vue-router'
import { ref, computed } from 'vue'
import type {
  NavigableRegion,
  NavigableZone,
  NavigationState,
  NavigationHistory,
  RegionConfig,
  RegisterZoneOptions,
  NavigationError, Direction, Bounds, NavigationMemoryState
} from './types'
import {useSound} from "./useSound.ts";

export const useNavigationStore = defineStore('navigation', () => {
  const { playSound } = useSound()
  // State
  const regions = ref<Map<string, NavigableRegion>>(new Map())
  const zones = ref<Map<string, NavigableZone>>(new Map())
  const history = ref<NavigationHistory[]>([])
  const navigationMemory = ref<NavigationMemoryState | null>(null)
  const lastMemoryPath = ref<string | null>(null)
  const pendingRestore = ref<{
    path: string
    attempts: number
    maxAttempts: number
    timeoutId?: number
  } | null>(null)

  const debug = ref(false)

  const state = ref<NavigationState>({
    activeRegion: null,
    activeZone: null,
    activeIndex: 0,
    lastError: null
  })

  // Getters
  const activeRegion = computed(() =>
    state.value.activeRegion ? regions.value.get(state.value.activeRegion) : null
  )

  const activeZone = computed(() =>
    state.value.activeZone ? zones.value.get(state.value.activeZone) : null
  )

  const activeIndex = computed(() => state.value.activeIndex)

  const isDebugEnabled = computed(() => debug.value)

  // Memory
  function queueStateRestore(routePath: string) {

    // Annuler toute restauration en attente
    if (pendingRestore.value?.timeoutId) {
      clearTimeout(pendingRestore.value.timeoutId)
    }

    pendingRestore.value = {
      path: routePath,
      attempts: 0,
      maxAttempts: 10 // Maximum de tentatives
    }

    attemptStateRestore()
  }

  function attemptStateRestore() {
    if (!pendingRestore.value || !navigationMemory.value) return

    const { path, attempts, maxAttempts } = pendingRestore.value

    // Vérifier si la zone existe maintenant
    const memorizedZone = zones.value.get(navigationMemory.value.zoneId)

    if (memorizedZone) {
      restoreState(path)
      pendingRestore.value = null
      return
    }

    // Si on a atteint le max de tentatives, abandonner
    if (attempts >= maxAttempts) {
      pendingRestore.value = null
      return
    }

    // Réessayer dans 100ms
    pendingRestore.value = {
      ...pendingRestore.value,
      attempts: attempts + 1,
      timeoutId: setTimeout(attemptStateRestore, 100) as unknown as number
    }
  }

  function saveState(routePath: string) {

    // 1. Validation des prérequis
    if (!routePath) {
      return false
    }

    if (!state.value.activeZone) {
      return false
    }

    if (!state.value.activeRegion) {
      return false
    }

    // 2. Vérification de la zone
    const currentZone = zones.value.get(state.value.activeZone)

    if (!currentZone) {
      return false
    }

    if (!currentZone.memory) {
      return false
    }

    // 3. Sauvegarde de l'état
    const memoryState: NavigationMemoryState = {
      regionId: state.value.activeRegion,
      zoneId: state.value.activeZone,
      index: state.value.activeIndex
    }

    navigationMemory.value = memoryState
    lastMemoryPath.value = routePath


    return true
  }

  function restoreState(routePath: string) {

    if (!navigationMemory.value || lastMemoryPath.value !== routePath) {
      return false
    }

    const memorizedZone = zones.value.get(navigationMemory.value.zoneId)
    if (!memorizedZone) {
      queueStateRestore(routePath)
      return false
    }

    // Reste de la logique de restauration...
    state.value = {
      ...state.value,
      activeRegion: navigationMemory.value.regionId,
      activeZone: navigationMemory.value.zoneId,
      activeIndex: navigationMemory.value.index
    }
    return true
  }

  // Nettoyer au unmount
  function cleanup() {
    if (pendingRestore.value?.timeoutId) {
      clearTimeout(pendingRestore.value.timeoutId)
      pendingRestore.value = null
    }
  }


  // navigate
  function navigate(direction: Direction) {
    if (debug.value) {
      console.log(`[Navigation] Navigating ${direction}`)
    }

    // Trouver la zone courante
    const currentZone = activeZone.value
    if (!currentZone) {
      console.log('[Navigation] No active zone')
      return false
    }

    const currentIndex = state.value.activeIndex

    if (debug.value) {
      console.log(`[Navigation] Current zone: ${currentZone.id}, index: ${currentIndex}`)
    }



    // Calcul du nouvel index basé sur la direction
    switch (currentZone.type) {
      case 'horizontal':
        if (direction === 'left' && currentIndex > 0) {
          setActiveZone(currentZone.id, currentIndex - 1)
          return true
        }
        if (direction === 'right' && currentIndex < currentZone.items.length - 1) {
          setActiveZone(currentZone.id, currentIndex + 1)
          return true
        }
        break

      case 'vertical':
        if (direction === 'up' && currentIndex > 0) {
          setActiveZone(currentZone.id, currentIndex - 1)
          return true
        }
        if (direction === 'down' && currentIndex < currentZone.items.length - 1) {
          setActiveZone(currentZone.id, currentIndex + 1)
          return true
        }
        break

      case 'grid':
        const columns = currentZone.columns || 1
        const totalItems = currentZone.items.length

        switch (direction) {
          case 'left':
            if (currentIndex % columns > 0) {
              setActiveZone(currentZone.id, currentIndex - 1)
              return true
            }
            break

          case 'right':
            if (currentIndex % columns < columns - 1 && currentIndex + 1 < totalItems) {
              setActiveZone(currentZone.id, currentIndex + 1)
              return true
            }
            break

          case 'up':
            if (currentIndex - columns >= 0) {
              setActiveZone(currentZone.id, currentIndex - columns)
              return true
            }
            break

          case 'down':
            if (currentIndex + columns < totalItems) {
              setActiveZone(currentZone.id, currentIndex + columns)
              return true
            }
            break
        }
        break
    }

    // Si on n'a pas pu naviguer dans la zone actuelle,
    // essayer de trouver une zone adjacente
    const nextZone = findAdjacentZone(currentZone, direction)
    if (nextZone) {
      setActiveZone(nextZone.id, 0)
      return true
    }

    setError({
      zone: currentZone.id,
      index: currentIndex,
      timestamp: Date.now()
    })

    return false
  }

  // utils

  function findAdjacentZone(currentZone: NavigableZone, direction: Direction): NavigableZone | null {
    if (!currentZone.bounds) return null

    const currentBounds = currentZone.bounds
    return Array.from(zones.value.values())
      .filter(zone => {
        if (!zone.bounds || zone.id === currentZone.id) return false
        if (zone.regionId !== currentZone.regionId) return false

        const targetBounds = zone.bounds
        const overlap = calculateOverlap(currentBounds, targetBounds, direction)

        switch (direction) {
          case 'up':
            return targetBounds.bottom <= currentBounds.top && overlap > 0
          case 'down':
            return targetBounds.top >= currentBounds.bottom && overlap > 0
          case 'left':
            return targetBounds.right <= currentBounds.left && overlap > 0
          case 'right':
            return targetBounds.left >= currentBounds.right && overlap > 0
        }
      })
      .sort((a, b) => {
        const distA = getDistance(currentBounds, a.bounds!, direction)
        const distB = getDistance(currentBounds, b.bounds!, direction)
        return distA - distB
      })[0] || null
  }

  function calculateOverlap(bounds1: Bounds, bounds2: Bounds, direction: Direction): number {
    if (direction === 'up' || direction === 'down') {
      return Math.min(bounds1.right, bounds2.right) - Math.max(bounds1.left, bounds2.left)
    }
    return Math.min(bounds1.bottom, bounds2.bottom) - Math.max(bounds1.top, bounds2.top)
  }

  function getDistance(bounds1: Bounds, bounds2: Bounds, direction: Direction): number {
    switch (direction) {
      case 'up':
        return bounds1.top - bounds2.bottom
      case 'down':
        return bounds2.top - bounds1.bottom
      case 'left':
        return bounds1.left - bounds2.right
      case 'right':
        return bounds2.left - bounds1.right
    }
  }

  // Actions
  function registerRegion(config: RegionConfig) {
    if (debug.value) {
      console.log(`[Navigation] Registering region: ${config.id}`)
    }

    regions.value.set(config.id, {
      id: config.id,
      priority: config.priority,
      persistent: config.persistent ?? false,
      defaultZone: config.defaultZone,
      zones: new Map(),
      onEnter: config.onEnter,
      onLeave: config.onLeave
    })

    // Si c'est la première région, la définir comme active
    if (regions.value.size === 1) {
      state.value.activeRegion = config.id
    }
  }

  function unregisterRegion(regionId: string) {
    if (debug.value) {
      console.log(`[Navigation] Unregistering region: ${regionId}`)
    }

    const region = regions.value.get(regionId)
    if (!region) return

    // Unregister all zones in the region first
    Array.from(region.zones.values()).forEach(zone => {
      unregisterZone(zone.id)
    })

    // Remove the region
    regions.value.delete(regionId)

    // If this was the active region, reset the active state
    if (state.value.activeRegion === regionId) {
      state.value.activeRegion = null
      state.value.activeZone = null
      state.value.activeIndex = 0

      // Try to activate the next available region
      const nextRegion = Array.from(regions.value.values())
        .sort((a, b) => a.priority - b.priority)[0]

      if (nextRegion) {
        setActiveRegion(nextRegion.id)
      }
    }
  }

  function registerZone(options: RegisterZoneOptions) {
    if (debug.value) {
      console.log(`[Navigation] Registering zone: ${options.id} in region: ${options.regionId}`)
      logNavigationState()
    }

    const region = regions.value.get(options.regionId)
    if (!region) {
      throw new Error(`Region ${options.regionId} not found`)
    }

    const zone: NavigableZone = {
      id: options.id,
      type: options.type,
      regionId: options.regionId,
      ref: options.ref.value,
      items: [],
      bounds: null,
      columns: options.columns,
      memory: options.memory,
      group: options.group,
      onSelect: options.onSelect,
      onFocus: options.onFocus,
      onBlur: options.onBlur
    }

    zones.value.set(options.id, {
      ...options,
      memory: !!options.memory,
      ...zone
    })
    region.zones.set(options.id, zone)

    // Si c'est la première zone de la région et que la région est active
    if (region.zones.size === 1 && state.value.activeRegion === options.regionId) {
      state.value.activeZone = options.id
    }
  }

  function unregisterZone(zoneId: string) {
    const zone = zones.value.get(zoneId)
    if (!zone) return

    if (debug.value) {
      console.log(`[Navigation] Unregistering zone: ${zoneId}`)
    }

    const region = regions.value.get(zone.regionId)
    if (region) {
      region.zones.delete(zoneId)
    }

    zones.value.delete(zoneId)

    // Si c'était la zone active, réinitialiser
    if (state.value.activeZone === zoneId) {
      state.value.activeZone = null
      state.value.activeIndex = 0
    }
  }

  function setActiveRegion(regionId: string | null) {
    if (debug.value) {
      console.log(`[Navigation] Setting active region: ${regionId}`)
    }

    const oldRegion = state.value.activeRegion ? regions.value.get(state.value.activeRegion) : null
    const newRegion = regionId ? regions.value.get(regionId) : null

    // Appeler les callbacks
    oldRegion?.onLeave?.()
    newRegion?.onEnter?.()

    state.value.activeRegion = regionId

    // Si la nouvelle région a une zone par défaut, l'activer
    if (newRegion?.defaultZone) {
      state.value.activeZone = newRegion.defaultZone
      state.value.activeIndex = 0
    }

    // Ajouter à l'historique
    addToHistory()
  }

  function setActiveZone(zoneId: string | null, index = 0) {
    if (debug.value) {
      if (typeof zoneId === "string") {
        console.log(`[Navigation] Setting active zone: ${zoneId} at index: ${index}`, {
          currentItems: zones.value.get(zoneId)?.items.length
        })
      }
      logNavigationState()
    }

    const oldZone = state.value.activeZone ? zones.value.get(state.value.activeZone) : null
    const newZone = zoneId ? zones.value.get(zoneId) : null

    // Appeler les callbacks
    oldZone?.onBlur?.(state.value.activeIndex)
    newZone?.onFocus?.(index)

    playSound('move')

    state.value.activeZone = zoneId
    state.value.activeIndex = index

    // Ajouter à l'historique
    addToHistory()
  }

  function setError(error: NavigationError) {
    state.value.lastError = error
    playSound('error')

    if (debug.value) {
      console.error('[Navigation] Error:', error)
    }

    // Nettoyer l'erreur après 500ms
    setTimeout(() => {
      if (state.value.lastError?.timestamp === error.timestamp) {
        state.value.lastError = null
      }
    }, 500)
  }

  function addToHistory() {
    if (!state.value.activeRegion || !state.value.activeZone) return

    history.value.push({
      region: state.value.activeRegion,
      zone: state.value.activeZone,
      index: state.value.activeIndex,
      timestamp: Date.now()
    })

    // Garder uniquement les 100 dernières entrées
    if (history.value.length > 100) {
      history.value.shift()
    }
  }

  function toggleDebug() {
    debug.value = !debug.value
  }

  function handleSelect() {
    const zone = activeZone.value
    if (!zone || !zone.onSelect) return false

    playSound('select')
    zone.onSelect(state.value.activeIndex)
    return true
  }

  function logNavigationState() {
    console.group('🗺️ Navigation State')

    console.group('Regions:')
    regions.value.forEach((region, id) => {
      console.log(`📍 Region: ${id}`, {
        isActive: state.value.activeRegion === id,
        zonesCount: region.zones.size,
        priority: region.priority,
        persistent: region.persistent
      })

      console.group('Zones:')
      region.zones.forEach((zone, zoneId) => {
        console.log(`🎯 Zone: ${zoneId}`, {
          type: zone.type,
          itemsCount: zone.items?.length || 0,
          isActive: state.value.activeZone === zoneId,
          activeIndex: state.value.activeZone === zoneId ? state.value.activeIndex : null,
          memory: !!zone.memory // Ajout de l'info memory
        })
      })
      console.groupEnd()
    })
    console.groupEnd()

    if (activeZone.value) {
      console.log('🎯 Active Zone Details:', {
        id: activeZone.value.id,
        type: activeZone.value.type,
        items: activeZone.value.items?.length || 0,
        currentIndex: state.value.activeIndex,
        memory: !!activeZone.value.memory // Ajout de l'info memory
      })
    }

    // Logs memory si présent
    if (navigationMemory.value) {
      console.group('💾 Memory State:')
      console.log('Path:', lastMemoryPath.value)
      console.log('Region:', navigationMemory.value.regionId)
      console.log('Zone:', navigationMemory.value.zoneId)
      console.log('Index:', navigationMemory.value.index)
      console.groupEnd()
    }

    console.groupEnd()
  }

  function setActiveElement(zoneId: string, index: number) {
    const zone = zones.value.get(zoneId);
    if (!zone) {
      console.warn(`[Navigation] Zone ${zoneId} not found`);
      return false;
    }

    // Vérifier que l'index est valide
    if (index < 0 || index >= zone.items.length) {
      console.warn(`[Navigation] Invalid index ${index} for zone ${zoneId}`);
      return false;
    }

    // Si la zone n'est pas dans la région active, activer sa région
    if (state.value.activeRegion !== zone.regionId) {
      setActiveRegion(zone.regionId);
    }

    // Activer la zone et l'index
    setActiveZone(zoneId, index);
    return true;
  }

  return {
    // Memory usage
    saveState,
    restoreState,
    cleanup,

    // State
    regions,
    zones,
    state,
    history,
    debug,

    // Getters
    activeRegion,
    activeZone,
    isDebugEnabled,
    activeIndex,

    // Actions
    registerRegion,
    unregisterRegion,
    registerZone,
    unregisterZone,
    setActiveRegion,
    setActiveZone,
    setError,
    toggleDebug,
    handleSelect,
    navigate,
    setActiveElement
  }
})