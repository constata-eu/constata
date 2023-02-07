import { AEAD } from "miscreant";
import ECPairFactory from 'ecpair';
import * as ecc from 'tiny-secp256k1';
import * as bitcoin from "bitcoinjs-lib";
import * as bitcoinMessage from "bitcoinjs-message";

const ECPair = ECPairFactory(ecc);

export const envs = {
  development: {
    network: bitcoin.networks.regtest,
    url: "http://localhost:8000",
  },
  production: {
    network: bitcoin.networks.bitcoin,
    url: "https://api.constata.eu",
  },
  staging: {
    network: bitcoin.networks.bitcoin,
    url: "https://api-staging.constata.eu",
  },
}
  
export async function getKeyPair(encrypted_key, password: string, environment) {
  const network = envs[environment].network;
  const pass = (new TextEncoder()).encode(password);
  let keyData = new Uint8Array(32);
  keyData.set(pass, 0);
  const key = await AEAD.importKey(keyData, "AES-CMAC-SIV");
  const serialized = Buffer.from(encrypted_key, "hex");

  try {
    const pkwif = await key.open(serialized.slice(24), serialized.slice(0, 16));
    const keyPair = ECPair.fromWIF((new TextDecoder()).decode(pkwif), network); 
    const { address } = bitcoin.payments.p2pkh({
      pubkey: keyPair.publicKey,
      network: network,
    });
    return [keyPair, address];
    
  } catch {
    throw new Error("certos.login.invalid_password")
  }
}
  
export function getSignedPayload(keyPair, address, message) {
  const signature = bitcoinMessage.sign(message, keyPair.privateKey, keyPair.compressed)
  return {
    payload: message.toString("base64"),
    signer: address,
    signature: signature.toString("base64"),
  };
}