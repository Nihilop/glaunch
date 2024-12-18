import {type RemovableRef} from "@vueuse/core";

export interface SteamProfile {
  steamid: string
  personaname: string
  profileurl: string
  avatar: string
  avatarmedium: string
  avatarfull: string
  personastate: number
  communityvisibilitystate: number
  profilestate: number
  lastlogoff: number
  commentpermission: number
  realname?: string
  primaryclanid?: string
  timecreated?: number
  gameid?: string
  gameserverip?: string
  gameextrainfo?: string
  loccountrycode?: string
  locstatecode?: string
  loccityid?: number
}

export interface BattleNetProfile {
  id: string
  battletag: string
  // Ajoutez d'autres champs selon vos besoins
}

export interface EpicProfile {
  id: string
  displayName: string
  // Ajoutez d'autres champs selon vos besoins
}

export interface PlatformState<T> {
  id: string | null
  token: any | null
  profile: T | null
  loading: boolean
  error: string | null
}