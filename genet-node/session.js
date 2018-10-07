const native = require('./binding')
const { Token } = native
const { Disposable } = require('disposables')
const { EventEmitter } = require('events')
class Frame {
  constructor(frame) {
    this._frame = frame
    this._root = null
  }

  get index() {
    return this._frame.index
  }

  get root() {
    return this._frame.layers[0] || null
  }

  get primary() {
    if (!this.root) {
      return null
    }
    function fistChild(layer) {
      if (layer.children.length === 0) {
        return layer
      }
      return fistChild(layer.children[0])
    }
    return fistChild(this.root)
  }

  query(id) {
    return this._frame.query(id)
  }
}

class Session extends EventEmitter {
  constructor(profile, options) {
    super()
    this._options = options
    this._sess = new native.Session(profile, (json) => {
      const event = JSON.parse(json)
      switch (event.type) {
        case 'frames':
          this._status.frames = event.length
          break
        case 'async_frames':
          this._status.asyncFrames = event.length
          break
        case 'filtered_frames':
          this._status.filters[Token.string(event.id)] =
            { frames: event.length }
          break
        case 'error':
          this.emit('error', event.error)
          break
        default:
      }
      this.emit('update', event)
    })
    this._streams = []
    this._streamReaders = new Set()
    this._status = {
      filters: {},
      frames: 0,
      asyncFrames: 0,
      stream: false,
    }
  }

  close() {
    this._sess.close()
  }

  frames(start, end) {
    return this._sess
      .frames(start, end)
      .map((frame) => new Frame(frame))
  }

  filteredFrames(id, start, end) {
    return this._sess.filteredFrames(Token.get(id), start, end)
  }

  get status() {
    return this._status
  }

  applyFilterMacros(filter) {
    return filter.replace(/@(.+)(:?\s|$)/g, (_, body) => {
      for (const macro of this._options.filterMacros) {
        const result = macro.func(body)
        if (typeof result === 'string') {
          return result
        }
      }
      throw new Error(`unrecognized macro: @${body}`)
    })
  }

  setFilter(id, filter = '') {
    this._sess.setFilter(Token.get(id), this.applyFilterMacros(filter))
    if (filter === '') {
      Reflect.deleteProperty(this._status.filters, id)
    }
  }

  createReader(id, arg = {}) {
    const handle = this._sess.createReader(id, JSON.stringify(arg))
    if (handle === 0) {
      throw new Error(`failed to invoke reader: ${id}`)
    }
    const disposable = new Disposable(() => {
      this._sess.closeReader(handle)
    })
    disposable.promise = new Promise((res, rej) => {
      this.on('update', (event) => {
        if (event.id === handle && event.type === 'output') {
          if (event.error === null) {
            res()
          } else {
            rej(new Error(event.error))
          }
        }
      })
    })
    return disposable
  }

  async createWriter(id, arg = {}, filter = '') {
    const handle = this._sess.createWriter(
      id, JSON.stringify(arg), this.applyFilterMacros(filter))
    if (handle === 0) {
      throw new Error(`failed to invoke writer: ${id}`)
    }
    const disposable = new Disposable(() => {
      this._sess.closeReader(handle)
    })
    disposable.promise = new Promise((res, rej) => {
      this.on('update', (event) => {
        if (event.id === handle && event.type === 'input') {
          if (event.error === null) {
            res()
          } else {
            rej(new Error(event.error))
          }
        }
      })
    })
    return disposable
  }

  regiterStreamReader(id, arg = {}) {
    const reader = {
      id,
      arg,
    }
    this._streamReaders.add(reader)
    return new Disposable(() => {
      this._streamReaders.delete(reader)
    })
  }

  startStream() {
    this.stopStream()
    this._streams = Array.from(this._streamReaders)
      .map(({ id, arg }) => this.createReader(id, arg))
    this._status.stream = true
  }

  stopStream() {
    for (const handle of this._streams) {
      handle.dispose()
    }
    this._streams = []
    this._status.stream = false
  }

  get length() {
    return this._sess.length
  }
}

class Profile extends native.Session.Profile { }

Reflect.defineProperty(Session, 'Profile', { value: Profile })

module.exports = Session
