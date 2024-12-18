import { defineStore } from 'pinia'
import { useStorage } from '@vueuse/core'
import { ref, computed } from 'vue'
import type { SteamProfile, BattleNetProfile, EpicProfile, PlatformState } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  // State
  const steam = ref<PlatformState<SteamProfile>>({
    id: useStorage('steam_id', null),
    token: null,
    profile: null,
    loading: false,
    error: null
  })

  const battlenet = ref<PlatformState<BattleNetProfile>>({
    id: null,
    token: useStorage('battlenet_id', null),
    profile: null,
    loading: false,
    error: null
  })

  const epic = ref<PlatformState<EpicProfile>>({
    id: useStorage('epic_client_id', null),
    token: useStorage('epic_id', null),
    profile: null,
    loading: false,
    error: null
  })

  // Getters
  const isSteamConnected = computed(() => !!steam.value.id)
  const isBattleNetConnected = computed(() => !!battlenet.value.token)
  const isEpicConnected = computed(() => !!epic.value.token)

  const steamProfile = computed(() => steam.value.profile)
  const battlenetProfile = computed(() => battlenet.value.profile)
  const epicProfile = computed(() => epic.value.profile)

  // Actions

  function setBattlenetToken(token: string | null) {
    battlenet.value.token = token
  }

  function setEpicToken(token: string | null) {
    epic.value.token = token
  }

  function setSteamId(id: string | null) {
    steam.value.id = id
  }

  function setSteamProfile(profile: SteamProfile | null) {
    steam.value.profile = profile
  }

  function setSteamLoading(loading: boolean) {
    steam.value.loading = loading
  }
  function setEpicLoading(loading: boolean) {
    epic.value.loading = loading
  }
  function setBattlenetLoading(loading: boolean) {
    battlenet.value.loading = loading
  }

  function setSteamError(error: string | null) {
    steam.value.error = error
  }

  function setEpicError(error: string | null) {
    epic.value.error = error
  }

  function setBattlenetError(error: string | null) {
    battlenet.value.error = error
  }

  function clearSteamAuth() {
    steam.value = {
      id: null,
      profile: null,
      loading: false,
      error: null
    }
  }

  function setBattleNetId(id: string | null) {
    battlenet.value.id = id
  }

  function setBattleNetProfile(profile: BattleNetProfile | null) {
    battlenet.value.profile = profile
  }



  function clearBattleNetAuth() {
    battlenet.value = {
      id: null,
      token: null,  // Ajout
      profile: null,
      loading: false,
      error: null
    }
  }

  function setEpicId(id: string | null) {
    epic.value.id = id
  }

  function setEpicProfile(profile: EpicProfile | null) {
    epic.value.profile = profile
  }

  function clearEpicAuth() {
    epic.value = {
      id: null,
      token: null,  // Ajout
      profile: null,
      loading: false,
      error: null
    }
  }

  function clearAllAuth() {
    clearSteamAuth()
    clearBattleNetAuth()
    clearEpicAuth()
  }

  return {
    // State
    steam,
    battlenet,
    epic,

    // Getters
    isSteamConnected,
    isBattleNetConnected,
    isEpicConnected,
    steamProfile,
    battlenetProfile,
    epicProfile,

    // Actions
    setSteamId,
    setSteamProfile,
    setSteamLoading,
    setSteamError,
    clearSteamAuth,
    setBattlenetToken,
    setEpicToken,
    setBattleNetId,
    setBattleNetProfile,
    clearBattleNetAuth,
    setEpicId,
    setEpicProfile,
    clearEpicAuth,
    clearAllAuth,
    setEpicLoading,
    setBattlenetLoading,
    setEpicError,
    setBattlenetError,
  }
})