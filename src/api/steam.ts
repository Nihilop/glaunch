import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'

export interface SteamProfile {
  steamid: string
  personaname: string
  profileurl: string
  avatar: string
  avatarmedium: string
  avatarfull: string
  personastate: number
  gameextrainfo?: string
}

export interface SteamFriend {
  steamid: string
  personaname: string
  profileurl: string
  avatar: string
  personastate: number
  gameextrainfo?: string
}

export function useSteamApi() {
  async function startAuth() {
    // Utilise la nouvelle commande auth_steam
    const authUrl = await invoke<string>('auth_steam')
    await open(authUrl)
  }

  async function getProfile(steamId: string): Promise<SteamProfile> {
    const response = await invoke<string>('get_steam_profile', { steamId })
    const data = JSON.parse(response)
    if (!data) {
      throw new Error('Failed to fetch Steam profile')
    }
    return data
  }

  async function getFriends(steamId: string): Promise<SteamFriend[]> {
    const response = await invoke<string>('get_steam_friends', { steamId })
    const data = JSON.parse(response)
    if (!data) {
      throw new Error('Failed to fetch Steam friends')
    }
    return data
  }

  return {
    startAuth,
    getProfile,
    getFriends
  }
}