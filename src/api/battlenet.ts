import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'

export interface BattleNetProfile {
  id: string
  battletag: string
  accountId: string
}

export interface BattleNetToken {
  access_token: string
  token_type: string
  expires_in: number
}

export function useBattleNetApi() {
  async function startAuth() {
    const authUrl = await invoke<string>('auth_battlenet')
    await open(authUrl)
  }

  async function exchangeCode(code: string): Promise<BattleNetToken> {
    const response = await invoke<string>('auth_battlenet_callback', { code })
    const data = JSON.parse(response)
    if (!data.access_token) {
      throw new Error('Failed to exchange Battle.net code')
    }
    return data
  }

  async function getProfile(token: string): Promise<BattleNetProfile> {
    const response = await invoke<string>('get_battlenet_profile', { token })
    const data = JSON.parse(response)
    if (!data) {
      throw new Error('Failed to fetch Battle.net profile')
    }
    return data
  }

  return {
    startAuth,
    exchangeCode,
    getProfile
  }
}