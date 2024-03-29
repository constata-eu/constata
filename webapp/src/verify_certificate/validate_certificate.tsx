import base64ToBytes from './base64_to_bytes';
import * as bitcoinMessage from "bitcoinjs-message";


async function validate_certificate(file_input, signer){
    const file = file_input.files[0];
    const lines = (await file.text()).split("\n"); 
    /* The document must end with a new line, but some services remove it.
     * If we detect that, we add the newline back to the end of the buffer
     * otherwise the validaiton will fail */
    if(lines.slice(-2,-1) !== ''){
      lines.push('');
    }
  
    function sanitize_line(l) {
      return (l.slice(-1) === '\r' ? l.slice(0,-1) : l) + "\n";
    }
    
    const signature = Buffer.from(base64ToBytes(lines.slice(-4, -3)[0]));
    const raw_message = lines.slice(0, -10).map(sanitize_line).join("");
    const message = Buffer.from(raw_message, "utf8");

    return bitcoinMessage.verify(message, signer, signature, null, true);
  }


export default validate_certificate;
