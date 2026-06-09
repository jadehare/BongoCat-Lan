<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { useElementSize } from '@vueuse/core'
import { message } from 'antdv-next'
import { isNil } from 'es-toolkit'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'

import type { LanRemoteClient } from '@/stores/lan'

import { useCatStore } from '@/stores/cat'
import { useModelStore } from '@/stores/model'
import { Live2d } from '@/utils/live2d'
import { getPressedModelDirs, getSupportedModelKey, pressModelKey } from '@/utils/modelInput'
import { clearObject } from '@/utils/shared'

const props = defineProps<{
  client: LanRemoteClient
  index: number
}>()

const catStore = useCatStore()
const modelStore = useModelStore()
const containerRef = ref<HTMLElement>()
const canvasRef = ref<HTMLCanvasElement>()
const modelSize = ref<{ width: number, height: number }>()
const pressedKeys = reactive<Record<string, string>>({})
const live2d = new Live2d()
const { width: containerWidth, height: containerHeight } = useElementSize(containerRef)
const renderStyle = computed(() => {
  const columns = 2
  const column = props.index % columns
  const row = Math.floor(props.index / columns)

  return {
    left: `${16 + column * 176}px`,
    bottom: `${16 + row * 176}px`,
  }
})
const overlayPaths = computed(() => Object.values(pressedKeys))

onUnmounted(() => {
  live2d.destroy()
})

function applyAxisValue(id: string, value: number) {
  const range = live2d.getParameterValueRange(id)

  if (!range) return

  const { min, max } = range

  live2d.setParameterValue(id, Math.max(min, value * max))
}

watch([canvasRef, containerRef], ([canvas, container]) => {
  if (!canvas || !container) return

  live2d.attach(canvas, container)
}, { immediate: true })

watch(() => modelStore.currentModel?.path, async (path) => {
  if (!path || !canvasRef.value || !containerRef.value) return

  try {
    const { width, height } = await live2d.load(path)

    modelSize.value = { width, height }
  } catch (error) {
    message.error(String(error))
  }
}, { immediate: true })

watch([containerWidth, containerHeight, modelSize], ([width, height, nextModelSize]) => {
  if (!nextModelSize || width <= 0 || height <= 0) return

  live2d.resizeModel(nextModelSize, { width, height })
}, { immediate: true })

function applyInputState() {
  const inputState = props.client.inputState

  clearObject([pressedKeys])

  for (const key of inputState.keyboardKeys) {
    const nextKey = getSupportedModelKey(key, modelStore.supportKeys)

    if (!nextKey) continue

    pressModelKey(pressedKeys, modelStore.supportKeys, nextKey)
  }

  for (const [name, value] of Object.entries(inputState.gamepadButtons)) {
    if (value <= 0 || name === 'LeftThumb' || name === 'RightThumb') continue

    pressModelKey(pressedKeys, modelStore.supportKeys, name)
  }

  const dirs = getPressedModelDirs(pressedKeys)
  const hasLeft = dirs.some(dir => dir.startsWith('left'))
  const hasRight = dirs.some(dir => dir.startsWith('right'))
  const leftStickX = inputState.gamepadAxes.LeftStickX ?? 0
  const leftStickY = inputState.gamepadAxes.LeftStickY ?? 0
  const rightStickX = inputState.gamepadAxes.RightStickX ?? 0
  const rightStickY = inputState.gamepadAxes.RightStickY ?? 0
  const leftThumb = (inputState.gamepadButtons.LeftThumb ?? 0) > 0
  const rightThumb = (inputState.gamepadButtons.RightThumb ?? 0) > 0
  const leftStickActive = leftThumb || leftStickX !== 0 || leftStickY !== 0
  const rightStickActive = rightThumb || rightStickX !== 0 || rightStickY !== 0

  live2d.setParameterValue('CatParamLeftHandDown', hasLeft || leftStickActive)
  live2d.setParameterValue('CatParamRightHandDown', hasRight || rightStickActive)
  live2d.setParameterValue('ParamMouseLeftDown', inputState.mouseButtons.includes('Left'))
  live2d.setParameterValue('ParamMouseRightDown', inputState.mouseButtons.includes('Right'))
  applyAxisValue('CatParamStickLX', leftStickX)
  applyAxisValue('CatParamStickLY', leftStickY)
  applyAxisValue('CatParamStickRX', rightStickX)
  applyAxisValue('CatParamStickRY', rightStickY)
  live2d.setParameterValue('CatParamStickShowLeftHand', leftStickActive)
  live2d.setParameterValue('CatParamStickShowRightHand', rightStickActive)
  live2d.setParameterValue('CatParamStickLeftDown', leftThumb)
  live2d.setParameterValue('CatParamStickRightDown', rightThumb)

  if (!inputState.cursor) return

  const screenWidth = window.screen.width || window.innerWidth || 1
  const screenHeight = window.screen.height || window.innerHeight || 1
  const xRatio = Math.max(0, Math.min(inputState.cursor.x / screenWidth, 1))
  const yRatio = Math.max(0, Math.min(inputState.cursor.y / screenHeight, 1))

  for (const id of [
    'ParamMouseX',
    'ParamMouseY',
    'ParamAngleX',
    'ParamAngleY',
    'ParamAngleZ',
    'ParamEyeBallX',
    'ParamEyeBallY',
  ]) {
    const range = live2d.getParameterValueRange(id)

    if (!range) continue

    const { min, max } = range

    if (isNil(min) || isNil(max)) continue

    const isXAxis = id.endsWith('X')
    const isYAxis = id.endsWith('Y')
    const isZAxis = id.endsWith('Z')

    let value: number

    if (isZAxis) {
      const dragX = 1 - 2 * xRatio
      const dragY = 1 - 2 * yRatio

      value = dragX * dragY * min
    } else {
      const ratio = isXAxis ? xRatio : yRatio

      value = max - ratio * (max - min)
    }

    if (!isYAxis && catStore.model.mouseMirror) {
      value *= -1
    }

    live2d.setParameterValue(id, value)
  }
}

watch(() => [
  props.client.inputState,
  modelStore.currentModel?.id,
  Object.keys(modelStore.supportKeys).length,
  catStore.model.mouseMirror,
], applyInputState, { deep: true, immediate: true })
</script>

<template>
  <div
    ref="containerRef"
    class="pointer-events-none absolute z-20 h-40 w-40 overflow-hidden border-white/20 bg-black/30 backdrop-blur-sm border rounded-3xl"
    :class="{ '-scale-x-100': catStore.model.mirror }"
    :style="renderStyle"
  >
    <div class="absolute left-2 top-2 z-30 max-w-[calc(100%-16px)] truncate bg-black/60 px-2 py-1 text-[11px] text-white rounded-full">
      {{ props.client.nickname }}
    </div>

    <canvas
      ref="canvasRef"
      class="absolute inset-0 h-full w-full"
    />

    <img
      v-for="path in overlayPaths"
      :key="path"
      class="absolute inset-0 h-full w-full object-contain"
      :src="convertFileSrc(path)"
    >
  </div>
</template>
