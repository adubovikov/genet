import { AttributeValueItem } from './value'
import DefaultSummary from './default-summary'
import genet from '@genet/api'
import m from 'mithril'
import parseColor from 'parse-color'
import throttle from 'lodash.throttle'

class FrameView {
  private frame: any
  view(vnode) {
    const { sess } = vnode.attrs;
    const { viewState, key } = vnode.attrs
    if (!this.frame) {
      [this.frame] = sess.frames(key, key + 1)
    }
    if (!this.frame) {
      return m('div')
    }
    const columns = vnode.attrs.columns.map((column, index) => {
      const content = [
        column.func(this.frame)
      ]
      if (index === 0) {
        content.unshift(m('span', [
          m('input', {
            type: 'checkbox',
            checked: viewState.checkedFrames.has(key),
            onchange: (event) => {
              if (event.target.checked) {
                viewState.checkedFrames.add(key)
              } else {
                viewState.checkedFrames.delete(key)
              }
              return false
            },
          }),
          m('span', {
            style: {
              visibility: sess.status.asyncFrames > this.frame.index
                ? 'hidden'
                : 'visible'
            }
          }, ['●'])
        ]))
      }
      return m('span', {
        class: 'column',
        style: {
          width: index === vnode.attrs.columns.length - 1
            ? 'auto'
            : `${viewState.headerWidthList[index] || 100}px`,
        },
      }, content)
    })
    return m('div', {
      class: 'frame',
      style: vnode.attrs.style,
      active: viewState.selectedFrame === key,
      'data-layer': this.frame.primary.id,
      onmousedown: () => {
        viewState.selectedFrame = key
        genet.action.emit('core:frame:selected', this.frame)
      },
    }, [
        m('div', { class: 'header' }, columns)
      ])
  }
}

export default class FrameListView {
  private itemHeight: number
  private height: number
  private scrollTop: number
  private prevFrames: number
  private mapHeight: number
  private columns: any[]
  private dummyItem: HTMLElement
  private barStyle: HTMLElement
  private readonly mapHeader: number[]
  private readonly mapBuffer: Buffer
  private readonly updateMapThrottle: (any) => void
  constructor() {
    this.itemHeight = 30
    this.height = 0
    this.scrollTop = 0
    this.prevFrames = 0
    this.mapHeight = 256
    this.columns = []
    this.updateMapThrottle = throttle((vnode) => {
      this.updateMap(vnode)
    }, 500)

    this.mapHeader = [
      0x42, 0x4d, 0x36, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00,
      0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0xff,
      0xff, 0xff, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x13, 0x0b, 0x00, 0x00, 0x13, 0x0b, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ]
    this.mapBuffer = Buffer.allocUnsafe(
      this.mapHeader.length + (4 * this.mapHeight))
    for (let index = 0; index < this.mapHeader.length; index += 1) {
      this.mapBuffer[index] = this.mapHeader[index]
    }
  }

  updateMap(vnode) {
    const { sess } = vnode.attrs
    const { status } = sess
    if (sess && status.frames > 0 && this.dummyItem) {
      const frames = (status.filters[1]
        ? status.filters[1].frames
        : status.frames)
      if (frames > 0) {
        for (let line = 0; line < this.mapHeight; line += 1) {
          let index = Math.floor(frames / this.mapHeight * (line + 0.5))
          if (status.filters[1]) {
            [index] = sess.filteredFrames(1, index, index + 1)
          }
          const [frame] = sess.frames(index, index + 1)
          this.dummyItem.setAttribute('data-layer', frame.primary.id)
          const [red, green, blue] =
            parseColor(getComputedStyle(this.dummyItem)
              .getPropertyValue('background-color')).rgb
          const offset = this.mapHeader.length
          this.mapBuffer[offset + (line * 4) + 0] = blue
          this.mapBuffer[offset + (line * 4) + 1] = green
          this.mapBuffer[offset + (line * 4) + 2] = red
        }
      }
      const data = `data:image/bmp;base64,${this.mapBuffer.toString('base64')}`
      this.barStyle.textContent = `
      nav.frame-list .padding, nav.frame-list::-webkit-scrollbar {
        background-image: url(${data});
      }
      `
    }
  }

  view(vnode) {
    const { status } = vnode.attrs.sess
    const frames = status.filters[1]
      ? status.filters[1].frames
      : status.frames
    const startIndex = Math.floor(this.scrollTop / this.itemHeight)
    const visibleItems = Math.min(
      Math.floor(this.height / this.itemHeight) + 2, frames - startIndex)
    const listStyle = { height: `${frames * this.itemHeight}px` }
    const filteredFrames =
      vnode.attrs.sess.filteredFrames(
        1, startIndex, startIndex + visibleItems)
    const indices = status.filters[1]
      ? filteredFrames
      : new Array(visibleItems).fill(0)
        .map((_val, index) => startIndex + index)
    const items = indices.map((seq, index) => {
      const itemStyle = {
        height: `${this.itemHeight}px`,
        top: `${(index + startIndex) * this.itemHeight}px`,
      }
      return m(FrameView, {
        style: itemStyle,
        key: seq,
        sess: vnode.attrs.sess,
        columns: this.columns,
        viewState: vnode.attrs.viewState,
      })
    })
    return m('nav', { class: 'frame-list' }, [
      m('style', { class: 'scrollbar-style' }),
      m('div', {
        style: 'display: none;',
        class: 'dummy-item',
      }),
      m('div', {
        class: 'padding',
        style: { height: `${this.itemHeight * frames}px` },
      }),
      m('div', {
        class: 'container',
        style: listStyle,
      }, items)
    ])
  }

  onupdate(vnode) {
    const { sess, viewState } = vnode.attrs
    const { status } = sess
    const frames = status.filters[1]
      ? status.filters[1].frames
      : status.frames
    if (this.prevFrames !== frames) {
      this.updateMapThrottle(vnode)
      this.prevFrames = frames
      if (!viewState.scrollLock) {
        vnode.dom.scrollTop = vnode.dom.scrollHeight - vnode.dom.clientHeight
      }
    }
  }

  oncreate(vnode) {
    this.dummyItem = vnode.dom.parentNode.querySelector('.dummy-item')
    this.barStyle = vnode.dom.parentNode.querySelector('.scrollbar-style')

    const resizeObserver = new (window as any).ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.target === vnode.dom) {
          this.height = entry.contentRect.height
          resizeObserver.observe(vnode.dom)
          m.redraw()
        }
      }
    })
    resizeObserver.observe(vnode.dom)
    vnode.dom.addEventListener('scroll', (event) => {
      this.scrollTop = event.target.scrollTop
      m.redraw()
    })

    this.columns = [
      { func: (frame) => m('span', [frame.index]) },
      {
        func: (frame) => {
          const { id } = frame.primary
          return m('span', {
            class: 'protocol',
            'data-layer': id,
          }, [genet.session.tokenName(id)])
        },
      }
    ]

    const columns =
      genet.config.get('_.framelist.columns', [])
    this.columns.push(...columns
      .map((col) => ({
        func: (frame) => {
          const result = frame.query(col.value)
          let renderer = AttributeValueItem
          if (result !== null &&
            typeof result === 'object' &&
            result.constructor.name === 'Attr') {
            renderer = genet.session.attrRenderer(result.type) || renderer
            return m(renderer, { attr: result })
          }
          return m(renderer, { attr: { value: result } })
        },
      })))

    this.columns.push({
      func: (frame) => {
        const { id } = frame.primary
        const renderer = genet.session.layerRenderer(id) || DefaultSummary
        return m(renderer, { layer: frame.primary, frame })
      },
    })
  }
}
