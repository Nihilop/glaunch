import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'

export interface EpicProfile {
  id: string
  displayName: string
  preferredLanguage?: string
  email?: string
  avatar?: string
}

export interface EpicFriend {
  accountId: string
  displayName: string
  status: string
  presence?: {
    status: string
    game?: string
  }
}

export function useEpicApi() {
  async function startAuth() {
    const authUrl = await invoke<string>('auth_epic')
    await open(authUrl)
  }

  async function exchangeCode(code: string) {
    return await invoke<string>('exchange_epic_code', { code })
  }

  async function getProfile(token: string, accountId: string): Promise<EpicProfile> {
    const response = await invoke<string>('get_epic_profile', { token, accountId })
    const data = JSON.parse(response)
    if (!data) {
      throw new Error('Failed to fetch Epic profile')
    }
    return data
  }

  async function getFriends(token: string): Promise<EpicFriend[]> {
    const response = await invoke<string>('get_epic_friends', { token })
    const data = JSON.parse(response)
    if (!data) {
      throw new Error('Failed to fetch Epic friends')
    }
    return data
  }

  return {
    startAuth,
    exchangeCode,  // Ajout de exchangeCode
    getProfile,
    getFriends
  }
}