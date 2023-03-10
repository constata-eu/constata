async function validate_certificate(file_input, signer){
  const file = file_input.files[0];
  const lines = (await file.text()).split("\n"); 
  /* The document must end with a new line, but some services remove it.
   * If we detect that, we add the newline back to the end of the buffer
   * otherwise the validaiton will fail */
  if(lines.slice(-2,-1) != ''){
    lines.push('');
  }

  function sanitize_line(l) {
    return (l.slice(-1) === '\r' ? l.slice(0,-1) : l) + "\n";
  }
  
  const signature = base64ToBytes(lines.slice(-4, -3)[0]);
  const raw_message = lines.slice(0, -10).map(sanitize_line).join("");
  const message = Buffer.from(raw_message, "utf8");
  return bitcoinMessage.verify(message, signer, signature, true);
}

function base64ToBytes(base64) {
  const bytesAsString = window.atob(base64);
  const size = bytesAsString.length;
  const bytes = new Uint8Array(size);
  for (let i = 0; i < size; i++) {
    bytes[i] = bytesAsString.charCodeAt(i);
  }
  return bytes;
}

window.inherits = function(ctor, superCtor) {
  if (superCtor) {
    ctor.super_ = superCtor
    ctor.prototype = Object.create(superCtor.prototype, {
      constructor: {
        value: ctor,
        enumerable: false,
        writable: true,
        configurable: true
      }
    })
  }
};

(function defineBufferModule(){
  /*!
   * The buffer module from node.js, for the browser.
   *
   * @author   Feross Aboukhadijeh <https://feross.org>
   * @license  MIT
   */
  window.Buffer = Buffer = function(arg, encodingOrOffset, length) {
    if (typeof arg === 'number') {
      return allocUnsafe(arg)
    }
    return from(arg, encodingOrOffset, length)
  }


  function createBuffer (length) {
    var buf = new Uint8Array(length)
    buf.__proto__ = Buffer.prototype
    return buf
  }


  if (typeof Symbol !== 'undefined' && Symbol.species != null &&
      Buffer[Symbol.species] === Buffer) {
    Object.defineProperty(Buffer, Symbol.species, {
      value: null,
      configurable: true,
      enumerable: false,
      writable: false
    })
  }

  function from (value, encodingOrOffset, length) {
    if (typeof value === 'string') {
      return fromString(value, encodingOrOffset)
    }

    if (ArrayBuffer.isView(value)) {
      return fromArrayLike(value)
    }

    if (isInstance(value, ArrayBuffer) ||
        (value && isInstance(value.buffer, ArrayBuffer))) {
      return fromArrayBuffer(value, encodingOrOffset, length)
    }

    var b = fromObject(value)
    if (b) return b
  }

  Buffer.from = function (value, encodingOrOffset, length) {
    return from(value, encodingOrOffset, length)
  }

  Buffer.prototype.__proto__ = Uint8Array.prototype
  Buffer.__proto__ = Uint8Array

  function alloc (size, fill, encoding) {
    if (size <= 0) {
      return createBuffer(size)
    }
    if (fill !== undefined) {
      return typeof encoding === 'string'
        ? createBuffer(size).fill(fill, encoding)
        : createBuffer(size).fill(fill)
    }
    return createBuffer(size)
  }

  Buffer.alloc = function (size, fill, encoding) {
    return alloc(size, fill, encoding)
  }

  function allocUnsafe (size) {
    return createBuffer(size < 0 ? 0 : size | 0)
  }

  Buffer.allocUnsafe = function (size) {
    return allocUnsafe(size)
  }

  function fromString (string, encoding) {
    var length = byteLength(string, encoding) | 0
    var buf = createBuffer(length)
    buf.write(string, encoding)
    return buf
  }

  function fromArrayLike (array) {
    var length = array.length < 0 ? 0 : array.length | 0
    var buf = createBuffer(length)
    for (var i = 0; i < length; i += 1) {
      buf[i] = array[i] & 255
    }
    return buf
  }

  function fromArrayBuffer (array, byteOffset, length) {
    var buf
    if (byteOffset === undefined && length === undefined) {
      buf = new Uint8Array(array)
    } else if (length === undefined) {
      buf = new Uint8Array(array, byteOffset)
    } else {
      buf = new Uint8Array(array, byteOffset, length)
    }

    // Return an augmented `Uint8Array` instance
    buf.__proto__ = Buffer.prototype
    return buf
  }

  function fromObject (obj) {
    if (Buffer.isBuffer(obj)) {
      var len = obj.length | 0
      var buf = createBuffer(len)

      if (buf.length === 0) {
        return buf
      }

      obj.copy(buf, 0, 0, len)
      return buf
    }

    if (obj.length !== undefined) {
      if (typeof obj.length !== 'number' || numberIsNaN(obj.length)) {
        return createBuffer(0)
      }
      return fromArrayLike(obj)
    }

    if (obj.type === 'Buffer' && Array.isArray(obj.data)) {
      return fromArrayLike(obj.data)
    }
  }

  Buffer.isBuffer = function isBuffer (b) {
    return b != null && b._isBuffer === true &&
      b !== Buffer.prototype // so Buffer.isBuffer(Buffer.prototype) will be false
  }

  function byteLength (string, encoding) {
    return utf8ToBytes(string).length
  }
  Buffer.byteLength = byteLength
  Buffer.prototype._isBuffer = true

  function utf8Write (buf, string, offset, length) {
    return blitBuffer(utf8ToBytes(string, buf.length - offset), buf, offset, length)
  }

  Buffer.prototype.write = function write (string, offset, length, encoding) {
    // Buffer#write(string)
    if (offset === undefined) {
      encoding = 'utf8'
      length = this.length
      offset = 0
    } else if (length === undefined && typeof offset === 'string') {
      encoding = offset
      length = this.length
      offset = 0
    } else if (isFinite(offset)) {
      offset = offset >>> 0
      if (isFinite(length)) {
        length = length >>> 0
        if (encoding === undefined) encoding = 'utf8'
      } else {
        encoding = length
        length = undefined
      }
    }

    var remaining = this.length - offset
    if (length === undefined || length > remaining) length = remaining

    return utf8Write(this, string, offset, length)
  }

  Buffer.prototype.readUInt8 = function readUInt8 (offset) {
    offset = offset >>> 0
    return this[offset]
  }

  Buffer.prototype.readInt32LE = function readInt32LE (offset) {
    offset = offset >>> 0

    return (this[offset]) |
      (this[offset + 1] << 8) |
      (this[offset + 2] << 16) |
      (this[offset + 3] << 24)
  }

  Buffer.prototype.readInt32BE = function readInt32BE (offset) {
    offset = offset >>> 0

    return (this[offset] << 24) |
      (this[offset + 1] << 16) |
      (this[offset + 2] << 8) |
      (this[offset + 3])
  }

  Buffer.prototype.writeUInt8 = function writeUInt8 (value, offset) {
    value = +value
    offset = offset >>> 0
    this[offset] = (value & 0xff)
    return offset + 1
  }

  Buffer.prototype.writeUInt16LE = function writeUInt16LE (value, offset) {
      value = +value
      offset = offset >>> 0
      this[offset] = (value & 0xff)
      this[offset + 1] = (value >>> 8)
      return offset + 2
  }

  Buffer.prototype.writeUInt32LE = function writeUInt32LE (value, offset) {
    value = +value
    offset = offset >>> 0
    this[offset + 3] = (value >>> 24)
    this[offset + 2] = (value >>> 16)
    this[offset + 1] = (value >>> 8)
    this[offset] = (value & 0xff)
    return offset + 4
  }

  Buffer.prototype.writeUInt32BE = function writeUInt32BE (value, offset) {
    value = +value
    offset = offset >>> 0
    this[offset] = (value >>> 24)
    this[offset + 1] = (value >>> 16)
    this[offset + 2] = (value >>> 8)
    this[offset + 3] = (value & 0xff)
    return offset + 4
  }

  Buffer.prototype.writeInt32LE = function writeInt32LE (value, offset) {
    value = +value
    offset = offset >>> 0
    this[offset] = (value & 0xff)
    this[offset + 1] = (value >>> 8)
    this[offset + 2] = (value >>> 16)
    this[offset + 3] = (value >>> 24)
    return offset + 4
  }

  Buffer.prototype.writeInt32BE = function writeInt32BE (value, offset) {
    value = +value
    offset = offset >>> 0
    if (value < 0) value = 0xffffffff + value + 1
    this[offset] = (value >>> 24)
    this[offset + 1] = (value >>> 16)
    this[offset + 2] = (value >>> 8)
    this[offset + 3] = (value & 0xff)
    return offset + 4
  }

  // copy(targetBuffer, targetStart=0, sourceStart=0, sourceEnd=buffer.length)
  Buffer.prototype.copy = function copy (target, targetStart, start, end) {
    if (!start) start = 0
    if (!end && end !== 0) end = this.length
    if (targetStart >= target.length) targetStart = target.length
    if (!targetStart) targetStart = 0
    if (end > 0 && end < start) end = start

    // Copy 0 bytes; we're done
    if (end === start) return 0
    if (target.length === 0 || this.length === 0) return 0

    // Are we oob?
    if (end > this.length) end = this.length
    if (target.length - targetStart < end - start) {
      end = target.length - targetStart + start
    }

    var len = end - start

    if (this === target && typeof Uint8Array.prototype.copyWithin === 'function') {
      // Use built-in when available, missing from IE11
      this.copyWithin(targetStart, start, end)
    } else if (this === target && start < targetStart && targetStart < end) {
      // descending copy from end
      for (var i = len - 1; i >= 0; --i) {
        target[i + targetStart] = this[i + start]
      }
    } else {
      Uint8Array.prototype.set.call(
        target,
        this.subarray(start, end),
        targetStart
      )
    }

    return len
  }

  Buffer.prototype.fill = function fill (val, start, end, encoding) {
    // Handle string cases:
    if (typeof val === 'string') {
      if (typeof start === 'string') {
        encoding = start
        start = 0
        end = this.length
      } else if (typeof end === 'string') {
        encoding = end
        end = this.length
      }
      if (encoding !== undefined && typeof encoding !== 'string') {
        throw new TypeError('encoding must be a string')
      }
      if (typeof encoding === 'string' && !Buffer.isEncoding(encoding)) {
        throw new TypeError('Unknown encoding: ' + encoding)
      }
      if (val.length === 1) {
        var code = val.charCodeAt(0)
        if ((encoding === 'utf8' && code < 128) ||
            encoding === 'latin1') {
          // Fast path: If `val` fits into a single byte, use that numeric value.
          val = code
        }
      }
    } else if (typeof val === 'number') {
      val = val & 255
    }

    // Invalid ranges are not set to a default, so can range check early.
    if (start < 0 || this.length < start || this.length < end) {
      throw new RangeError('Out of range index')
    }

    if (end <= start) {
      return this
    }

    start = start >>> 0
    end = end === undefined ? this.length : end >>> 0

    if (!val) val = 0

    var i
    if (typeof val === 'number') {
      for (i = start; i < end; ++i) {
        this[i] = val
      }
    } else {
      var bytes = Buffer.isBuffer(val)
        ? val
        : Buffer.from(val, encoding)
      var len = bytes.length
      if (len === 0) {
        throw new TypeError('The value "' + val +
          '" is invalid for argument "value"')
      }
      for (i = 0; i < end - start; ++i) {
        this[i + start] = bytes[i % len]
      }
    }

    return this
  }

  // HELPER FUNCTIONS
  // ================

  function utf8ToBytes (string, units) {
    units = units || Infinity
    var codePoint
    var length = string.length
    var leadSurrogate = null
    var bytes = []

    for (var i = 0; i < length; ++i) {
      codePoint = string.charCodeAt(i)

      // is surrogate component
      if (codePoint > 0xD7FF && codePoint < 0xE000) {
        // last char was a lead
        if (!leadSurrogate) {
          // no lead yet
          if (codePoint > 0xDBFF) {
            // unexpected trail
            if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
            continue
          } else if (i + 1 === length) {
            // unpaired lead
            if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
            continue
          }

          // valid lead
          leadSurrogate = codePoint

          continue
        }

        // 2 leads in a row
        if (codePoint < 0xDC00) {
          if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
          leadSurrogate = codePoint
          continue
        }

        // valid surrogate pair
        codePoint = (leadSurrogate - 0xD800 << 10 | codePoint - 0xDC00) + 0x10000
      } else if (leadSurrogate) {
        // valid bmp char, but last char was a lead
        if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
      }

      leadSurrogate = null

      // encode utf8
      if (codePoint < 0x80) {
        if ((units -= 1) < 0) break
        bytes.push(codePoint)
      } else if (codePoint < 0x800) {
        if ((units -= 2) < 0) break
        bytes.push(
          codePoint >> 0x6 | 0xC0,
          codePoint & 0x3F | 0x80
        )
      } else if (codePoint < 0x10000) {
        if ((units -= 3) < 0) break
        bytes.push(
          codePoint >> 0xC | 0xE0,
          codePoint >> 0x6 & 0x3F | 0x80,
          codePoint & 0x3F | 0x80
        )
      } else if (codePoint < 0x110000) {
        if ((units -= 4) < 0) break
        bytes.push(
          codePoint >> 0x12 | 0xF0,
          codePoint >> 0xC & 0x3F | 0x80,
          codePoint >> 0x6 & 0x3F | 0x80,
          codePoint & 0x3F | 0x80
        )
      } else {
        throw new Error('Invalid code point')
      }
    }

    return bytes
  }

  function blitBuffer (src, dst, offset, length) {
    for (var i = 0; i < length; ++i) {
      if ((i + offset >= dst.length) || (i >= src.length)) break
      dst[i + offset] = src[i]
    }
    return i
  }

  function isInstance (obj, type) {
    return obj instanceof type ||
      (obj != null && obj.constructor != null && obj.constructor.name != null &&
        obj.constructor.name === type.name)
  }

  function numberIsNaN (obj) {
    // For IE11 support
    return obj !== obj // eslint-disable-line no-self-compare
  }

})();

(function defineHashBaseModule(){
  window.HashBase = HashBase = function(blockSize) {
    this._block = Buffer.allocUnsafe(blockSize)
    this._blockSize = blockSize
    this._blockOffset = 0
    this._length = [0, 0, 0, 0]
    this._finalized = false
  }

  HashBase.prototype.update = function (data, encoding) {
    if (this._finalized) throw new Error('Digest already called')
    if (!Buffer.isBuffer(data)) data = Buffer.from(data, encoding)

    // consume data
    var block = this._block
    var offset = 0
    while (this._blockOffset + data.length - offset >= this._blockSize) {
      for (var i = this._blockOffset; i < this._blockSize;) block[i++] = data[offset++]
      this._update()
      this._blockOffset = 0
    }
    while (offset < data.length) block[this._blockOffset++] = data[offset++]

    // update length
    for (var j = 0, carry = data.length * 8; carry > 0; ++j) {
      this._length[j] += carry
      carry = (this._length[j] / 0x0100000000) | 0
      if (carry > 0) this._length[j] -= 0x0100000000 * carry
    }

    return this
  }

  HashBase.prototype.digest = function (encoding) {
    if (this._finalized) throw new Error('Digest already called')
    this._finalized = true

    var digest = this._digest()
    if (encoding !== undefined) digest = digest.toString(encoding)

    // reset state
    this._block.fill(0)
    this._blockOffset = 0
    for (var i = 0; i < 4; ++i) this._length[i] = 0

    return digest
  }
})();

(function defineRipemd160Module(){
	/*
	The MIT License (MIT)

	Copyright (c) 2016 crypto-browserify

	Permission is hereby granted, free of charge, to any person obtaining a copy
	of this software and associated documentation files (the "Software"), to deal
	in the Software without restriction, including without limitation the rights
	to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
	copies of the Software, and to permit persons to whom the Software is
	furnished to do so, subject to the following conditions:

	The above copyright notice and this permission notice shall be included in all
	copies or substantial portions of the Software.

	THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
	IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
	FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
	AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
	LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
	OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
	SOFTWARE.
	*/

  var ARRAY16 = new Array(16)

  var zl = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5, 2, 14, 11, 8,
    3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12,
    1, 9, 11, 10, 0, 8, 12, 4, 13, 3, 7, 15, 14, 5, 6, 2,
    4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13
  ]

  var zr = [
    5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12,
    6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12, 4, 9, 1, 2,
    15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13,
    8, 6, 4, 1, 3, 11, 15, 0, 5, 12, 2, 13, 9, 7, 10, 14,
    12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11
  ]

  var sl = [
    11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8,
    7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15, 9, 11, 7, 13, 12,
    11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5,
    11, 12, 14, 15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12,
    9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6
  ]

  var sr = [
    8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6,
    9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12, 7, 6, 15, 13, 11,
    9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5,
    15, 5, 8, 11, 14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8,
    8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11
  ]

  var hl = [0x00000000, 0x5a827999, 0x6ed9eba1, 0x8f1bbcdc, 0xa953fd4e]
  var hr = [0x50a28be6, 0x5c4dd124, 0x6d703ef3, 0x7a6d76e9, 0x00000000]

  window.RIPEMD160 = RIPEMD160 = function() {
    HashBase.call(this, 64)

    // state
    this._a = 0x67452301
    this._b = 0xefcdab89
    this._c = 0x98badcfe
    this._d = 0x10325476
    this._e = 0xc3d2e1f0
  }

  inherits(RIPEMD160, HashBase)

  RIPEMD160.prototype._update = function () {
    var words = ARRAY16
    for (var j = 0; j < 16; ++j) words[j] = this._block.readInt32LE(j * 4)

    var al = this._a | 0
    var bl = this._b | 0
    var cl = this._c | 0
    var dl = this._d | 0
    var el = this._e | 0

    var ar = this._a | 0
    var br = this._b | 0
    var cr = this._c | 0
    var dr = this._d | 0
    var er = this._e | 0

    // computation
    for (var i = 0; i < 80; i += 1) {
      var tl
      var tr
      if (i < 16) {
        tl = fn1(al, bl, cl, dl, el, words[zl[i]], hl[0], sl[i])
        tr = fn5(ar, br, cr, dr, er, words[zr[i]], hr[0], sr[i])
      } else if (i < 32) {
        tl = fn2(al, bl, cl, dl, el, words[zl[i]], hl[1], sl[i])
        tr = fn4(ar, br, cr, dr, er, words[zr[i]], hr[1], sr[i])
      } else if (i < 48) {
        tl = fn3(al, bl, cl, dl, el, words[zl[i]], hl[2], sl[i])
        tr = fn3(ar, br, cr, dr, er, words[zr[i]], hr[2], sr[i])
      } else if (i < 64) {
        tl = fn4(al, bl, cl, dl, el, words[zl[i]], hl[3], sl[i])
        tr = fn2(ar, br, cr, dr, er, words[zr[i]], hr[3], sr[i])
      } else { // if (i<80) {
        tl = fn5(al, bl, cl, dl, el, words[zl[i]], hl[4], sl[i])
        tr = fn1(ar, br, cr, dr, er, words[zr[i]], hr[4], sr[i])
      }

      al = el
      el = dl
      dl = rotl(cl, 10)
      cl = bl
      bl = tl

      ar = er
      er = dr
      dr = rotl(cr, 10)
      cr = br
      br = tr
    }

    // update state
    var t = (this._b + cl + dr) | 0
    this._b = (this._c + dl + er) | 0
    this._c = (this._d + el + ar) | 0
    this._d = (this._e + al + br) | 0
    this._e = (this._a + bl + cr) | 0
    this._a = t
  }

  RIPEMD160.prototype._digest = function () {
    // create padding and handle blocks
    this._block[this._blockOffset++] = 0x80
    if (this._blockOffset > 56) {
      this._block.fill(0, this._blockOffset, 64)
      this._update()
      this._blockOffset = 0
    }

    this._block.fill(0, this._blockOffset, 56)
    this._block.writeUInt32LE(this._length[0], 56)
    this._block.writeUInt32LE(this._length[1], 60)
    this._update()

    // produce result
    var buffer = Buffer.alloc ? Buffer.alloc(20) : new Buffer(20)
    buffer.writeInt32LE(this._a, 0)
    buffer.writeInt32LE(this._b, 4)
    buffer.writeInt32LE(this._c, 8)
    buffer.writeInt32LE(this._d, 12)
    buffer.writeInt32LE(this._e, 16)
    return buffer
  }

  function rotl (x, n) {
    return (x << n) | (x >>> (32 - n))
  }

  function fn1 (a, b, c, d, e, m, k, s) {
    return (rotl((a + (b ^ c ^ d) + m + k) | 0, s) + e) | 0
  }

  function fn2 (a, b, c, d, e, m, k, s) {
    return (rotl((a + ((b & c) | ((~b) & d)) + m + k) | 0, s) + e) | 0
  }

  function fn3 (a, b, c, d, e, m, k, s) {
    return (rotl((a + ((b | (~c)) ^ d) + m + k) | 0, s) + e) | 0
  }

  function fn4 (a, b, c, d, e, m, k, s) {
    return (rotl((a + ((b & d) | (c & (~d))) + m + k) | 0, s) + e) | 0
  }

  function fn5 (a, b, c, d, e, m, k, s) {
    return (rotl((a + (b ^ (c | (~d))) + m + k) | 0, s) + e) | 0
  }
})();

(function defineCipherBaseModule(){
  window.CipherBase = CipherBase = function(hashMode) {
    this.hashMode = typeof hashMode === 'string'
    if (this.hashMode) {
      this[hashMode] = this._finalOrDigest
    } else {
      this.final = this._finalOrDigest
    }
    if (this._final) {
      this.__final = this._final
      this._final = null
    }
    this._decoder = null
    this._encoding = null
  }

  CipherBase.prototype.update = function (data, inputEnc, outputEnc) {
    if (typeof data === 'string') {
      data = Buffer.from(data, inputEnc)
    }

    var outData = this._update(data)
    if (this.hashMode) return this

    if (outputEnc) {
      outData = this._toString(outData, outputEnc)
    }

    return outData
  }

  CipherBase.prototype._finalOrDigest = function (outputEnc) {
    var outData = this.__final() || Buffer.alloc(0)
    if (outputEnc) {
      outData = this._toString(outData, outputEnc, true)
    }
    return outData
  }
})();

(function defineHashBaseModule(){
  window.Hash = Hash = function(blockSize, finalSize) {
    this._block = Buffer.alloc(blockSize)
    this._finalSize = finalSize
    this._blockSize = blockSize
    this._len = 0
  }

  Hash.prototype.update = function (data, enc) {
    if (typeof data === 'string') {
      enc = enc || 'utf8'
      data = Buffer.from(data, enc)
    }

    var block = this._block
    var blockSize = this._blockSize
    var length = data.length
    var accum = this._len

    for (var offset = 0; offset < length;) {
      var assigned = accum % blockSize
      var remainder = Math.min(length - offset, blockSize - assigned)

      for (var i = 0; i < remainder; i++) {
        block[assigned + i] = data[offset + i]
      }

      accum += remainder
      offset += remainder

      if ((accum % blockSize) === 0) {
        this._update(block)
      }
    }

    this._len += length
    return this
  }

  Hash.prototype.digest = function (enc) {
    var rem = this._len % this._blockSize

    this._block[rem] = 0x80

    // zero (rem + 1) trailing bits, where (rem + 1) is the smallest
    // non-negative solution to the equation (length + 1 + (rem + 1)) === finalSize mod blockSize
    this._block.fill(0, rem + 1)

    if (rem >= this._finalSize) {
      this._update(this._block)
      this._block.fill(0)
    }

    var bits = this._len * 8

    // uint32
    if (bits <= 0xffffffff) {
      this._block.writeUInt32BE(bits, this._blockSize - 4)

    // uint64
    } else {
      var lowBits = (bits & 0xffffffff) >>> 0
      var highBits = (bits - lowBits) / 0x100000000

      this._block.writeUInt32BE(highBits, this._blockSize - 8)
      this._block.writeUInt32BE(lowBits, this._blockSize - 4)
    }

    this._update(this._block)
    var hash = this._hash()

    return enc ? hash.toString(enc) : hash
  }
})();

(function defineSha256Module(){
	/*
	 * A JavaScript implementation of the Secure Hash Algorithm, SHA-256, as defined
	 * in FIPS 180-2
	 * Version 2.2 Copyright Angel Marin, Paul Johnston 2000 - 2009.
	 * Other contributors: Greg Holt, Andrew Kepert, Ydnar, Lostinet
	 * Distributed under the BSD License
	 * See http://pajhome.org.uk/crypt/md5 for details.
	 * Also http://anmar.eu.org/projects/jssha2/
	 */

  var K = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5,
    0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3,
    0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC,
    0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7,
    0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13,
    0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3,
    0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5,
    0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208,
    0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2
  ]

  var W = new Array(64)

  window.Sha256 = Sha256 = function() {
    this.init()

    this._w = W // new Array(64)

    Hash.call(this, 64, 56)
  }

  inherits(Sha256, Hash)

  Sha256.prototype.init = function () {
    this._a = 0x6a09e667
    this._b = 0xbb67ae85
    this._c = 0x3c6ef372
    this._d = 0xa54ff53a
    this._e = 0x510e527f
    this._f = 0x9b05688c
    this._g = 0x1f83d9ab
    this._h = 0x5be0cd19

    return this
  }

  function ch (x, y, z) {
    return z ^ (x & (y ^ z))
  }

  function maj (x, y, z) {
    return (x & y) | (z & (x | y))
  }

  function sigma0 (x) {
    return (x >>> 2 | x << 30) ^ (x >>> 13 | x << 19) ^ (x >>> 22 | x << 10)
  }

  function sigma1 (x) {
    return (x >>> 6 | x << 26) ^ (x >>> 11 | x << 21) ^ (x >>> 25 | x << 7)
  }

  function gamma0 (x) {
    return (x >>> 7 | x << 25) ^ (x >>> 18 | x << 14) ^ (x >>> 3)
  }

  function gamma1 (x) {
    return (x >>> 17 | x << 15) ^ (x >>> 19 | x << 13) ^ (x >>> 10)
  }

  Sha256.prototype._update = function (M) {
    var W = this._w

    var a = this._a | 0
    var b = this._b | 0
    var c = this._c | 0
    var d = this._d | 0
    var e = this._e | 0
    var f = this._f | 0
    var g = this._g | 0
    var h = this._h | 0

    for (var i = 0; i < 16; ++i) W[i] = M.readInt32BE(i * 4)
    for (; i < 64; ++i) W[i] = (gamma1(W[i - 2]) + W[i - 7] + gamma0(W[i - 15]) + W[i - 16]) | 0

    for (var j = 0; j < 64; ++j) {
      var T1 = (h + sigma1(e) + ch(e, f, g) + K[j] + W[j]) | 0
      var T2 = (sigma0(a) + maj(a, b, c)) | 0

      h = g
      g = f
      f = e
      e = (d + T1) | 0
      d = c
      c = b
      b = a
      a = (T1 + T2) | 0
    }

    this._a = (a + this._a) | 0
    this._b = (b + this._b) | 0
    this._c = (c + this._c) | 0
    this._d = (d + this._d) | 0
    this._e = (e + this._e) | 0
    this._f = (f + this._f) | 0
    this._g = (g + this._g) | 0
    this._h = (h + this._h) | 0
  }

  Sha256.prototype._hash = function () {
    var H = Buffer.allocUnsafe(32)

    H.writeInt32BE(this._a, 0)
    H.writeInt32BE(this._b, 4)
    H.writeInt32BE(this._c, 8)
    H.writeInt32BE(this._d, 12)
    H.writeInt32BE(this._e, 16)
    H.writeInt32BE(this._f, 20)
    H.writeInt32BE(this._g, 24)
    H.writeInt32BE(this._h, 28)

    return H
  }
})();

(function defineCreateHashModule(){
  function HashSha256() {
    CipherBase.call(this, 'digest')
    this._hash = new Sha256()
  }

  inherits(HashSha256, CipherBase)

  HashSha256.prototype._update = function (data) {
    this._hash.update(data)
  }

  HashSha256.prototype._final = function () {
    return this._hash.digest()
  }

  window.createHash = function(alg) {
    alg = alg.toLowerCase()
    if (alg === 'rmd160' || alg === 'ripemd160') return new RIPEMD160()
    if (alg === 'sha256' ) return new HashSha256();
  }
})();

(function defineBigNumberModule(){
  /*
  Copyright Fedor Indutny, 2015.

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.
  */
  window.BN = BN = function(number, base) {
    if (BN.isBN(number)) {
      return number;
    }

    this.negative = 0;
    this.words = null;
    this.length = 0;

    // Reduction context
    this.red = null;

    if (number !== null) {
      this._init(number || 0, base);
    }
  }


  BN.isBN = function isBN (num) {
    if (num instanceof BN) {
      return true;
    }

    return num !== null && typeof num === 'object' &&
      num.constructor.wordSize === BN.wordSize && Array.isArray(num.words);
  };


  BN.prototype._init = function init (number, base) {
    if (typeof number === 'number') {
      return this._initNumber(number, base);
    }

    if (typeof number === 'object') {
      return this._initArray(number, base);
    }

    number = number.toString().replace(/\s+/g, '');
    var start = 0;
    if (number[0] === '-') {
      start++;
      this.negative = 1;
    }

    if (start < number.length) {
      if (base === 16) {
        this._parseHex(number, start);
      } else {
        this._parseBase(number, base, start);
      }
    }
  };

  BN.prototype._initNumber = function _initNumber (number, base) {
    if (number < 0) {
      this.negative = 1;
      number = -number;
    }
    if (number < 0x4000000) {
      this.words = [ number & 0x3ffffff ];
      this.length = 1;
    } else if (number < 0x10000000000000) {
      this.words = [
        number & 0x3ffffff,
        (number / 0x4000000) & 0x3ffffff
      ];
      this.length = 2;
    } else {
      this.words = [
        number & 0x3ffffff,
        (number / 0x4000000) & 0x3ffffff,
        1
      ];
      this.length = 3;
    }
  };

  BN.prototype._initArray = function _initArray (number, base) {
    // Perhaps a Uint8Array
    if (number.length <= 0) {
      this.words = [ 0 ];
      this.length = 1;
      return this;
    }

    this.length = Math.ceil(number.length / 3);
    this.words = new Array(this.length);
    for (var i = 0; i < this.length; i++) {
      this.words[i] = 0;
    }

    var j, w;
    var off = 0;
    for (i = number.length - 1, j = 0; i >= 0; i -= 3) {
      w = number[i] | (number[i - 1] << 8) | (number[i - 2] << 16);
      this.words[j] |= (w << off) & 0x3ffffff;
      this.words[j + 1] = (w >>> (26 - off)) & 0x3ffffff;
      off += 24;
      if (off >= 26) {
        off -= 26;
        j++;
      }
    }
    return this.strip();
  };

  function parseHex4Bits (string, index) {
    var c = string.charCodeAt(index);
    // 'A' - 'F'
    if (c >= 65 && c <= 70) {
      return c - 55;
    // 'a' - 'f'
    } else if (c >= 97 && c <= 102) {
      return c - 87;
    // '0' - '9'
    } else {
      return (c - 48) & 0xf;
    }
  }

  function parseHexByte (string, lowerBound, index) {
    var r = parseHex4Bits(string, index);
    if (index - 1 >= lowerBound) {
      r |= parseHex4Bits(string, index - 1) << 4;
    }
    return r;
  }

  BN.prototype._parseHex = function _parseHex (number, start) {
    // Create possibly bigger array to ensure that it fits the number
    this.length = Math.ceil((number.length - start) / 6);
    this.words = new Array(this.length);
    for (var i = 0; i < this.length; i++) {
      this.words[i] = 0;
    }

    // 24-bits chunks
    var off = 0;
    var j = 0;

    var w;
    for (i = number.length - 1; i >= start; i -= 2) {
      w = parseHexByte(number, start, i) << off;
      this.words[j] |= w & 0x3ffffff;
      if (off >= 18) {
        off -= 18;
        j += 1;
        this.words[j] |= w >>> 26;
      } else {
        off += 8;
      }
    }

    this.strip();
  };

  BN.prototype.copy = function copy (dest) {
    dest.words = new Array(this.length);
    for (var i = 0; i < this.length; i++) {
      dest.words[i] = this.words[i];
    }
    dest.length = this.length;
    dest.negative = this.negative;
    dest.red = this.red;
  };

  BN.prototype.clone = function clone () {
    var r = new BN(null);
    this.copy(r);
    return r;
  };

  BN.prototype._expand = function _expand (size) {
    while (this.length < size) {
      this.words[this.length++] = 0;
    }
    return this;
  };

  // Remove leading `0` from `this`
  BN.prototype.strip = function strip () {
    while (this.length > 1 && this.words[this.length - 1] === 0) {
      this.length--;
    }
    return this._normSign();
  };

  BN.prototype._normSign = function _normSign () {
    // -0 = 0
    if (this.length === 1 && this.words[0] === 0) {
      this.negative = 0;
    }
    return this;
  };

  BN.prototype.toArray = function toArray (length) {
    var ArrayType = Array;
    var byteLength = this.byteLength();
    var reqLength = length || Math.max(1, byteLength);

    this.strip();
    var res = new ArrayType(reqLength);

    var b, i;
    var q = this.clone();
    for (i = 0; i < reqLength - byteLength; i++) {
      res[i] = 0;
    }

    for (i = 0; !q.isZero(); i++) {
      b = q.andln(0xff);
      q.iushrn(8);

      res[reqLength - i - 1] = b;
    }

    return res;
  };

  BN.prototype._countBits = function _countBits (w) {
    return 32 - Math.clz32(w);
  };

  // Return number of used bits in a BN
  BN.prototype.bitLength = function bitLength () {
    var w = this.words[this.length - 1];
    var hi = this._countBits(w);
    return (this.length - 1) * 26 + hi;
  };

  BN.prototype.byteLength = function byteLength () {
    return Math.ceil(this.bitLength() / 8);
  };

  // Return negative clone of `this`
  BN.prototype.neg = function neg () {
    return this.clone().ineg();
  };

  BN.prototype.ineg = function ineg () {
    if (!this.isZero()) {
      this.negative ^= 1;
    }

    return this;
  };

  // Add `num` to `this` in-place
  BN.prototype.iadd = function iadd (num) {
    var r;

    // negative + positive
    if (this.negative !== 0 && num.negative === 0) {
      this.negative = 0;
      r = this.isub(num);
      this.negative ^= 1;
      return this._normSign();

    // positive + negative
    } else if (this.negative === 0 && num.negative !== 0) {
      num.negative = 0;
      r = this.isub(num);
      num.negative = 1;
      return r._normSign();
    }

    // a.length > b.length
    var a, b;
    if (this.length > num.length) {
      a = this;
      b = num;
    } else {
      a = num;
      b = this;
    }

    var carry = 0;
    for (var i = 0; i < b.length; i++) {
      r = (a.words[i] | 0) + (b.words[i] | 0) + carry;
      this.words[i] = r & 0x3ffffff;
      carry = r >>> 26;
    }
    for (; carry !== 0 && i < a.length; i++) {
      r = (a.words[i] | 0) + carry;
      this.words[i] = r & 0x3ffffff;
      carry = r >>> 26;
    }

    this.length = a.length;
    if (carry !== 0) {
      this.words[this.length] = carry;
      this.length++;
    // Copy the rest of the words
    } else if (a !== this) {
      for (; i < a.length; i++) {
        this.words[i] = a.words[i];
      }
    }

    return this;
  };

  // Add `num` to `this`
  BN.prototype.add = function add (num) {
    var res;
    if (num.negative !== 0 && this.negative === 0) {
      num.negative = 0;
      res = this.sub(num);
      num.negative ^= 1;
      return res;
    } else if (num.negative === 0 && this.negative !== 0) {
      this.negative = 0;
      res = num.sub(this);
      this.negative = 1;
      return res;
    }

    if (this.length > num.length) return this.clone().iadd(num);

    return num.clone().iadd(this);
  };

  // Subtract `num` from `this` in-place
  BN.prototype.isub = function isub (num) {
    // this - (-num) = this + num
    if (num.negative !== 0) {
      num.negative = 0;
      var r = this.iadd(num);
      num.negative = 1;
      return r._normSign();

    // -this - num = -(this + num)
    } else if (this.negative !== 0) {
      this.negative = 0;
      this.iadd(num);
      this.negative = 1;
      return this._normSign();
    }

    // At this point both numbers are positive
    var cmp = this.cmp(num);

    // Optimization - zeroify
    if (cmp === 0) {
      this.negative = 0;
      this.length = 1;
      this.words[0] = 0;
      return this;
    }

    // a > b
    var a, b;
    if (cmp > 0) {
      a = this;
      b = num;
    } else {
      a = num;
      b = this;
    }

    var carry = 0;
    for (var i = 0; i < b.length; i++) {
      r = (a.words[i] | 0) - (b.words[i] | 0) + carry;
      carry = r >> 26;
      this.words[i] = r & 0x3ffffff;
    }
    for (; carry !== 0 && i < a.length; i++) {
      r = (a.words[i] | 0) + carry;
      carry = r >> 26;
      this.words[i] = r & 0x3ffffff;
    }

    // Copy rest of the words
    if (carry === 0 && i < a.length && a !== this) {
      for (; i < a.length; i++) {
        this.words[i] = a.words[i];
      }
    }

    this.length = Math.max(this.length, i);

    if (a !== this) {
      this.negative = 1;
    }

    return this.strip();
  };

  // Subtract `num` from `this`
  BN.prototype.sub = function sub (num) {
    return this.clone().isub(num);
  };

  function smallMulTo (self, num, out) {
    out.negative = num.negative ^ self.negative;
    var len = (self.length + num.length) | 0;
    out.length = len;
    len = (len - 1) | 0;

    // Peel one iteration (compiler can't do it, because of code complexity)
    var a = self.words[0] | 0;
    var b = num.words[0] | 0;
    var r = a * b;

    var lo = r & 0x3ffffff;
    var carry = (r / 0x4000000) | 0;
    out.words[0] = lo;

    for (var k = 1; k < len; k++) {
      // Sum all words with the same `i + j = k` and accumulate `ncarry`,
      // note that ncarry could be >= 0x3ffffff
      var ncarry = carry >>> 26;
      var rword = carry & 0x3ffffff;
      var maxJ = Math.min(k, num.length - 1);
      for (var j = Math.max(0, k - self.length + 1); j <= maxJ; j++) {
        var i = (k - j) | 0;
        a = self.words[i] | 0;
        b = num.words[j] | 0;
        r = a * b + rword;
        ncarry += (r / 0x4000000) | 0;
        rword = r & 0x3ffffff;
      }
      out.words[k] = rword | 0;
      carry = ncarry | 0;
    }
    if (carry !== 0) {
      out.words[k] = carry | 0;
    } else {
      out.length--;
    }

    return out.strip();
  }

  BN.prototype.mulTo = function mulTo (num, out) {
    var res;
    var len = this.length + num.length;
    if (len < 63) {
      res = smallMulTo(this, num, out);
    } else if (len < 1024) {
      res = bigMulTo(this, num, out);
    }

    return res;
  };

  BN.prototype.mul = function mul (num) {
    var out = new BN(null);
    out.words = new Array(this.length + num.length);
    return this.mulTo(num, out);
  };

  BN.prototype.imul = function imul (num) {
    return this.clone().mulTo(num, this);
  };

  BN.prototype.iushln = function iushln (bits) {
    var r = bits % 26;
    var s = (bits - r) / 26;
    var carryMask = (0x3ffffff >>> (26 - r)) << (26 - r);
    var i;

    if (r !== 0) {
      var carry = 0;

      for (i = 0; i < this.length; i++) {
        var newCarry = this.words[i] & carryMask;
        var c = ((this.words[i] | 0) - newCarry) << r;
        this.words[i] = c | carry;
        carry = newCarry >>> (26 - r);
      }

      if (carry) {
        this.words[i] = carry;
        this.length++;
      }
    }

    if (s !== 0) {
      for (i = this.length - 1; i >= 0; i--) {
        this.words[i + s] = this.words[i];
      }

      for (i = 0; i < s; i++) {
        this.words[i] = 0;
      }

      this.length += s;
    }

    return this.strip();
  };

  BN.prototype.iushrn = function iushrn (bits, hint, extended) {
    var h;
    if (hint) {
      h = (hint - (hint % 26)) / 26;
    } else {
      h = 0;
    }

    var r = bits % 26;
    var s = Math.min((bits - r) / 26, this.length);
    var mask = 0x3ffffff ^ ((0x3ffffff >>> r) << r);
    var maskedWords = extended;

    h -= s;
    h = Math.max(0, h);

    // Extended mode, copy masked part
    if (maskedWords) {
      for (var i = 0; i < s; i++) {
        maskedWords.words[i] = this.words[i];
      }
      maskedWords.length = s;
    }

    if (s === 0) {
      // No-op, we should not move anything at all
    } else if (this.length > s) {
      this.length -= s;
      for (i = 0; i < this.length; i++) {
        this.words[i] = this.words[i + s];
      }
    } else {
      this.words[0] = 0;
      this.length = 1;
    }

    var carry = 0;
    for (i = this.length - 1; i >= 0 && (carry !== 0 || i >= h); i--) {
      var word = this.words[i] | 0;
      this.words[i] = (carry << (26 - r)) | (word >>> r);
      carry = word & mask;
    }

    // Push carried bits as a mask
    if (maskedWords && carry !== 0) {
      maskedWords.words[maskedWords.length++] = carry;
    }

    if (this.length === 0) {
      this.words[0] = 0;
      this.length = 1;
    }

    return this.strip();
  };

  BN.prototype.ushln = function ushln (bits) {
    return this.clone().iushln(bits);
  };

  BN.prototype.ushrn = function ushrn (bits) {
    return this.clone().iushrn(bits);
  };

  // Add plain number `num` to `this`
  BN.prototype.iaddn = function iaddn (num) {
    if (num < 0) return this.isubn(-num);

    // Possible sign change
    if (this.negative !== 0) {
      if (this.length === 1 && (this.words[0] | 0) < num) {
        this.words[0] = num - (this.words[0] | 0);
        this.negative = 0;
        return this;
      }

      this.negative = 0;
      this.isubn(num);
      this.negative = 1;
      return this;
    }

    return this._iaddn(num);
  };

  BN.prototype._iaddn = function _iaddn (num) {
    this.words[0] += num;

    for (var i = 0; i < this.length && this.words[i] >= 0x4000000; i++) {
      this.words[i] -= 0x4000000;
      if (i === this.length - 1) {
        this.words[i + 1] = 1;
      } else {
        this.words[i + 1]++;
      }
    }
    this.length = Math.max(this.length, i + 1);

    return this;
  };

  BN.prototype._ishlnsubmul = function _ishlnsubmul (num, mul, shift) {
    var len = num.length + shift;
    var i;

    this._expand(len);

    var w;
    var carry = 0;
    for (i = 0; i < num.length; i++) {
      w = (this.words[i + shift] | 0) + carry;
      var right = (num.words[i] | 0) * mul;
      w -= right & 0x3ffffff;
      carry = (w >> 26) - ((right / 0x4000000) | 0);
      this.words[i + shift] = w & 0x3ffffff;
    }
    for (; i < this.length - shift; i++) {
      w = (this.words[i + shift] | 0) + carry;
      carry = w >> 26;
      this.words[i + shift] = w & 0x3ffffff;
    }

    if (carry === 0) return this.strip();

    // Subtraction overflow
    carry = 0;
    for (i = 0; i < this.length; i++) {
      w = -(this.words[i] | 0) + carry;
      carry = w >> 26;
      this.words[i] = w & 0x3ffffff;
    }
    this.negative = 1;

    return this.strip();
  };

  BN.prototype._wordDiv = function _wordDiv (num, mode) {
    var shift = this.length - num.length;

    var a = this.clone();
    var b = num;

    // Normalize
    var bhi = b.words[b.length - 1] | 0;
    var bhiBits = this._countBits(bhi);
    shift = 26 - bhiBits;
    if (shift !== 0) {
      b = b.ushln(shift);
      a.iushln(shift);
      bhi = b.words[b.length - 1] | 0;
    }

    // Initialize quotient
    var m = a.length - b.length;
    var q;

    if (mode !== 'mod') {
      q = new BN(null);
      q.length = m + 1;
      q.words = new Array(q.length);
      for (var i = 0; i < q.length; i++) {
        q.words[i] = 0;
      }
    }

    var diff = a.clone()._ishlnsubmul(b, 1, m);
    if (diff.negative === 0) {
      a = diff;
      if (q) {
        q.words[m] = 1;
      }
    }

    for (var j = m - 1; j >= 0; j--) {
      var qj = (a.words[b.length + j] | 0) * 0x4000000 +
        (a.words[b.length + j - 1] | 0);

      // NOTE: (qj / bhi) is (0x3ffffff * 0x4000000 + 0x3ffffff) / 0x2000000 max
      // (0x7ffffff)
      qj = Math.min((qj / bhi) | 0, 0x3ffffff);

      a._ishlnsubmul(b, qj, j);
      while (a.negative !== 0) {
        qj--;
        a.negative = 0;
        a._ishlnsubmul(b, 1, j);
        if (!a.isZero()) {
          a.negative ^= 1;
        }
      }
      if (q) {
        q.words[j] = qj;
      }
    }
    if (q) {
      q.strip();
    }
    a.strip();

    // Denormalize
    if (mode !== 'div' && shift !== 0) {
      a.iushrn(shift);
    }

    return {
      div: q || null,
      mod: a
    };
  };

  // NOTE: 1) `mode` can be set to `mod` to request mod only,
  //       to `div` to request div only, or be absent to
  //       request both div & mod
  //       2) `positive` is true if unsigned mod is requested
  BN.prototype.divmod = function divmod (num, mode, positive) {
    if (this.isZero()) {
      return {
        div: new BN(0),
        mod: new BN(0)
      };
    }

    var div, mod, res;
    if (this.negative !== 0 && num.negative === 0) {
      res = this.neg().divmod(num, mode);

      if (mode !== 'mod') {
        div = res.div.neg();
      }

      if (mode !== 'div') {
        mod = res.mod.neg();
        if (positive && mod.negative !== 0) {
          mod.iadd(num);
        }
      }

      return {
        div: div,
        mod: mod
      };
    }

    if (this.negative === 0 && num.negative !== 0) {
      res = this.divmod(num.neg(), mode);

      if (mode !== 'mod') {
        div = res.div.neg();
      }

      return {
        div: div,
        mod: res.mod
      };
    }

    if ((this.negative & num.negative) !== 0) {
      res = this.neg().divmod(num.neg(), mode);

      if (mode !== 'div') {
        mod = res.mod.neg();
        if (positive && mod.negative !== 0) {
          mod.isub(num);
        }
      }

      return {
        div: res.div,
        mod: mod
      };
    }

    // Both numbers are positive at this point

    // Strip both numbers to approximate shift value
    if (num.length > this.length || this.cmp(num) < 0) {
      return {
        div: new BN(0),
        mod: this
      };
    }

    // Very short reduction
    if (num.length === 1) {
      if (mode === 'div') {
        return {
          div: this.divn(num.words[0]),
          mod: null
        };
      }

      if (mode === 'mod') {
        return {
          div: null,
          mod: new BN(this.modn(num.words[0]))
        };
      }

      return {
        div: this.divn(num.words[0]),
        mod: new BN(this.modn(num.words[0]))
      };
    }

    return this._wordDiv(num, mode);
  };

  BN.prototype.div = function div (num) {
    return this.divmod(num, 'div', false).div;
  };

  BN.prototype.umod = function umod (num) {
    return this.divmod(num, 'mod', true).mod;
  };

  // Find Round(`this` / `num`)
  BN.prototype.divRound = function divRound (num) {
    var dm = this.divmod(num);

    // Fast case - exact division
    if (dm.mod.isZero()) return dm.div;

    var mod = dm.div.negative !== 0 ? dm.mod.isub(num) : dm.mod;

    var half = num.ushrn(1);
    var r2 = num.andln(1);
    var cmp = mod.cmp(half);

    // Round down
    if (cmp < 0 || r2 === 1 && cmp === 0) return dm.div;

    // Round up
    return dm.div.negative !== 0 ? dm.div.isubn(1) : dm.div.iaddn(1);
  };

  BN.prototype.modn = function modn (num) {
    var p = (1 << 26) % num;

    var acc = 0;
    for (var i = this.length - 1; i >= 0; i--) {
      acc = (p * acc + (this.words[i] | 0)) % num;
    }

    return acc;
  };

  BN.prototype.egcd = function egcd (p) {
    var x = this;
    var y = p.clone();

    if (x.negative !== 0) {
      x = x.umod(p);
    } else {
      x = x.clone();
    }

    // A * x + B * y = x
    var A = new BN(1);
    var B = new BN(0);

    // C * x + D * y = y
    var C = new BN(0);
    var D = new BN(1);

    var g = 0;

    while (x.isEven() && y.isEven()) {
      x.iushrn(1);
      y.iushrn(1);
      ++g;
    }

    var yp = y.clone();
    var xp = x.clone();

    while (!x.isZero()) {
      for (var i = 0, im = 1; (x.words[0] & im) === 0 && i < 26; ++i, im <<= 1);
      if (i > 0) {
        x.iushrn(i);
        while (i-- > 0) {
          if (A.isOdd() || B.isOdd()) {
            A.iadd(yp);
            B.isub(xp);
          }

          A.iushrn(1);
          B.iushrn(1);
        }
      }

      for (var j = 0, jm = 1; (y.words[0] & jm) === 0 && j < 26; ++j, jm <<= 1);
      if (j > 0) {
        y.iushrn(j);
        while (j-- > 0) {
          if (C.isOdd() || D.isOdd()) {
            C.iadd(yp);
            D.isub(xp);
          }

          C.iushrn(1);
          D.iushrn(1);
        }
      }

      if (x.cmp(y) >= 0) {
        x.isub(y);
        A.isub(C);
        B.isub(D);
      } else {
        y.isub(x);
        C.isub(A);
        D.isub(B);
      }
    }

    return {
      a: C,
      b: D,
      gcd: y.iushln(g)
    };
  };

  BN.prototype._invmp = function _invmp (p) {
    var a = this;
    var b = p.clone();

    if (a.negative !== 0) {
      a = a.umod(p);
    } else {
      a = a.clone();
    }

    var x1 = new BN(1);
    var x2 = new BN(0);

    var delta = b.clone();

    while (a.cmpn(1) > 0 && b.cmpn(1) > 0) {
      for (var i = 0, im = 1; (a.words[0] & im) === 0 && i < 26; ++i, im <<= 1);
      if (i > 0) {
        a.iushrn(i);
        while (i-- > 0) {
          if (x1.isOdd()) {
            x1.iadd(delta);
          }

          x1.iushrn(1);
        }
      }

      for (var j = 0, jm = 1; (b.words[0] & jm) === 0 && j < 26; ++j, jm <<= 1);
      if (j > 0) {
        b.iushrn(j);
        while (j-- > 0) {
          if (x2.isOdd()) {
            x2.iadd(delta);
          }

          x2.iushrn(1);
        }
      }

      if (a.cmp(b) >= 0) {
        a.isub(b);
        x1.isub(x2);
      } else {
        b.isub(a);
        x2.isub(x1);
      }
    }

    var res;
    if (a.cmpn(1) === 0) {
      res = x1;
    } else {
      res = x2;
    }

    if (res.cmpn(0) < 0) {
      res.iadd(p);
    }

    return res;
  };

  BN.prototype.invm = function invm (num) {
    return this.egcd(num).a.umod(num);
  };

  BN.prototype.isEven = function isEven () {
    return (this.words[0] & 1) === 0;
  };

  BN.prototype.isOdd = function isOdd () {
    return (this.words[0] & 1) === 1;
  };

  // And first word and num
  BN.prototype.andln = function andln (num) {
    return this.words[0] & num;
  };

  BN.prototype.isZero = function isZero () {
    return this.length === 1 && this.words[0] === 0;
  };

  BN.prototype.cmpn = function cmpn (num) {
    var negative = num < 0;

    if (this.negative !== 0 && !negative) return -1;
    if (this.negative === 0 && negative) return 1;

    this.strip();

    var res;
    if (this.length > 1) {
      res = 1;
    } else {
      if (negative) {
        num = -num;
      }

      var w = this.words[0] | 0;
      res = w === num ? 0 : w < num ? -1 : 1;
    }
    if (this.negative !== 0) return -res | 0;
    return res;
  };

  BN.prototype.cmp = function cmp (num) {
    if (this.negative !== 0 && num.negative === 0) return -1;
    if (this.negative === 0 && num.negative !== 0) return 1;

    var res = this.ucmp(num);
    if (this.negative !== 0) return -res | 0;
    return res;
  };

  // Unsigned comparison
  BN.prototype.ucmp = function ucmp (num) {
    // At this point both numbers have the same sign
    if (this.length > num.length) return 1;
    if (this.length < num.length) return -1;

    var res = 0;
    for (var i = this.length - 1; i >= 0; i--) {
      var a = this.words[i] | 0;
      var b = num.words[i] | 0;

      if (a === b) continue;
      if (a < b) {
        res = -1;
      } else if (a > b) {
        res = 1;
      }
      break;
    }
    return res;
  };

  BN.red = function red (num) {
    return new Red(num);
  };

  BN.prototype.toRed = function toRed (ctx) {
    return ctx.convertTo(this)._forceRed(ctx);
  };

  BN.prototype.fromRed = function fromRed () {
    return this.red.convertFrom(this);
  };

  BN.prototype._forceRed = function _forceRed (ctx) {
    this.red = ctx;
    return this;
  };

  BN.prototype.forceRed = function forceRed (ctx) {
    return this._forceRed(ctx);
  };

  BN.prototype.redAdd = function redAdd (num) {
    return this.red.add(this, num);
  };

  BN.prototype.redIAdd = function redIAdd (num) {
    return this.red.iadd(this, num);
  };

  BN.prototype.redSub = function redSub (num) {
    return this.red.sub(this, num);
  };

  BN.prototype.redISub = function redISub (num) {
    return this.red.isub(this, num);
  };

  BN.prototype.redMul = function redMul (num) {
    return this.red.mul(this, num);
  };

  BN.prototype.redSqr = function redSqr () {
    return this.red.sqr(this);
  };

  // Square root over p
  BN.prototype.redSqrt = function redSqrt () {
    return this.red.sqrt(this);
  };

  BN.prototype.redInvm = function redInvm () {
    return this.red.invm(this);
  };

  // Return negative clone of `this` % `red modulo`
  BN.prototype.redNeg = function redNeg () {
    return this.red.neg(this);
  };

  // Prime numbers with efficient reduction
  var primes = { k256: null };

  // Pseudo-Mersenne prime
  function MPrime (name, p) {
    // P = 2 ^ N - K
    this.name = name;
    this.p = new BN(p, 16);
    this.n = this.p.bitLength();
    this.k = new BN(1).iushln(this.n).isub(this.p);

    this.tmp = this._tmp();
  }

  MPrime.prototype._tmp = function _tmp () {
    var tmp = new BN(null);
    tmp.words = new Array(Math.ceil(this.n / 13));
    return tmp;
  };

  MPrime.prototype.ireduce = function ireduce (num) {
    // Assumes that `num` is less than `P^2`
    // num = HI * (2 ^ N - K) + HI * K + LO = HI * K + LO (mod P)
    var r = num;
    var rlen;

    do {
      this.split(r, this.tmp);
      r = this.imulK(r);
      r = r.iadd(this.tmp);
      rlen = r.bitLength();
    } while (rlen > this.n);

    var cmp = rlen < this.n ? -1 : r.ucmp(this.p);
    if (cmp === 0) {
      r.words[0] = 0;
      r.length = 1;
    } else if (cmp > 0) {
      r.isub(this.p);
    } else {
      if (r.strip !== undefined) {
        // r is BN v4 instance
        r.strip();
      } else {
        // r is BN v5 instance
        r._strip();
      }
    }

    return r;
  };

  function K256 () {
    MPrime.call(
      this,
      'k256',
      'ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff fffffffe fffffc2f');
  }
  inherits(K256, MPrime);

  K256.prototype.split = function split (input, output) {
    // 256 = 9 * 26 + 22
    var mask = 0x3fffff;

    var outLen = Math.min(input.length, 9);
    for (var i = 0; i < outLen; i++) {
      output.words[i] = input.words[i];
    }
    output.length = outLen;

    if (input.length <= 9) {
      input.words[0] = 0;
      input.length = 1;
      return;
    }

    // Shift by 9 limbs
    var prev = input.words[9];
    output.words[output.length++] = prev & mask;

    for (i = 10; i < input.length; i++) {
      var next = input.words[i] | 0;
      input.words[i - 10] = ((next & mask) << 4) | (prev >>> 22);
      prev = next;
    }
    prev >>>= 22;
    input.words[i - 10] = prev;
    if (prev === 0 && input.length > 10) {
      input.length -= 10;
    } else {
      input.length -= 9;
    }
  };

  K256.prototype.imulK = function imulK (num) {
    // K = 0x1000003d1 = [ 0x40, 0x3d1 ]
    num.words[num.length] = 0;
    num.words[num.length + 1] = 0;
    num.length += 2;

    // bounded at: 0x40 * 0x3ffffff + 0x3d0 = 0x100000390
    var lo = 0;
    for (var i = 0; i < num.length; i++) {
      var w = num.words[i] | 0;
      lo += w * 0x3d1;
      num.words[i] = lo & 0x3ffffff;
      lo = w * 0x40 + ((lo / 0x4000000) | 0);
    }

    // Fast length reduction
    if (num.words[num.length - 1] === 0) {
      num.length--;
      if (num.words[num.length - 1] === 0) {
        num.length--;
      }
    }
    return num;
  };

  BN._prime = function prime (name) {
    if (primes[name]) return primes[name];

    var prime = new K256();
    primes[name] = prime;

    return prime;
  };

  function Red (m) {
    var prime = BN._prime(m);
    this.m = prime.p;
    this.prime = prime;
  }

  Red.prototype.imod = function imod (a) {
    if (this.prime) return this.prime.ireduce(a)._forceRed(this);
    return a.umod(this.m)._forceRed(this);
  };

  Red.prototype.neg = function neg (a) {
    if (a.isZero()) {
      return a.clone();
    }

    return this.m.sub(a)._forceRed(this);
  };

  Red.prototype.add = function add (a, b) {
    var res = a.add(b);
    if (res.cmp(this.m) >= 0) {
      res.isub(this.m);
    }
    return res._forceRed(this);
  };

  Red.prototype.iadd = function iadd (a, b) {
    var res = a.iadd(b);
    if (res.cmp(this.m) >= 0) {
      res.isub(this.m);
    }
    return res;
  };

  Red.prototype.sub = function sub (a, b) {
    var res = a.sub(b);
    if (res.cmpn(0) < 0) {
      res.iadd(this.m);
    }
    return res._forceRed(this);
  };

  Red.prototype.isub = function isub (a, b) {
    var res = a.isub(b);
    if (res.cmpn(0) < 0) {
      res.iadd(this.m);
    }
    return res;
  };

  Red.prototype.mul = function mul (a, b) {
    return this.imod(a.mul(b));
  };

  Red.prototype.sqr = function sqr (a) {
    return this.mul(a, a);
  };

  Red.prototype.sqrt = function sqrt (a) {
    if (a.isZero()) return a.clone();

    var mod3 = this.m.andln(3);

    // Fast case
    if (mod3 === 3) {
      var pow = this.m.add(new BN(1)).iushrn(2);
      return this.pow(a, pow);
    }

    // Tonelli-Shanks algorithm (Totally unoptimized and slow)
    //
    // Find Q and S, that Q * 2 ^ S = (P - 1)
    var q = this.m.subn(1);
    var s = 0;
    while (!q.isZero() && q.andln(1) === 0) {
      s++;
      q.iushrn(1);
    }

    var one = new BN(1).toRed(this);
    var nOne = one.redNeg();

    // Find quadratic non-residue
    // NOTE: Max is such because of generalized Riemann hypothesis.
    var lpow = this.m.subn(1).iushrn(1);
    var z = this.m.bitLength();
    z = new BN(2 * z * z).toRed(this);

    while (this.pow(z, lpow).cmp(nOne) !== 0) {
      z.redIAdd(nOne);
    }

    var c = this.pow(z, q);
    var r = this.pow(a, q.addn(1).iushrn(1));
    var t = this.pow(a, q);
    var m = s;
    while (t.cmp(one) !== 0) {
      var tmp = t;
      for (var i = 0; tmp.cmp(one) !== 0; i++) {
        tmp = tmp.redSqr();
      }
      var b = this.pow(c, new BN(1).iushln(m - i - 1));

      r = r.redMul(b);
      c = b.redSqr();
      t = t.redMul(c);
      m = i;
    }

    return r;
  };

  Red.prototype.invm = function invm (a) {
    var inv = a._invmp(this.m);
    if (inv.negative !== 0) {
      inv.negative = 0;
      return this.imod(inv).redNeg();
    } else {
      return this.imod(inv);
    }
  };

  Red.prototype.pow = function pow (a, num) {
    if (num.isZero()) return new BN(1).toRed(this);
    if (num.cmpn(1) === 0) return a.clone();

    var windowSize = 4;
    var wnd = new Array(1 << windowSize);
    wnd[0] = new BN(1).toRed(this);
    wnd[1] = a;
    for (var i = 2; i < wnd.length; i++) {
      wnd[i] = this.mul(wnd[i - 1], a);
    }

    var res = wnd[0];
    var current = 0;
    var currentLen = 0;
    var start = num.bitLength() % 26;
    if (start === 0) {
      start = 26;
    }

    for (i = num.length - 1; i >= 0; i--) {
      var word = num.words[i];
      for (var j = start - 1; j >= 0; j--) {
        var bit = (word >> j) & 1;
        if (res !== wnd[0]) {
          res = this.sqr(res);
        }

        if (bit === 0 && current === 0) {
          currentLen = 0;
          continue;
        }

        current <<= 1;
        current |= bit;
        currentLen++;
        if (currentLen !== windowSize && (i !== 0 || j !== 0)) continue;

        res = this.mul(res, wnd[current]);
        currentLen = 0;
        current = 0;
      }
      start = 26;
    }

    return res;
  };

  Red.prototype.convertTo = function convertTo (num) {
    var r = num.umod(this.m);

    return r === num ? r.clone() : r;
  };

  Red.prototype.convertFrom = function convertFrom (num) {
    var res = num.clone();
    res.red = null;
    return res;
  };
})();

(function defineUtilsModule(){
  window.utils = utils = {};

  utils.assert = function(val, msg) {
    if (!val)
      throw new Error(msg || 'Assertion failed');
  }
  utils.assert.equal = function assertEqual(l, r, msg) {
    if (l != r)
      throw new Error(msg || ('Assertion failed: ' + l + ' != ' + r));
  };

  utils.toHex = function(msg) {
    var res = '';
    for (var i = 0; i < msg.length; i++)
      res += zero2(msg[i].toString(16));
    return res;
  }

  utils.encode = function encode(arr, enc) {
    if (enc === 'hex')
      return toHex(arr);
    else
      return arr;
  };

  // Represent k1, k2 in a Joint Sparse Form
  utils.getJSF = function(k1, k2) {
    var jsf = [
      [],
      [],
    ];

    k1 = k1.clone();
    k2 = k2.clone();
    var d1 = 0;
    var d2 = 0;
    var m8;
    while (k1.cmpn(-d1) > 0 || k2.cmpn(-d2) > 0) {
      // First phase
      var m14 = (k1.andln(3) + d1) & 3;
      var m24 = (k2.andln(3) + d2) & 3;
      if (m14 === 3)
        m14 = -1;
      if (m24 === 3)
        m24 = -1;
      var u1;
      if ((m14 & 1) === 0) {
        u1 = 0;
      } else {
        m8 = (k1.andln(7) + d1) & 7;
        if ((m8 === 3 || m8 === 5) && m24 === 2)
          u1 = -m14;
        else
          u1 = m14;
      }
      jsf[0].push(u1);

      var u2;
      if ((m24 & 1) === 0) {
        u2 = 0;
      } else {
        m8 = (k2.andln(7) + d2) & 7;
        if ((m8 === 3 || m8 === 5) && m14 === 2)
          u2 = -m24;
        else
          u2 = m24;
      }
      jsf[1].push(u2);

      // Second phase
      if (2 * d1 === u1 + 1)
        d1 = 1 - d1;
      if (2 * d2 === u2 + 1)
        d2 = 1 - d2;
      k1.iushrn(1);
      k2.iushrn(1);
    }

    return jsf;
  }
})();

(function defineBaseCurveModule(){
  var getJSF = utils.getJSF;
  var assert = utils.assert;

  window.BaseCurve = BaseCurve = function(type, conf) {
    this.type = type;
    this.p = new BN(conf.p, 16);

    // Use Montgomery, when there is no fast reduction for the prime
    this.red = conf.prime ? BN.red(conf.prime) : BN.mont(this.p);

    // Useful for many curves
    this.zero = new BN(0).toRed(this.red);
    this.one = new BN(1).toRed(this.red);
    this.two = new BN(2).toRed(this.red);

    // Curve configuration, optional
    this.n = conf.n && new BN(conf.n, 16);
    this.g = conf.g && this.pointFromJSON(conf.g, conf.gRed);

    // Temporary arrays
    this._wnafT1 = new Array(4);
    this._wnafT2 = new Array(4);
    this._wnafT3 = new Array(4);
    this._wnafT4 = new Array(4);

    this._bitLength = this.n ? this.n.bitLength() : 0;

    // Generalized Greg Maxwell's trick
    var adjustCount = this.n && this.p.div(this.n);
    if (!adjustCount || adjustCount.cmpn(100) > 0) {
      this.redN = null;
    } else {
      this._maxwellTrick = true;
      this.redN = this.n.toRed(this.red);
    }
  }

  BaseCurve.prototype._fixedNafMul = function _fixedNafMul(p, k) {
    assert(p.precomputed);
    var doubles = p._getDoubles();

    var naf = getNAF(k, 1, this._bitLength);
    var I = (1 << (doubles.step + 1)) - (doubles.step % 2 === 0 ? 2 : 1);
    I /= 3;

    // Translate into more windowed form
    var repr = [];
    var j;
    var nafW;
    for (j = 0; j < naf.length; j += doubles.step) {
      nafW = 0;
      for (var l = j + doubles.step - 1; l >= j; l--)
        nafW = (nafW << 1) + naf[l];
      repr.push(nafW);
    }

    var a = this.jpoint(null, null, null);
    var b = this.jpoint(null, null, null);
    for (var i = I; i > 0; i--) {
      for (j = 0; j < repr.length; j++) {
        nafW = repr[j];
        if (nafW === i)
          b = b.mixedAdd(doubles.points[j]);
        else if (nafW === -i)
          b = b.mixedAdd(doubles.points[j].neg());
      }
      a = a.add(b);
    }
    return a.toP();
  };

  BaseCurve.prototype._wnafMul = function _wnafMul(p, k) {
    var w = 4;

    // Precompute window
    var nafPoints = p._getNAFPoints(w);
    w = nafPoints.wnd;
    var wnd = nafPoints.points;

    // Get NAF form
    var naf = getNAF(k, w, this._bitLength);

    // Add `this`*(N+1) for every w-NAF index
    var acc = this.jpoint(null, null, null);
    for (var i = naf.length - 1; i >= 0; i--) {
      // Count zeroes
      for (var l = 0; i >= 0 && naf[i] === 0; i--)
        l++;
      if (i >= 0)
        l++;
      acc = acc.dblp(l);

      if (i < 0)
        break;
      var z = naf[i];
      assert(z !== 0);
      if (p.type === 'affine') {
        // J +- P
        if (z > 0)
          acc = acc.mixedAdd(wnd[(z - 1) >> 1]);
        else
          acc = acc.mixedAdd(wnd[(-z - 1) >> 1].neg());
      } else {
        // J +- J
        if (z > 0)
          acc = acc.add(wnd[(z - 1) >> 1]);
        else
          acc = acc.add(wnd[(-z - 1) >> 1].neg());
      }
    }
    return p.type === 'affine' ? acc.toP() : acc;
  };

  BaseCurve.prototype._wnafMulAdd = function _wnafMulAdd(defW,
    points,
    coeffs,
    len,
    jacobianResult) {
    var wndWidth = this._wnafT1;
    var wnd = this._wnafT2;
    var naf = this._wnafT3;

    // Fill all arrays
    var max = 0;
    var i;
    var j;
    var p;
    for (i = 0; i < len; i++) {
      p = points[i];
      var nafPoints = p._getNAFPoints(defW);
      wndWidth[i] = nafPoints.wnd;
      wnd[i] = nafPoints.points;
    }

    // Comb small window NAFs
    for (i = len - 1; i >= 1; i -= 2) {
      var a = i - 1;
      var b = i;
      if (wndWidth[a] !== 1 || wndWidth[b] !== 1) {
        naf[a] = getNAF(coeffs[a], wndWidth[a], this._bitLength);
        naf[b] = getNAF(coeffs[b], wndWidth[b], this._bitLength);
        max = Math.max(naf[a].length, max);
        max = Math.max(naf[b].length, max);
        continue;
      }

      var comb = [
        points[a], /* 1 */
        null, /* 3 */
        null, /* 5 */
        points[b], /* 7 */
      ];

      // Try to avoid Projective points, if possible
      if (points[a].y.cmp(points[b].y) === 0) {
        comb[1] = points[a].add(points[b]);
        comb[2] = points[a].toJ().mixedAdd(points[b].neg());
      } else if (points[a].y.cmp(points[b].y.redNeg()) === 0) {
        comb[1] = points[a].toJ().mixedAdd(points[b]);
        comb[2] = points[a].add(points[b].neg());
      } else {
        comb[1] = points[a].toJ().mixedAdd(points[b]);
        comb[2] = points[a].toJ().mixedAdd(points[b].neg());
      }

      var index = [
        -3, /* -1 -1 */
        -1, /* -1 0 */
        -5, /* -1 1 */
        -7, /* 0 -1 */
        0, /* 0 0 */
        7, /* 0 1 */
        5, /* 1 -1 */
        1, /* 1 0 */
        3,  /* 1 1 */
      ];

      var jsf = getJSF(coeffs[a], coeffs[b]);
      max = Math.max(jsf[0].length, max);
      naf[a] = new Array(max);
      naf[b] = new Array(max);
      for (j = 0; j < max; j++) {
        var ja = jsf[0][j] | 0;
        var jb = jsf[1][j] | 0;

        naf[a][j] = index[(ja + 1) * 3 + (jb + 1)];
        naf[b][j] = 0;
        wnd[a] = comb;
      }
    }

    var acc = this.jpoint(null, null, null);
    var tmp = this._wnafT4;
    for (i = max; i >= 0; i--) {
      var k = 0;

      while (i >= 0) {
        var zero = true;
        for (j = 0; j < len; j++) {
          tmp[j] = naf[j][i] | 0;
          if (tmp[j] !== 0)
            zero = false;
        }
        if (!zero)
          break;
        k++;
        i--;
      }
      if (i >= 0)
        k++;
      acc = acc.dblp(k);
      if (i < 0)
        break;

      for (j = 0; j < len; j++) {
        var z = tmp[j];
        p;
        if (z === 0)
          continue;
        else if (z > 0)
          p = wnd[j][(z - 1) >> 1];
        else if (z < 0)
          p = wnd[j][(-z - 1) >> 1].neg();

        if (p.type === 'affine')
          acc = acc.mixedAdd(p);
        else
          acc = acc.add(p);
      }
    }
    // Zeroify references
    for (i = 0; i < len; i++)
      wnd[i] = null;

    if (jacobianResult)
      return acc;
    else
      return acc.toP();
  };

  function BasePoint(curve, type) {
    this.curve = curve;
    this.type = type;
    this.precomputed = null;
  }
  BaseCurve.BasePoint = BasePoint;

  BasePoint.prototype.validate = function validate() {
    return this.curve.validate(this);
  };

  BasePoint.prototype._encode = function _encode(compact) {
    var len = this.curve.p.byteLength();
    var x = this.getX().toArray(len);

    if (compact)
      return [ this.getY().isEven() ? 0x02 : 0x03 ].concat(x);

    return [ 0x04 ].concat(x, this.getY().toArray(len));
  };

  BasePoint.prototype.encode = function encode(enc, compact) {
    return utils.encode(this._encode(compact), enc);
  };

  BasePoint.prototype.precompute = function precompute(power) {
    if (this.precomputed)
      return this;

    var precomputed = {
      doubles: null,
      naf: null,
      beta: null,
    };
    precomputed.naf = this._getNAFPoints(8);
    precomputed.doubles = this._getDoubles(4, power);
    precomputed.beta = this._getBeta();
    this.precomputed = precomputed;

    return this;
  };

  BasePoint.prototype._hasDoubles = function _hasDoubles(k) {
    if (!this.precomputed)
      return false;

    var doubles = this.precomputed.doubles;
    if (!doubles)
      return false;

    return doubles.points.length >= Math.ceil((k.bitLength() + 1) / doubles.step);
  };

  BasePoint.prototype._getDoubles = function _getDoubles(step, power) {
    if (this.precomputed && this.precomputed.doubles)
      return this.precomputed.doubles;

    var doubles = [ this ];
    var acc = this;
    for (var i = 0; i < power; i += step) {
      for (var j = 0; j < step; j++)
        acc = acc.dbl();
      doubles.push(acc);
    }
    return {
      step: step,
      points: doubles,
    };
  };

  BasePoint.prototype._getNAFPoints = function _getNAFPoints(wnd) {
    if (this.precomputed && this.precomputed.naf)
      return this.precomputed.naf;

    var res = [ this ];
    var max = (1 << wnd) - 1;
    var dbl = max === 1 ? null : this.dbl();
    for (var i = 1; i < max; i++)
      res[i] = res[i - 1].add(dbl);
    return {
      wnd: wnd,
      points: res,
    };
  };

  BasePoint.prototype._getBeta = function _getBeta() {
    return null;
  };

  BasePoint.prototype.dblp = function dblp(k) {
    var r = this;
    for (var i = 0; i < k; i++)
      r = r.dbl();
    return r;
  };
})();

(function defineShortModule(){
  var assert = utils.assert;
  var Base = BaseCurve;

  window.ShortCurve = ShortCurve = function(conf) {
    Base.call(this, 'short', conf);

    this.a = new BN(conf.a, 16).toRed(this.red);
    this.b = new BN(conf.b, 16).toRed(this.red);
    this.tinv = this.two.redInvm();

    this.zeroA = this.a.fromRed().cmpn(0) === 0;
    this.threeA = this.a.fromRed().sub(this.p).cmpn(-3) === 0;

    // If the curve is endomorphic, precalculate beta and lambda
    this.endo = this._getEndomorphism(conf);
    this._endoWnafT1 = new Array(4);
    this._endoWnafT2 = new Array(4);
  };
  inherits(ShortCurve, Base);

  ShortCurve.prototype._getEndomorphism = function _getEndomorphism(conf) {
    // No efficient endomorphism
    if (!this.zeroA || !this.g || !this.n || this.p.modn(3) !== 1)
      return;

    // Compute beta and lambda, that lambda * P = (beta * Px; Py)
    var beta;
    var lambda;
    if (conf.beta) {
      beta = new BN(conf.beta, 16).toRed(this.red);
    } else {
      var betas = this._getEndoRoots(this.p);
      // Choose the smallest beta
      beta = betas[0].cmp(betas[1]) < 0 ? betas[0] : betas[1];
      beta = beta.toRed(this.red);
    }
    if (conf.lambda) {
      lambda = new BN(conf.lambda, 16);
    } else {
      // Choose the lambda that is matching selected beta
      var lambdas = this._getEndoRoots(this.n);
      if (this.g.mul(lambdas[0]).x.cmp(this.g.x.redMul(beta)) === 0) {
        lambda = lambdas[0];
      } else {
        lambda = lambdas[1];
        assert(this.g.mul(lambda).x.cmp(this.g.x.redMul(beta)) === 0);
      }
    }

    // Get basis vectors, used for balanced length-two representation
    var basis;
    if (conf.basis) {
      basis = conf.basis.map(function(vec) {
        return {
          a: new BN(vec.a, 16),
          b: new BN(vec.b, 16),
        };
      });
    } else {
      basis = this._getEndoBasis(lambda);
    }

    return {
      beta: beta,
      lambda: lambda,
      basis: basis,
    };
  };

  ShortCurve.prototype._getEndoRoots = function _getEndoRoots(num) {
    // Find roots of for x^2 + x + 1 in F
    // Root = (-1 +- Sqrt(-3)) / 2
    //
    var red = num === this.p ? this.red : BN.mont(num);
    var tinv = new BN(2).toRed(red).redInvm();
    var ntinv = tinv.redNeg();

    var s = new BN(3).toRed(red).redNeg().redSqrt().redMul(tinv);

    var l1 = ntinv.redAdd(s).fromRed();
    var l2 = ntinv.redSub(s).fromRed();
    return [ l1, l2 ];
  };

  ShortCurve.prototype._getEndoBasis = function _getEndoBasis(lambda) {
    // aprxSqrt >= sqrt(this.n)
    var aprxSqrt = this.n.ushrn(Math.floor(this.n.bitLength() / 2));

    // 3.74
    // Run EGCD, until r(L + 1) < aprxSqrt
    var u = lambda;
    var v = this.n.clone();
    var x1 = new BN(1);
    var y1 = new BN(0);
    var x2 = new BN(0);
    var y2 = new BN(1);

    // NOTE: all vectors are roots of: a + b * lambda = 0 (mod n)
    var a0;
    var b0;
    // First vector
    var a1;
    var b1;
    // Second vector
    var a2;
    var b2;

    var prevR;
    var i = 0;
    var r;
    var x;
    while (u.cmpn(0) !== 0) {
      var q = v.div(u);
      r = v.sub(q.mul(u));
      x = x2.sub(q.mul(x1));
      var y = y2.sub(q.mul(y1));

      if (!a1 && r.cmp(aprxSqrt) < 0) {
        a0 = prevR.neg();
        b0 = x1;
        a1 = r.neg();
        b1 = x;
      } else if (a1 && ++i === 2) {
        break;
      }
      prevR = r;

      v = u;
      u = r;
      x2 = x1;
      x1 = x;
      y2 = y1;
      y1 = y;
    }
    a2 = r.neg();
    b2 = x;

    var len1 = a1.sqr().add(b1.sqr());
    var len2 = a2.sqr().add(b2.sqr());
    if (len2.cmp(len1) >= 0) {
      a2 = a0;
      b2 = b0;
    }

    // Normalize signs
    if (a1.negative) {
      a1 = a1.neg();
      b1 = b1.neg();
    }
    if (a2.negative) {
      a2 = a2.neg();
      b2 = b2.neg();
    }

    return [
      { a: a1, b: b1 },
      { a: a2, b: b2 },
    ];
  };

  ShortCurve.prototype._endoSplit = function _endoSplit(k) {
    var basis = this.endo.basis;
    var v1 = basis[0];
    var v2 = basis[1];

    var c1 = v2.b.mul(k).divRound(this.n);
    var c2 = v1.b.neg().mul(k).divRound(this.n);

    var p1 = c1.mul(v1.a);
    var p2 = c2.mul(v2.a);
    var q1 = c1.mul(v1.b);
    var q2 = c2.mul(v2.b);

    // Calculate answer
    var k1 = k.sub(p1).sub(p2);
    var k2 = q1.add(q2).neg();
    return { k1: k1, k2: k2 };
  };

  ShortCurve.prototype.pointFromX = function pointFromX(x, odd) {
    x = new BN(x, 16);
    if (!x.red)
      x = x.toRed(this.red);

    var y2 = x.redSqr().redMul(x).redIAdd(x.redMul(this.a)).redIAdd(this.b);
    var y = y2.redSqrt();
    if (y.redSqr().redSub(y2).cmp(this.zero) !== 0)
      throw new Error('invalid point');

    // XXX Is there any way to tell if the number is odd without converting it
    // to non-red form?
    var isOdd = y.fromRed().isOdd();
    if (odd && !isOdd || !odd && isOdd)
      y = y.redNeg();

    return this.point(x, y);
  };

  ShortCurve.prototype.validate = function validate(point) {
    if (point.inf)
      return true;

    var x = point.x;
    var y = point.y;

    var ax = this.a.redMul(x);
    var rhs = x.redSqr().redMul(x).redIAdd(ax).redIAdd(this.b);
    return y.redSqr().redISub(rhs).cmpn(0) === 0;
  };

  ShortCurve.prototype._endoWnafMulAdd = function _endoWnafMulAdd(points, coeffs, jacobianResult) {
    var npoints = this._endoWnafT1;
    var ncoeffs = this._endoWnafT2;
    for (var i = 0; i < points.length; i++) {
      var split = this._endoSplit(coeffs[i]);
      var p = points[i];
      var beta = p._getBeta();

      if (split.k1.negative) {
        split.k1.ineg();
        p = p.neg(true);
      }
      if (split.k2.negative) {
        split.k2.ineg();
        beta = beta.neg(true);
      }

      npoints[i * 2] = p;
      npoints[i * 2 + 1] = beta;
      ncoeffs[i * 2] = split.k1;
      ncoeffs[i * 2 + 1] = split.k2;
    }
    var res = this._wnafMulAdd(1, npoints, ncoeffs, i * 2, jacobianResult);

    // Clean-up references to points and coefficients
    for (var j = 0; j < i * 2; j++) {
      npoints[j] = null;
      ncoeffs[j] = null;
    }
    return res;
  };

  function Point(curve, x, y, isRed) {
    Base.BasePoint.call(this, curve, 'affine');
    if (x === null && y === null) {
      this.x = null;
      this.y = null;
      this.inf = true;
    } else {
      this.x = new BN(x, 16);
      this.y = new BN(y, 16);
      // Force redgomery representation when loading from JSON
      if (isRed) {
        this.x.forceRed(this.curve.red);
        this.y.forceRed(this.curve.red);
      }
      if (!this.x.red)
        this.x = this.x.toRed(this.curve.red);
      if (!this.y.red)
        this.y = this.y.toRed(this.curve.red);
      this.inf = false;
    }
  }
  inherits(Point, Base.BasePoint);

  ShortCurve.prototype.point = function point(x, y, isRed) {
    return new Point(this, x, y, isRed);
  };

  ShortCurve.prototype.pointFromJSON = function pointFromJSON(obj, red) {
    return Point.fromJSON(this, obj, red);
  };

  Point.prototype._getBeta = function _getBeta() {
    if (!this.curve.endo)
      return;

    var pre = this.precomputed;
    if (pre && pre.beta)
      return pre.beta;

    var beta = this.curve.point(this.x.redMul(this.curve.endo.beta), this.y);
    if (pre) {
      var curve = this.curve;
      var endoMul = function(p) {
        return curve.point(p.x.redMul(curve.endo.beta), p.y);
      };
      pre.beta = beta;
      beta.precomputed = {
        beta: null,
        naf: pre.naf && {
          wnd: pre.naf.wnd,
          points: pre.naf.points.map(endoMul),
        },
        doubles: pre.doubles && {
          step: pre.doubles.step,
          points: pre.doubles.points.map(endoMul),
        },
      };
    }
    return beta;
  };

  Point.fromJSON = function fromJSON(curve, obj, red) {
    if (typeof obj === 'string')
      obj = JSON.parse(obj);
    var res = curve.point(obj[0], obj[1], red);
    if (!obj[2])
      return res;

    function obj2point(obj) {
      return curve.point(obj[0], obj[1], red);
    }

    var pre = obj[2];
    res.precomputed = {
      beta: null,
      doubles: pre.doubles && {
        step: pre.doubles.step,
        points: [ res ].concat(pre.doubles.points.map(obj2point)),
      },
      naf: pre.naf && {
        wnd: pre.naf.wnd,
        points: [ res ].concat(pre.naf.points.map(obj2point)),
      },
    };
    return res;
  };

  Point.prototype.isInfinity = function isInfinity() {
    return this.inf;
  };

  Point.prototype.add = function add(p) {
    // O + P = P
    if (this.inf)
      return p;

    // P + O = P
    if (p.inf)
      return this;

    // P + P = 2P
    if (this.eq(p))
      return this.dbl();

    // P + (-P) = O
    if (this.neg().eq(p))
      return this.curve.point(null, null);

    // P + Q = O
    if (this.x.cmp(p.x) === 0)
      return this.curve.point(null, null);

    var c = this.y.redSub(p.y);
    if (c.cmpn(0) !== 0)
      c = c.redMul(this.x.redSub(p.x).redInvm());
    var nx = c.redSqr().redISub(this.x).redISub(p.x);
    var ny = c.redMul(this.x.redSub(nx)).redISub(this.y);
    return this.curve.point(nx, ny);
  };

  Point.prototype.dbl = function dbl() {
    if (this.inf)
      return this;

    // 2P = O
    var ys1 = this.y.redAdd(this.y);
    if (ys1.cmpn(0) === 0)
      return this.curve.point(null, null);

    var a = this.curve.a;

    var x2 = this.x.redSqr();
    var dyinv = ys1.redInvm();
    var c = x2.redAdd(x2).redIAdd(x2).redIAdd(a).redMul(dyinv);

    var nx = c.redSqr().redISub(this.x.redAdd(this.x));
    var ny = c.redMul(this.x.redSub(nx)).redISub(this.y);
    return this.curve.point(nx, ny);
  };

  Point.prototype.getX = function getX() {
    return this.x.fromRed();
  };

  Point.prototype.getY = function getY() {
    return this.y.fromRed();
  };

  Point.prototype.mul = function mul(k) {
    k = new BN(k, 16);
    if (this.isInfinity())
      return this;
    else if (this._hasDoubles(k))
      return this.curve._fixedNafMul(this, k);
    else if (this.curve.endo)
      return this.curve._endoWnafMulAdd([ this ], [ k ]);
    else
      return this.curve._wnafMul(this, k);
  };

  Point.prototype.mulAdd = function mulAdd(k1, p2, k2) {
    var points = [ this, p2 ];
    var coeffs = [ k1, k2 ];
    if (this.curve.endo)
      return this.curve._endoWnafMulAdd(points, coeffs);
    else
      return this.curve._wnafMulAdd(1, points, coeffs, 2);
  };

  Point.prototype.eq = function eq(p) {
    return this === p ||
           this.inf === p.inf &&
               (this.inf || this.x.cmp(p.x) === 0 && this.y.cmp(p.y) === 0);
  };

  Point.prototype.neg = function neg(_precompute) {
    if (this.inf)
      return this;

    var res = this.curve.point(this.x, this.y.redNeg());
    if (_precompute && this.precomputed) {
      var pre = this.precomputed;
      var negate = function(p) {
        return p.neg();
      };
      res.precomputed = {
        naf: pre.naf && {
          wnd: pre.naf.wnd,
          points: pre.naf.points.map(negate),
        },
        doubles: pre.doubles && {
          step: pre.doubles.step,
          points: pre.doubles.points.map(negate),
        },
      };
    }
    return res;
  };

  Point.prototype.toJ = function toJ() {
    if (this.inf)
      return this.curve.jpoint(null, null, null);

    var res = this.curve.jpoint(this.x, this.y, this.curve.one);
    return res;
  };

  function JPoint(curve, x, y, z) {
    Base.BasePoint.call(this, curve, 'jacobian');
    if (x === null && y === null && z === null) {
      this.x = this.curve.one;
      this.y = this.curve.one;
      this.z = new BN(0);
    } else {
      this.x = new BN(x, 16);
      this.y = new BN(y, 16);
      this.z = new BN(z, 16);
    }
    if (!this.x.red)
      this.x = this.x.toRed(this.curve.red);
    if (!this.y.red)
      this.y = this.y.toRed(this.curve.red);
    if (!this.z.red)
      this.z = this.z.toRed(this.curve.red);

    this.zOne = this.z === this.curve.one;
  }
  inherits(JPoint, Base.BasePoint);

  ShortCurve.prototype.jpoint = function jpoint(x, y, z) {
    return new JPoint(this, x, y, z);
  };

  JPoint.prototype.toP = function toP() {
    if (this.isInfinity())
      return this.curve.point(null, null);

    var zinv = this.z.redInvm();
    var zinv2 = zinv.redSqr();
    var ax = this.x.redMul(zinv2);
    var ay = this.y.redMul(zinv2).redMul(zinv);

    return this.curve.point(ax, ay);
  };

  JPoint.prototype.neg = function neg() {
    return this.curve.jpoint(this.x, this.y.redNeg(), this.z);
  };

  JPoint.prototype.add = function add(p) {
    // O + P = P
    if (this.isInfinity())
      return p;

    // P + O = P
    if (p.isInfinity())
      return this;

    // 12M + 4S + 7A
    var pz2 = p.z.redSqr();
    var z2 = this.z.redSqr();
    var u1 = this.x.redMul(pz2);
    var u2 = p.x.redMul(z2);
    var s1 = this.y.redMul(pz2.redMul(p.z));
    var s2 = p.y.redMul(z2.redMul(this.z));

    var h = u1.redSub(u2);
    var r = s1.redSub(s2);
    if (h.cmpn(0) === 0) {
      if (r.cmpn(0) !== 0)
        return this.curve.jpoint(null, null, null);
      else
        return this.dbl();
    }

    var h2 = h.redSqr();
    var h3 = h2.redMul(h);
    var v = u1.redMul(h2);

    var nx = r.redSqr().redIAdd(h3).redISub(v).redISub(v);
    var ny = r.redMul(v.redISub(nx)).redISub(s1.redMul(h3));
    var nz = this.z.redMul(p.z).redMul(h);

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype.mixedAdd = function mixedAdd(p) {
    // O + P = P
    if (this.isInfinity())
      return p.toJ();

    // P + O = P
    if (p.isInfinity())
      return this;

    // 8M + 3S + 7A
    var z2 = this.z.redSqr();
    var u1 = this.x;
    var u2 = p.x.redMul(z2);
    var s1 = this.y;
    var s2 = p.y.redMul(z2).redMul(this.z);

    var h = u1.redSub(u2);
    var r = s1.redSub(s2);
    if (h.cmpn(0) === 0) {
      if (r.cmpn(0) !== 0)
        return this.curve.jpoint(null, null, null);
      else
        return this.dbl();
    }

    var h2 = h.redSqr();
    var h3 = h2.redMul(h);
    var v = u1.redMul(h2);

    var nx = r.redSqr().redIAdd(h3).redISub(v).redISub(v);
    var ny = r.redMul(v.redISub(nx)).redISub(s1.redMul(h3));
    var nz = this.z.redMul(h);

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype.dblp = function dblp(pow) {
    if (pow === 0)
      return this;
    if (this.isInfinity())
      return this;
    if (!pow)
      return this.dbl();

    var i;
    if (this.curve.zeroA || this.curve.threeA) {
      var r = this;
      for (i = 0; i < pow; i++)
        r = r.dbl();
      return r;
    }

    // 1M + 2S + 1A + N * (4S + 5M + 8A)
    // N = 1 => 6M + 6S + 9A
    var a = this.curve.a;
    var tinv = this.curve.tinv;

    var jx = this.x;
    var jy = this.y;
    var jz = this.z;
    var jz4 = jz.redSqr().redSqr();

    // Reuse results
    var jyd = jy.redAdd(jy);
    for (i = 0; i < pow; i++) {
      var jx2 = jx.redSqr();
      var jyd2 = jyd.redSqr();
      var jyd4 = jyd2.redSqr();
      var c = jx2.redAdd(jx2).redIAdd(jx2).redIAdd(a.redMul(jz4));

      var t1 = jx.redMul(jyd2);
      var nx = c.redSqr().redISub(t1.redAdd(t1));
      var t2 = t1.redISub(nx);
      var dny = c.redMul(t2);
      dny = dny.redIAdd(dny).redISub(jyd4);
      var nz = jyd.redMul(jz);
      if (i + 1 < pow)
        jz4 = jz4.redMul(jyd4);

      jx = nx;
      jz = nz;
      jyd = dny;
    }

    return this.curve.jpoint(jx, jyd.redMul(tinv), jz);
  };

  JPoint.prototype.dbl = function dbl() {
    if (this.isInfinity())
      return this;

    if (this.curve.zeroA)
      return this._zeroDbl();
    else if (this.curve.threeA)
      return this._threeDbl();
    else
      return this._dbl();
  };

  JPoint.prototype._zeroDbl = function _zeroDbl() {
    var nx;
    var ny;
    var nz;
    // Z = 1
    if (this.zOne) {
      // hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html
      //     #doubling-mdbl-2007-bl
      // 1M + 5S + 14A

      // XX = X1^2
      var xx = this.x.redSqr();
      // YY = Y1^2
      var yy = this.y.redSqr();
      // YYYY = YY^2
      var yyyy = yy.redSqr();
      // S = 2 * ((X1 + YY)^2 - XX - YYYY)
      var s = this.x.redAdd(yy).redSqr().redISub(xx).redISub(yyyy);
      s = s.redIAdd(s);
      // M = 3 * XX + a; a = 0
      var m = xx.redAdd(xx).redIAdd(xx);
      // T = M ^ 2 - 2*S
      var t = m.redSqr().redISub(s).redISub(s);

      // 8 * YYYY
      var yyyy8 = yyyy.redIAdd(yyyy);
      yyyy8 = yyyy8.redIAdd(yyyy8);
      yyyy8 = yyyy8.redIAdd(yyyy8);

      // X3 = T
      nx = t;
      // Y3 = M * (S - T) - 8 * YYYY
      ny = m.redMul(s.redISub(t)).redISub(yyyy8);
      // Z3 = 2*Y1
      nz = this.y.redAdd(this.y);
    } else {
      // hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html
      //     #doubling-dbl-2009-l
      // 2M + 5S + 13A

      // A = X1^2
      var a = this.x.redSqr();
      // B = Y1^2
      var b = this.y.redSqr();
      // C = B^2
      var c = b.redSqr();
      // D = 2 * ((X1 + B)^2 - A - C)
      var d = this.x.redAdd(b).redSqr().redISub(a).redISub(c);
      d = d.redIAdd(d);
      // E = 3 * A
      var e = a.redAdd(a).redIAdd(a);
      // F = E^2
      var f = e.redSqr();

      // 8 * C
      var c8 = c.redIAdd(c);
      c8 = c8.redIAdd(c8);
      c8 = c8.redIAdd(c8);

      // X3 = F - 2 * D
      nx = f.redISub(d).redISub(d);
      // Y3 = E * (D - X3) - 8 * C
      ny = e.redMul(d.redISub(nx)).redISub(c8);
      // Z3 = 2 * Y1 * Z1
      nz = this.y.redMul(this.z);
      nz = nz.redIAdd(nz);
    }

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype._threeDbl = function _threeDbl() {
    var nx;
    var ny;
    var nz;
    // Z = 1
    if (this.zOne) {
      // hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-3.html
      //     #doubling-mdbl-2007-bl
      // 1M + 5S + 15A

      // XX = X1^2
      var xx = this.x.redSqr();
      // YY = Y1^2
      var yy = this.y.redSqr();
      // YYYY = YY^2
      var yyyy = yy.redSqr();
      // S = 2 * ((X1 + YY)^2 - XX - YYYY)
      var s = this.x.redAdd(yy).redSqr().redISub(xx).redISub(yyyy);
      s = s.redIAdd(s);
      // M = 3 * XX + a
      var m = xx.redAdd(xx).redIAdd(xx).redIAdd(this.curve.a);
      // T = M^2 - 2 * S
      var t = m.redSqr().redISub(s).redISub(s);
      // X3 = T
      nx = t;
      // Y3 = M * (S - T) - 8 * YYYY
      var yyyy8 = yyyy.redIAdd(yyyy);
      yyyy8 = yyyy8.redIAdd(yyyy8);
      yyyy8 = yyyy8.redIAdd(yyyy8);
      ny = m.redMul(s.redISub(t)).redISub(yyyy8);
      // Z3 = 2 * Y1
      nz = this.y.redAdd(this.y);
    } else {
      // hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-3.html#doubling-dbl-2001-b
      // 3M + 5S

      // delta = Z1^2
      var delta = this.z.redSqr();
      // gamma = Y1^2
      var gamma = this.y.redSqr();
      // beta = X1 * gamma
      var beta = this.x.redMul(gamma);
      // alpha = 3 * (X1 - delta) * (X1 + delta)
      var alpha = this.x.redSub(delta).redMul(this.x.redAdd(delta));
      alpha = alpha.redAdd(alpha).redIAdd(alpha);
      // X3 = alpha^2 - 8 * beta
      var beta4 = beta.redIAdd(beta);
      beta4 = beta4.redIAdd(beta4);
      var beta8 = beta4.redAdd(beta4);
      nx = alpha.redSqr().redISub(beta8);
      // Z3 = (Y1 + Z1)^2 - gamma - delta
      nz = this.y.redAdd(this.z).redSqr().redISub(gamma).redISub(delta);
      // Y3 = alpha * (4 * beta - X3) - 8 * gamma^2
      var ggamma8 = gamma.redSqr();
      ggamma8 = ggamma8.redIAdd(ggamma8);
      ggamma8 = ggamma8.redIAdd(ggamma8);
      ggamma8 = ggamma8.redIAdd(ggamma8);
      ny = alpha.redMul(beta4.redISub(nx)).redISub(ggamma8);
    }

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype._dbl = function _dbl() {
    var a = this.curve.a;

    // 4M + 6S + 10A
    var jx = this.x;
    var jy = this.y;
    var jz = this.z;
    var jz4 = jz.redSqr().redSqr();

    var jx2 = jx.redSqr();
    var jy2 = jy.redSqr();

    var c = jx2.redAdd(jx2).redIAdd(jx2).redIAdd(a.redMul(jz4));

    var jxd4 = jx.redAdd(jx);
    jxd4 = jxd4.redIAdd(jxd4);
    var t1 = jxd4.redMul(jy2);
    var nx = c.redSqr().redISub(t1.redAdd(t1));
    var t2 = t1.redISub(nx);

    var jyd8 = jy2.redSqr();
    jyd8 = jyd8.redIAdd(jyd8);
    jyd8 = jyd8.redIAdd(jyd8);
    jyd8 = jyd8.redIAdd(jyd8);
    var ny = c.redMul(t2).redISub(jyd8);
    var nz = jy.redAdd(jy).redMul(jz);

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype.trpl = function trpl() {
    if (!this.curve.zeroA)
      return this.dbl().add(this);

    // hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#tripling-tpl-2007-bl
    // 5M + 10S + ...

    // XX = X1^2
    var xx = this.x.redSqr();
    // YY = Y1^2
    var yy = this.y.redSqr();
    // ZZ = Z1^2
    var zz = this.z.redSqr();
    // YYYY = YY^2
    var yyyy = yy.redSqr();
    // M = 3 * XX + a * ZZ2; a = 0
    var m = xx.redAdd(xx).redIAdd(xx);
    // MM = M^2
    var mm = m.redSqr();
    // E = 6 * ((X1 + YY)^2 - XX - YYYY) - MM
    var e = this.x.redAdd(yy).redSqr().redISub(xx).redISub(yyyy);
    e = e.redIAdd(e);
    e = e.redAdd(e).redIAdd(e);
    e = e.redISub(mm);
    // EE = E^2
    var ee = e.redSqr();
    // T = 16*YYYY
    var t = yyyy.redIAdd(yyyy);
    t = t.redIAdd(t);
    t = t.redIAdd(t);
    t = t.redIAdd(t);
    // U = (M + E)^2 - MM - EE - T
    var u = m.redIAdd(e).redSqr().redISub(mm).redISub(ee).redISub(t);
    // X3 = 4 * (X1 * EE - 4 * YY * U)
    var yyu4 = yy.redMul(u);
    yyu4 = yyu4.redIAdd(yyu4);
    yyu4 = yyu4.redIAdd(yyu4);
    var nx = this.x.redMul(ee).redISub(yyu4);
    nx = nx.redIAdd(nx);
    nx = nx.redIAdd(nx);
    // Y3 = 8 * Y1 * (U * (T - U) - E * EE)
    var ny = this.y.redMul(u.redMul(t.redISub(u)).redISub(e.redMul(ee)));
    ny = ny.redIAdd(ny);
    ny = ny.redIAdd(ny);
    ny = ny.redIAdd(ny);
    // Z3 = (Z1 + E)^2 - ZZ - EE
    var nz = this.z.redAdd(e).redSqr().redISub(zz).redISub(ee);

    return this.curve.jpoint(nx, ny, nz);
  };

  JPoint.prototype.mul = function mul(k, kbase) {
    k = new BN(k, kbase);

    return this.curve._wnafMul(this, k);
  };

  JPoint.prototype.eq = function eq(p) {
    if (p.type === 'affine')
      return this.eq(p.toJ());

    if (this === p)
      return true;

    // x1 * z2^2 == x2 * z1^2
    var z2 = this.z.redSqr();
    var pz2 = p.z.redSqr();
    if (this.x.redMul(pz2).redISub(p.x.redMul(z2)).cmpn(0) !== 0)
      return false;

    // y1 * z2^3 == y2 * z1^3
    var z3 = z2.redMul(this.z);
    var pz3 = pz2.redMul(p.z);
    return this.y.redMul(pz3).redISub(p.y.redMul(z3)).cmpn(0) === 0;
  };

  JPoint.prototype.isInfinity = function isInfinity() {
    // XXX This code assumes that zero is always zero in red
    return this.z.cmpn(0) === 0;
  };
})();

(function defineCurvesModule(){
  window.curves = curves = {};

  function PresetCurve(options) {
    this.curve = new ShortCurve(options);
    this.g = this.curve.g;
    this.n = this.curve.n;
  }
  curves.PresetCurve = PresetCurve;

  function defineCurve(name, options) {
    Object.defineProperty(curves, name, {
      configurable: true,
      enumerable: true,
      get: function() {
        var curve = new PresetCurve(options);
        Object.defineProperty(curves, name, {
          configurable: true,
          enumerable: true,
          value: curve,
        });
        return curve;
      },
    });
  }

  defineCurve('secp256k1', {
    type: 'short',
    prime: 'k256',
    p: 'ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff fffffffe fffffc2f',
    a: '0',
    b: '7',
    n: 'ffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141',
    h: '1',

    // Precomputed endomorphism
    beta: '7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee',
    lambda: '5363ad4cc05c30e0a5261c028812645a122e22ea20816678df02967c1b23bd72',
    basis: [
      {
        a: '3086d221a7d46bcde86c90e49284eb15',
        b: '-e4437ed6010e88286f547fa90abfe4c3',
      },
      {
        a: '114ca50f7a8e2f3f657c1108d9d44cfd8',
        b: '3086d221a7d46bcde86c90e49284eb15',
      },
    ],

    gRed: false,
    g: [
      '79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798',
      '483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8',
      {},
    ],
  });
})();

(function defineEllipticModule(){
  function Signature(options, enc) {
    if (options instanceof Signature)
      return options;

    this.r = new BN(options.r, 16);
    this.s = new BN(options.s, 16);
    if (options.recoveryParam === undefined)
      this.recoveryParam = null;
    else
      this.recoveryParam = options.recoveryParam;
  }

  window.EC = EC = function(options) {
    if (!(this instanceof EC))
      return new EC(options);

    options = curves[options];

    // Shortcut for `elliptic.ec(elliptic.curves.curveName)`
    if (options instanceof curves.PresetCurve)
      options = { curve: options };

    this.curve = options.curve.curve;
    this.n = this.curve.n;
    this.nh = this.n.ushrn(1);
    this.g = this.curve.g;

    // Point on curve
    this.g = options.curve.g;
    this.g.precompute(options.curve.n.bitLength() + 1);

    // Hash for function for DRBG
    this.hash = options.hash || options.curve.hash;
  }

  EC.prototype.recoverPubKey = function(msg, signature, j, enc) {
    signature = new Signature(signature, enc);

    var n = this.n;
    var e = new BN(msg);
    var r = signature.r;
    var s = signature.s;

    // A set LSB signifies that the y-coordinate is odd
    var isYOdd = j & 1;
    var isSecondKey = j >> 1;

    // 1.1. Let x = r + jn.
    if (isSecondKey)
      r = this.curve.pointFromX(r.add(this.curve.n), isYOdd);
    else
      r = this.curve.pointFromX(r, isYOdd);

    var rInv = signature.r.invm(n);
    var s1 = n.sub(e).mul(rInv).umod(n);
    var s2 = s.mul(rInv).umod(n);

    // 1.6.1 Compute Q = r^-1 (sR -  eG)
    //               Q = r^-1 (sR + -eG)
    return this.g.mulAdd(s1, r, s2);
  };

  EC.prototype.getKeyRecoveryParam = function(e, signature, Q, enc) {
    signature = new Signature(signature, enc);
    if (signature.recoveryParam !== null)
      return signature.recoveryParam;

    for (var i = 0; i < 4; i++) {
      var Qprime;
      try {
        Qprime = this.recoverPubKey(e, signature, i);
      } catch (e) {
        continue;
      }

      if (Qprime.eq(Q))
        return i;
    }
  };
})();

(function defineSecp256k1Module(){
  var ec = new EC('secp256k1')
  var ecparams = ec.curve

  window.secp256k1 = {
    "recover": function (message, signature, recovery, compressed) {
      var sigObj = { r: signature.slice(0, 32), s: signature.slice(32, 64) }

      var sigr = new BN(sigObj.r)
      var sigs = new BN(sigObj.s)
      if (sigr.cmp(ecparams.n) >= 0 || sigs.cmp(ecparams.n) >= 0) throw new Error("ECDSA_SIGNATURE_PARSE_FAIL")

      if (sigr.isZero() || sigs.isZero()) throw new Error()

      var point = ec.recoverPubKey(message, sigObj, recovery)
      return Buffer.from(point.encode(true, compressed))
    }
  }
})();

(function defineBs58CheckModule(){
  var ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  var BASE_MAP = new Uint8Array(256)
  for (var i = 0; i < ALPHABET.length; i++) {
    var x = ALPHABET.charAt(i)
    var xc = x.charCodeAt(0)
    BASE_MAP[xc] = i
  }
  var LEADER = ALPHABET.charAt(0)
  var FACTOR = Math.log(58) / Math.log(256)
  var iFACTOR = Math.log(256) / Math.log(58)

  function base58decode(source) {
    if (source.length === 0) { return Buffer.alloc(0) }
    var psz = 0
    if (source[psz] === ' ') { return }
    var zeroes = 0
    var length = 0
    while (source[psz] === LEADER) {
      zeroes++
      psz++
    }
    var size = (((source.length - psz) * FACTOR) + 1) >>> 0;
    var b256 = new Uint8Array(size)
    while (source[psz]) {
      var carry = BASE_MAP[source.charCodeAt(psz)]
      if (carry === 255) { return }
      var i = 0
      for (var it3 = size - 1; (carry !== 0 || i < length) && (it3 !== -1); it3--, i++) {
        carry += (58 * b256[it3]) >>> 0
        b256[it3] = (carry % 256) >>> 0
        carry = (carry / 256) >>> 0
      }
      if (carry !== 0) { throw new Error('Non-zero carry') }
      length = i
      psz++
    }
    if (source[psz] === ' ') { return }
    var it4 = size - length
    while (it4 !== size && b256[it4] === 0) {
      it4++
    }
    var vch = Buffer.allocUnsafe(zeroes + (size - it4))
    vch.fill(0x00, 0, zeroes)
    var j = zeroes
    while (it4 !== size) {
      vch[j++] = b256[it4++]
    }
    return vch
  }

  window.bs58check = function(string) {
    var buffer = base58decode(string)
    var payload = buffer.slice(0, -4)
    var checksum = buffer.slice(-4)
    var once = createHash('sha256').update(payload).digest()
    var newChecksum = createHash('sha256').update(once).digest()

    if (checksum[0] ^ newChecksum[0] |
        checksum[1] ^ newChecksum[1] |
        checksum[2] ^ newChecksum[2] |
        checksum[3] ^ newChecksum[3]) return

    return payload;
  }
})();

(function defineVaruintBitcoinModule(){
  window.varuint = {
    "encode": function(number, buffer, offset) {
      if (!buffer) buffer = Buffer.allocUnsafe(encodingLength(number))
      if (!Buffer.isBuffer(buffer)) throw new TypeError('buffer must be a Buffer instance')
      if (!offset) offset = 0

      if (number < 0xfd) {
        buffer.writeUInt8(number, offset)
      } else if (number <= 0xffff) {
        buffer.writeUInt8(0xfd, offset)
        buffer.writeUInt16LE(number, offset + 1)
      } else if (number <= 0xffffffff) {
        buffer.writeUInt8(0xfe, offset)
        buffer.writeUInt32LE(number, offset + 1)
      } else {
        buffer.writeUInt8(0xff, offset)
        buffer.writeUInt32LE(number >>> 0, offset + 1)
        buffer.writeUInt32LE((number / 0x100000000) | 0, offset + 5)
      }
      return buffer
    },
    "encodingLength": function(number){
      return (
        number < 0xfd ? 1
          : number <= 0xffff ? 3
            : number <= 0xffffffff ? 5
              : 9
      )
    }
  };
})();

(function defineBech32Module(){
  const ALPHABET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
  const ALPHABET_MAP = {};
  for (let z = 0; z < ALPHABET.length; z++) {
      const x = ALPHABET.charAt(z);
      ALPHABET_MAP[x] = z;
  }
  function polymodStep(pre) {
      const b = pre >> 25;
      return (((pre & 0x1ffffff) << 5) ^
          (-((b >> 0) & 1) & 0x3b6a57b2) ^
          (-((b >> 1) & 1) & 0x26508e6d) ^
          (-((b >> 2) & 1) & 0x1ea119fa) ^
          (-((b >> 3) & 1) & 0x3d4233dd) ^
          (-((b >> 4) & 1) & 0x2a1462b3));
  }
  function prefixChk(prefix) {
      let chk = 1;
      for (let i = 0; i < prefix.length; ++i) {
          const c = prefix.charCodeAt(i);
          if (c < 33 || c > 126)
              return 'Invalid prefix (' + prefix + ')';
          chk = polymodStep(chk) ^ (c >> 5);
      }
      chk = polymodStep(chk);
      for (let i = 0; i < prefix.length; ++i) {
          const v = prefix.charCodeAt(i);
          chk = polymodStep(chk) ^ (v & 0x1f);
      }
      return chk;
  }
  function convert(data, inBits, outBits, pad) {
      let value = 0;
      let bits = 0;
      const maxV = (1 << outBits) - 1;
      const result = [];
      for (let i = 0; i < data.length; ++i) {
          value = (value << inBits) | data[i];
          bits += inBits;
          while (bits >= outBits) {
              bits -= outBits;
              result.push((value >> bits) & maxV);
          }
      }
      if (pad) {
          if (bits > 0) {
              result.push((value << (outBits - bits)) & maxV);
          }
      }
      else {
          if (bits >= inBits)
              return 'Excess padding';
          if ((value << (outBits - bits)) & maxV)
              return 'Non-zero padding';
      }
      return result;
  }
  function toWords(bytes) {
      return convert(bytes, 8, 5, true);
  }
  function fromWordsUnsafe(words) {
      const res = convert(words, 5, 8, false);
      if (Array.isArray(res))
          return res;
  }
  function fromWords(words) {
      const res = convert(words, 5, 8, false);
      if (Array.isArray(res))
          return res;
      throw new Error(res);
  }
  function getLibraryFromEncoding(encoding) {
      let ENCODING_CONST;
      if (encoding === 'bech32') {
          ENCODING_CONST = 1;
      }
      else {
          ENCODING_CONST = 0x2bc830a3;
      }
      function encode(prefix, words, LIMIT) {
          LIMIT = LIMIT || 90;
          if (prefix.length + 7 + words.length > LIMIT)
              throw new TypeError('Exceeds length limit');
          prefix = prefix.toLowerCase();
          // determine chk mod
          let chk = prefixChk(prefix);
          if (typeof chk === 'string')
              throw new Error(chk);
          let result = prefix + '1';
          for (let i = 0; i < words.length; ++i) {
              const x = words[i];
              if (x >> 5 !== 0)
                  throw new Error('Non 5-bit word');
              chk = polymodStep(chk) ^ x;
              result += ALPHABET.charAt(x);
          }
          for (let i = 0; i < 6; ++i) {
              chk = polymodStep(chk);
          }
          chk ^= ENCODING_CONST;
          for (let i = 0; i < 6; ++i) {
              const v = (chk >> ((5 - i) * 5)) & 0x1f;
              result += ALPHABET.charAt(v);
          }
          return result;
      }
      function __decode(str, LIMIT) {
          LIMIT = LIMIT || 90;
          if (str.length < 8)
              return str + ' too short';
          if (str.length > LIMIT)
              return 'Exceeds length limit';
          // don't allow mixed case
          const lowered = str.toLowerCase();
          const uppered = str.toUpperCase();
          if (str !== lowered && str !== uppered)
              return 'Mixed-case string ' + str;
          str = lowered;
          const split = str.lastIndexOf('1');
          if (split === -1)
              return 'No separator character for ' + str;
          if (split === 0)
              return 'Missing prefix for ' + str;
          const prefix = str.slice(0, split);
          const wordChars = str.slice(split + 1);
          if (wordChars.length < 6)
              return 'Data too short';
          let chk = prefixChk(prefix);
          if (typeof chk === 'string')
              return chk;
          const words = [];
          for (let i = 0; i < wordChars.length; ++i) {
              const c = wordChars.charAt(i);
              const v = ALPHABET_MAP[c];
              if (v === undefined)
                  return 'Unknown character ' + c;
              chk = polymodStep(chk) ^ v;
              // not in the checksum?
              if (i + 6 >= wordChars.length)
                  continue;
              words.push(v);
          }
          if (chk !== ENCODING_CONST)
              return 'Invalid checksum for ' + str;
          return { prefix, words };
      }
      function decodeUnsafe(str, LIMIT) {
          const res = __decode(str, LIMIT);
          if (typeof res === 'object')
              return res;
      }
      function decode(str, LIMIT) {
          const res = __decode(str, LIMIT);
          if (typeof res === 'object')
              return res;
          throw new Error(res);
      }
      return {
          decodeUnsafe,
          decode,
          encode,
          toWords,
          fromWordsUnsafe,
          fromWords,
      };
  }
  window.bech32 = getLibraryFromEncoding('bech32');
})();

(function defineBitcoinJsMessageModule(){
  function sha256 (b) {
    return createHash('sha256')
      .update(b)
      .digest()
  }
  function hash256 (buffer) {
    return sha256(sha256(buffer))
  }
  function hash160 (buffer) {
    return createHash('ripemd160')
      .update(sha256(buffer))
      .digest()
  }

  function decodeSignature (buffer) {
    const flagByte = buffer.readUInt8(0) - 27

    return {
      compressed: !!(flagByte & 12),
      segwitType: !(flagByte & 8)
        ? null
        : !(flagByte & 4)
          ? SEGWIT_TYPES.P2SH_P2WPKH
          : SEGWIT_TYPES.P2WPKH,
      recovery: flagByte & 3,
      signature: buffer.slice(1)
    }
  }

  function magicHash (raw_message) {
    const messagePrefix = Buffer.from('\u0018Bitcoin Signed Message:\n', 'utf8')
    const message = Buffer.from(raw_message, 'utf8')
    const messageVISize = varuint.encodingLength(message.length)
    const buffer = Buffer.allocUnsafe(
      messagePrefix.length + messageVISize + message.length
    )
    messagePrefix.copy(buffer, 0)
    varuint.encode(message.length, buffer, messagePrefix.length)
    message.copy(buffer, messagePrefix.length + messageVISize)
    return hash256(buffer)
  }

  function bufferEquals(a, b) {
    function isBufferConstructor (obj) {
      return !!obj.constructor && typeof obj.constructor.isBuffer === 'function' && obj.constructor.isBuffer(obj)
    }

    function isBuffer(obj) {
      return obj != null && (isBufferConstructor(obj) || !!obj._isBuffer)
    }

    if (a === b) {
      return true;
    }

    if (typeof a.equals === 'function') {
      return a.equals(b);
    }

    if (a.length !== b.length) {
      return false;
    }

    for (var i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) {
        return false;
      }
    }

    return true;
  };

  window.bitcoinMessage = {
    "verify": function(message, address, raw_signature, segwit) {
      const signature = Buffer.from(raw_signature, 'base64')
      const parsed = decodeSignature(signature)
      const hash = magicHash(message)
      const publicKey = secp256k1.recover(
        hash,
        parsed.signature,
        parsed.recovery,
        parsed.compressed
      )
      const actual = hash160(publicKey)
      let expected
      if(segwit) {
        const result = bech32.decode(address)
        const data = bech32.fromWords(result.words.slice(1))
        expected = Buffer.from(data)
      }else{
        expected = bs58check(address).slice(1)
      }
      return bufferEquals(actual, expected)
    }
  }
})();

