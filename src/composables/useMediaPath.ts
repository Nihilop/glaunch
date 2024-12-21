import { convertFileSrc } from '@tauri-apps/api/core'
import {appDataDir, resolveResource} from '@tauri-apps/api/path'

export function useMediaPath() {
  // Cache pour éviter de résoudre plusieurs fois le même chemin
  const pathCache = new Map<string, string>()


  const resolveMediaPath = async (path: string | null): Promise<string> => {
    if (!path) return '/placeholder-game.png'
    const appDataDirPath = await appDataDir()
    console.log(appDataDirPath)
    console.log(path)
    // Vérifier le cache
    if (pathCache.has(path)) {
      return pathCache.get(path)!
    }

    try {
      if (path.startsWith('media/')) {
        const convertedPath = convertFileSrc(appDataDirPath + '/' + path)

        // Mettre en cache
        pathCache.set(path, convertedPath)
        return convertedPath
      }

      const convertedPath = convertFileSrc(path)
      pathCache.set(path, convertedPath)
      return convertedPath
    } catch (error) {
      console.error('Failed to resolve media path:', error)
      return '/placeholder-game.png'
    }
  }

  return {
    resolveMediaPath
  }
}