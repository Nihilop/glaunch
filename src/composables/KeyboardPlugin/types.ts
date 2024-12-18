import type { Ref } from 'vue'

export interface Bounds {
  top: number
  right: number
  bottom: number
  left: number
}

export type Direction = 'up' | 'down' | 'left' | 'right'
export type ZoneType = 'grid' | 'horizontal' | 'vertical'

export interface NavigationError {
  zone: string
  index: number
  timestamp: number
}

export interface NavigationHistory {
  region: string
  zone: string
  index: number
  timestamp: number
}

export interface RegionConfig {
  id: string
  priority: number
  persistent?: boolean
  defaultZone?: string
  onEnter?: () => void
  onLeave?: () => void
}

export interface ZoneConfig {
  id: string
  type: ZoneType
  regionId: string
  columns?: number
  memory?: boolean
  group?: string
  onSelect?: (index: number) => void
  onFocus?: (index: number) => void
  onBlur?: (index: number) => void
}

export interface NavigableZone extends ZoneConfig {
  ref: HTMLElement | null
  items: HTMLElement[]
  bounds: Bounds | null
}

export interface NavigableRegion {
  id: string
  priority: number
  persistent: boolean
  defaultZone?: string
  zones: Map<string, NavigableZone>
  onEnter?: () => void
  onLeave?: () => void
}

export interface NavigationState {
  activeRegion: string | null
  activeZone: string | null
  activeIndex: number
  lastError: NavigationError | null
}

export interface RegisterZoneOptions extends ZoneConfig {
  ref: Ref<HTMLElement | null>
}

export interface NavigationResult {
  success: boolean
  newIndex?: number
  switchZone?: boolean
  switchRegion?: boolean
  targetZone?: string
  targetRegion?: string
}

export interface KeyboardNavigationPlugin {
  regions: Map<string, NavigableRegion>
  zones: Map<string, NavigableZone>
  state: NavigationState
  history: NavigationHistory[]
  debug: boolean
}

export interface NavigationMemoryState {
  regionId: string
  zoneId: string
  index: number
}