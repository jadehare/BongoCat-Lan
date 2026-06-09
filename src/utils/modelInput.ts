import { sep } from '@tauri-apps/api/path'
import { findKey, nth } from 'es-toolkit/compat'

export function getSupportedModelKey(key: string, supportKeys: Record<string, string>) {
  let nextKey = key
  const unsupportedKey = !supportKeys[nextKey]

  if (key.startsWith('F') && unsupportedKey) {
    nextKey = key.replace(/F(\d+)/, 'Fn')
  }

  for (const item of ['Meta', 'Shift', 'Alt', 'Control']) {
    if (key.startsWith(item) && unsupportedKey) {
      const regex = new RegExp(`^(${item}).*`)

      nextKey = key.replace(regex, '$1')
    }
  }

  return nextKey
}

export function pressModelKey(
  pressedKeys: Record<string, string>,
  supportKeys: Record<string, string>,
  key: string,
) {
  const path = supportKeys[key]

  if (!path) return

  const dirName = nth(path.split(sep()), -2)

  if (!dirName) return

  const prevKey = findKey(pressedKeys, value => value.includes(dirName))

  if (prevKey) {
    delete pressedKeys[prevKey]
  }

  pressedKeys[key] = path
}

export function releaseModelKey(pressedKeys: Record<string, string>, key: string) {
  delete pressedKeys[key]
}

export function getPressedModelDirs(pressedKeys: Record<string, string>) {
  return Object.values(pressedKeys).map((path) => {
    return nth(path.split(sep()), -2) ?? ''
  })
}
