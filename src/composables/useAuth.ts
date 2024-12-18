import { useAuthStore } from '@/stores/auth'
import { useSteamApi, useBattleNetApi, useEpicApi } from '@/api'
import { onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { storeToRefs } from 'pinia'

export function useAuth() {
  const authStore = useAuthStore()
  const steamApi = useSteamApi()
  const battleNetApi = useBattleNetApi()
  const epicApi = useEpicApi()

  // Extraire les refs réactives du store
  const {
    steam,
    steamProfile,
    battlenet,
    battlenetProfile,
    epic,
    epicProfile
  } = storeToRefs(authStore)

  async function initializeDeepLinkHandlers() {
    await listen('deep-link', (event: any) => {
      const url = event.payload
      if (url.startsWith('glaunch://auth/steam')) {
        handleSteamCallback(url)
      } else if (url.startsWith('glaunch://auth/epic')) {
        handleEpicCallback(url)
      } else if (url.startsWith('glaunch://auth/battlenet')) {
        handleBattlenetCallback(url)
      }
    })
  }

  async function initializeSteamAuth() {
    if (steam.value.id) {
      try {
        authStore.setSteamLoading(true)
        const profile = await steamApi.getProfile(steam.value.id)
        authStore.setSteamProfile(profile)
      } catch (error) {
        authStore.setSteamError(error instanceof Error ? error.message : 'Error fetching profile')
      } finally {
        authStore.setSteamLoading(false)
      }
    }
  }

  async function initializeBattlenetAuth() {
    if (battlenet.value.token) {
      try {
        authStore.setBattlenetLoading(true)
        const profile = await battleNetApi.getProfile(battlenet.value.token)
        authStore.setBattleNetProfile(profile)
      } catch (error) {
        authStore.setBattlenetError(error instanceof Error ? error.message : 'Error fetching profile')
      } finally {
        authStore.setBattlenetLoading(false)
      }
    }
  }

  async function initializeEpicAuth() {
    if (epic.value.token) {
      try {
        authStore.setEpicLoading(true)
        const profile = await epicApi.getProfile(epic.value.token, epic.value.id)
        authStore.setEpicProfile(profile)
      } catch (error) {
        authStore.setEpicError(error instanceof Error ? error.message : 'Error fetching profile')
      } finally {
        authStore.setEpicLoading(false)
      }
    }
  }




  async function handleSteamCallback(url: string) {
    try {
      const urlObj = new URL(url)
      const identity = urlObj.searchParams.get('openid.identity')

      if (identity) {
        const steamIdMatch = identity.match(/\/id\/(\d+)$/)
        if (steamIdMatch) {
          const id = steamIdMatch[1]
          authStore.setSteamId(id)
          const profile = await steamApi.getProfile(id)
          authStore.setSteamProfile(profile)
        }
      }
    } catch (error) {
      console.error('Error in Steam callback:', error)
      authStore.setSteamError(error instanceof Error ? error.message : 'Unknown error')
    }
  }

  async function handleEpicCallback(url: string) {
    try {
      const urlObj = new URL(url)
      const code = urlObj.searchParams.get('code')

      if (code) {
        authStore.setEpicLoading(true)

        // 1. D'abord échanger le code contre un token
        const token = await epicApi.exchangeCode(code)
        const authData = JSON.parse(token)
        authStore.setEpicToken(authData.access_token)
        authStore.setEpicId(authData.account_id)

        // 2. Utiliser le token pour obtenir le profil
        const profile = await epicApi.getProfile(authData.access_token, authData.account_id)
        authStore.setEpicProfile(profile)
      }
    } catch (error) {
      console.error('Error in Epic callback:', error)
      authStore.setEpicError(error instanceof Error ? error.message : 'Unknown error')
    } finally {
      authStore.setEpicLoading(false)
    }
  }

  async function handleBattlenetCallback(url: string) {
    try {
      const urlObj = new URL(url)
      const code = urlObj.searchParams.get('code')

      if (code) {
        authStore.setBattlenetLoading(true)

        // 1. Échanger le code contre un token
        const tokenData = await battleNetApi.exchangeCode(code)
        authStore.setBattlenetToken(tokenData.access_token)

        // 2. Utiliser le token pour obtenir le profil
        const profile = await battleNetApi.getProfile(tokenData.access_token)
        authStore.setBattleNetProfile(profile)
      }
    } catch (error) {
      console.error('Error in Battle.net callback:', error)
      authStore.setBattlenetError(error instanceof Error ? error.message : 'Unknown error')
    } finally {
      authStore.setBattlenetLoading(false)
    }
  }

  async function startSteamAuth() {
    authStore.setSteamLoading(true)
    try {
      await steamApi.startAuth()
    } catch (error) {
      authStore.setSteamError(error instanceof Error ? error.message : 'Error starting Steam auth')
    } finally {
      authStore.setSteamLoading(false)
    }
  }

  onMounted(async () => {
    await initializeDeepLinkHandlers()
    await initializeSteamAuth()
    await initializeBattlenetAuth()
    await initializeEpicAuth()
  })

  return {
    // États
    steam,
    steamProfile,
    battlenet,
    battlenetProfile,
    epic,
    epicProfile,

    // Actions Steam
    startSteamAuth,
    disconnectSteam: () => authStore.clearSteamAuth(),

    // Actions Battle.net
    startBattleNetAuth: battleNetApi.startAuth,
    disconnectBattleNet: () => authStore.clearBattleNetAuth(),

    // Actions Epic
    startEpicAuth: epicApi.startAuth,
    disconnectEpic: () => authStore.clearEpicAuth(),

    // Actions globales
    disconnectAll: () => authStore.clearAllAuth()
  }
}