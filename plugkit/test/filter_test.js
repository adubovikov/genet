const assert = require('assert')
const FilterCompiler = require('../filter')
function execute (filter) {
  const compiler = new FilterCompiler()
  return compiler.build(compiler.link(compiler.compile(filter)))()
}
describe('FilterCompiler', () => {
  describe('#compile', () => {
    it('should return empty string for empty input', () => {
      const compiler = new FilterCompiler()
      assert.strictEqual('', compiler.compile(''))
    })
  })
  describe('#compileFunction', () => {
    it('should return a filter function', () => {
      assert.strictEqual(true, execute(''))
      assert.strictEqual(false, execute('false'))
      assert.strictEqual(0, execute('0'))
      assert.strictEqual(-100, execute('-100'))
      assert.strictEqual(false, execute('0 < -100'))
      assert.strictEqual(11000, execute('500 * 22'))
      assert.strictEqual(true, execute('22 == "22"'))
      assert.strictEqual(false, execute('22 === "22"'))
      assert.strictEqual(true, execute('"aaa" === "aaa"'))
      assert.strictEqual('aaabbb', execute('"aaa" + "bbb"'))
      assert.strictEqual(true, execute('[] == []'))
      assert.strictEqual(true, execute('[555, "rrr"] === [555, "rrr"]'))
      assert.strictEqual(true, execute('[555, "rrr"] !== [555, "lll"]'))
      assert.strictEqual(true, execute('["lll" + "r"] > ["lll"]'))
      assert.strictEqual(true, execute('[1] > [0]'))
      assert.strictEqual(true, execute('[0, 0, 1] < [1, 0, 0]'))
      assert.strictEqual(512, execute('2 << 8'))
      assert.strictEqual(11, execute('5 + (true ? 6 : 8)'))
    })
  })
})
