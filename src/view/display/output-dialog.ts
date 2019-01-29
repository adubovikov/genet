import PanelView from './panel-view'
import flatten from 'lodash.flatten'
import genet from '@genet/api'
import m from 'mithril'

function parseRange(exp) {
  const ranges = exp
    .trim()
    .split(',')
    .map((str) => str.replace(/\s+/g, '').trim()
      .split('-'))
    .map((nums) => {
      if (nums.length === 1 && nums[0] === '') {
        return null
      }
      const begin = Number.parseInt(nums[0] || 0, 10)
      const end = Number.parseInt(nums[1] || Number.MAX_SAFE_INTEGER, 10)
      if (nums.length === 1) {
        return [begin, begin]
      } else if (nums.length === 2) {
        if (begin < end) {
          return [begin, end]
        }
        return [end, begin]

      }
      return null
    })
    .filter((range) => range !== null)
  ranges.sort((lhs, rhs) => lhs[0] - rhs[0])
  const merged: [number, number][] = []
  for (const range of ranges) {
    const last = merged[merged.length - 1]
    if (last && last[1] >= range[0]) {
      last[1] = Math.max(last[1], range[1])
    } else {
      merged.push(range)
    }
  }
  return merged.map((range) => {
    if (range[0] === 0 && range[1] === Number.MAX_SAFE_INTEGER) {
      return ''
    }
    if (range[0] === 0) {
      return `($.index <= ${range[1]})`
    }
    if (range[1] === Number.MAX_SAFE_INTEGER) {
      return `(${range[0]} <= $.index)`
    }
    return `(${range[0]} <= $.index && $.index <= ${range[1]})`
  }).join(' || ')
}

export default class OutputDialog {
  private filter: string
  private output: string
  private mode: string
  constructor() {
    this.filter = ''
    this.output = ''
    this.mode = genet.workspace.get('_.pcap.exporter.mode', 'all')
  }

  oncreate(vnode) {
    vnode.dom.querySelector('select[name=filter-type]').value = this.mode
    vnode.dom.querySelector(
      'input[type=text][name=range]').value =
      genet.workspace.get('_.pcap.exporter.range', '')
    vnode.dom.querySelector(
      'input[type=text][name=filter]').value =
      genet.workspace.get('_.pcap.exporter.filter', '')
    process.nextTick(() => {
      this.update(vnode)
    })
  }

  update(vnode) {
    this.output = vnode.dom.querySelector('select[name=output-id]').value
    this.mode = vnode.dom.querySelector('select[name=filter-type]').value

    switch (this.mode) {
      case 'visible':
        this.filter = vnode.attrs.displayFilter
        break
      case 'range':
        this.filter = parseRange(vnode.dom.querySelector(
          'input[type=text][name=range]').value)
        break
      case 'checked':
        {
          const list = Array.from(
            vnode.attrs.checkedFrames.values()).join(',')
          this.filter = parseRange(list)
        }
        break
      case 'filter':
        this.filter = vnode.dom.querySelector(
          'input[type=text][name=filter]').value
        break
      default:
        this.filter = ''
    }

    process.nextTick(() => {
      genet.workspace.set('_.pcap.exporter.mode', this.mode)
      genet.workspace.set('_.pcap.exporter.range', vnode.dom.querySelector(
        'input[type=text][name=range]').value)
      genet.workspace.set('_.pcap.exporter.filter', vnode.dom.querySelector(
        'input[type=text][name=filter]').value)
    })
  }

  view(vnode) {
    const { callback } = vnode.attrs
    const panels = genet.workspace.panelLayout['dialog:output'] || []
    const layout = flatten(panels).map((tab) => genet.workspace.panel(tab))
      .filter((panel) => typeof panel !== 'undefined')
    if (this.output === '' && layout.length > 0) {
      this.output = layout[0].id
    }

    return m('div', [
      m('ul', [
        m('li', [
          m('select', {
            name: 'filter-type',
            onchange: () => this.update(vnode),
          }, [
              m('option', { value: 'all' }, ['All Frames']),
              m('option', { value: 'visible' }, ['Visible Frames']),
              m('option', { value: 'checked' },
                [`Checked Frames (${vnode.attrs.checkedFrames.size})`]),
              m('option', { value: 'range' }, ['Index Range']),
              m('option', { value: 'filter' }, ['Custom Filter'])
            ])
        ]),
        m('li', {
          style: {
            display:
              this.mode === 'range'
                ? 'block'
                : 'none',
          },
        }, [
            m('input', {
              type: 'text',
              name: 'range',
              placeholder: 'e.g. 0-20, 51, 60-',
              onchange: () => this.update(vnode),
            })
          ]),
        m('li', {
          style: {
            display:
              this.mode === 'filter'
                ? 'block'
                : 'none',
          },
        }, [
            m('input', {
              type: 'text',
              name: 'filter',
              placeholder: 'e.g. tcp.flags.ack',
              onchange: () => this.update(vnode),
            })
          ]),
        m('li', [
          m('select', {
            name: 'output-id',
            onchange: () => this.update(vnode),
          }, layout.map((panel) =>
            m('option', { value: panel.id }, [panel.name])))
        ])
      ]),
      m('div',
        layout.map((panel) => m('div', {
          style: {
            display: panel.id === this.output
              ? 'block'
              : 'none',
          },
        }, [
            m(PanelView, {
              ...panel,
              attrs: {
                callback: (id, url) => {
                  const { sess } = vnode.attrs
                  sess.createWriter(id, url, this.filter)
                    .then(() => {
                      genet.notify.show(url.pathname, {
                        type: 'sussess',
                        title: 'Exported',
                      })
                    })
                    .catch((err) => {
                      genet.notify.show(`${url.pathname}\n${err.message}`, {
                        type: 'error',
                        title: 'Error',
                      })
                    })
                  callback()
                },
              },
            })
          ]))
      )])
  }
}
