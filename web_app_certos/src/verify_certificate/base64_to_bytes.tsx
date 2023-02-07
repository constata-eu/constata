function base64ToBytes(base64) {
    const bytesAsString = window.atob(base64);
    const size = bytesAsString.length;
    const bytes = new Uint8Array(size);
    for (let i = 0; i < size; i++) {
      bytes[i] = bytesAsString.charCodeAt(i);
    }
    return bytes;
  }

  export default base64ToBytes;