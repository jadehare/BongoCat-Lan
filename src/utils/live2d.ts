import type { MotionInfo } from 'easy-live2d'

import { convertFileSrc } from '@tauri-apps/api/core'
import { readDir, readTextFile } from '@tauri-apps/plugin-fs'
import { Config, CubismSetting, Live2DSprite, Priority } from 'easy-live2d'
import { groupBy } from 'es-toolkit/compat'
import JSON5 from 'json5'
import { Application, Ticker } from 'pixi.js'

import { i18n } from '@/locales'

import { join } from './path'

Config.MouseFollow = false

export interface ModelSize {
  width: number
  height: number
}

type ResizeTarget = Window | HTMLElement

export class Live2d {
  private app: Application | null = null
  public model: Live2DSprite | null = null
  private view: HTMLCanvasElement | null = null
  private resizeTarget: ResizeTarget = window

  constructor(view?: HTMLCanvasElement, resizeTarget: ResizeTarget = window) {
    this.view = view ?? null
    this.resizeTarget = resizeTarget
  }

  public attach(view: HTMLCanvasElement, resizeTarget: ResizeTarget = window) {
    this.view = view
    this.resizeTarget = resizeTarget
  }

  private initApp() {
    if (this.app) return

    const view = this.view ?? document.getElementById('live2dCanvas') as HTMLCanvasElement | null

    if (!view) {
      throw new Error(i18n.global.t('utils.live2d.hints.canvasNotFound'))
    }

    this.view = view

    this.app = new Application()

    return this.app.init({
      view,
      resizeTo: this.resizeTarget,
      backgroundAlpha: 0,
      autoDensity: true,
      resolution: devicePixelRatio,
    })
  }

  public async load(path: string) {
    await this.initApp()

    this.destroyModel()

    const files = await readDir(path)

    const modelFile = files.find(file => file.name.endsWith('.model3.json'))

    if (!modelFile) {
      throw new Error(i18n.global.t('utils.live2d.hints.notFound'))
    }

    const modelPath = join(path, modelFile.name)

    const modelJSON = JSON5.parse(await readTextFile(modelPath))

    const modelSetting = new CubismSetting({
      modelJSON,
    })

    modelSetting.redirectPath(({ file }) => {
      return convertFileSrc(join(path, file))
    })

    this.model = new Live2DSprite({
      modelSetting,
      ticker: Ticker.shared,
    })

    this.app?.stage.addChild(this.model)

    await this.model.ready

    const { width, height } = this.model

    const motions = groupBy(this.model.getMotions(), 'group')
    const expressions = this.model.getExpressions()

    return {
      width,
      height,
      motions,
      expressions,
    }
  }

  public destroy() {
    this.destroyModel()

    this.app?.destroy(true)
    this.app = null
  }

  private destroyModel() {
    if (!this.model) return

    this.model?.destroy()

    this.model = null
  }

  public resizeModel(modelSize: ModelSize, viewport?: ModelSize) {
    if (!this.model) return

    const { width, height } = modelSize
    const nextViewport = viewport ?? this.getViewportSize()

    const scaleX = nextViewport.width / width
    const scaleY = nextViewport.height / height
    const scale = Math.min(scaleX, scaleY)

    this.model.scale.set(scale)
    this.model.x = nextViewport.width / 2
    this.model.y = nextViewport.height / 2
    this.model.anchor.set(0.5)
  }

  public startMotion(motion: MotionInfo) {
    return this.model?.startMotion({
      ...motion,
      priority: Priority.Normal,
    })
  }

  public setExpression(index: number) {
    return this.model?.setExpression({ index })
  }

  public getParameterValueRange(id: string) {
    return this.model?.getParameterValueRangeById(id)
  }

  public setParameterValue(id: string, value: number | boolean) {
    return this.model?.setParameterValueById(id, Number(value))
  }

  public setMotionSoundEnabled(enabled: boolean) {
    Config.MotionSound = enabled
  }

  public setMaxFPS(fps: number) {
    Ticker.shared.maxFPS = fps
  }

  private getViewportSize(): ModelSize {
    if (this.resizeTarget instanceof Window) {
      return {
        width: this.resizeTarget.innerWidth,
        height: this.resizeTarget.innerHeight,
      }
    }

    return {
      width: this.resizeTarget.clientWidth,
      height: this.resizeTarget.clientHeight,
    }
  }
}

const live2d = new Live2d()

export default live2d
