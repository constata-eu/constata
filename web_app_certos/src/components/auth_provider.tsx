import { Buffer } from "buffer";
import { getKeyPair, getSignedPayload} from "./cypher";
import type { Credentials } from './types';
import _ from 'lodash';

declare global {
  interface Window { pass?: string; }
}

export function setSignupMode(token, key) {
  localStorage.setItem("signupToken", token);
  localStorage.setItem("signupEncryptedKey", key);
}

export function getSignupData() {
  let token = localStorage.getItem("signupToken");
  let key = localStorage.getItem("signupEncryptedKey");
  if (token && key) {
    return {"Signup-Token": token, "Signup-Encrypted-Key": key};
  } else {
    return null;
  }
}
export function unsetSignupMode() {
  localStorage.removeItem("signupToken");
  localStorage.removeItem("signupEncryptedKey");
}

export function getStorage() {
  return {
    "public_key": localStorage.getItem("public_key"),
    "encrypted_key": localStorage.getItem("encrypted_key"),
    "environment": localStorage.getItem("environment"),
  };
}

export function setStorage(object) {
  if (!_.every(['public_key', 'encrypted_key', 'environment'], (k) => object[k])){
    return false;
  }
  localStorage.setItem("public_key", object.public_key);
  localStorage.setItem("encrypted_key", object.encrypted_key);
  localStorage.setItem("environment", object.environment);
  return true;
}

export function setStorageFromString(string) {
  try{
    return setStorage(JSON.parse(string));
  } catch(e) {
    return false;
  }
}

export const checkStorage = () =>
  _.every(['public_key', 'encrypted_key', 'environment'], (k) => localStorage.getItem(k))

export function clearStorage() {
  window.pass = undefined;
  for(const k of ['public_key', 'encrypted_key', 'environment']) {
    localStorage.removeItem(k);
  }
}

export async function getRawAuthorization(url: string, method: string, body: string | null) {
  if (!window.pass) return;
  if (!checkStorage()) return;
  const conf: Credentials = getStorage();
  const [keyPair, address] = await getKeyPair(conf.encrypted_key, window.pass, conf.environment);
  const {pathname, search } = new URL(url, document.location.origin);

  async function sha256sum(plaintext){
    return Buffer.from(await crypto.subtle.digest('SHA-256', (new TextEncoder()).encode(plaintext))).toString("hex")
  }

  const payload = Buffer.from(JSON.stringify({
    "path": pathname,
    "method": method,
    "nonce": Date.now(),
    "body_hash": body ? (await sha256sum(body)) : null,
    "query_hash": search.length > 1 ? (await sha256sum(search.substr(1))) : null
  }));

  const signed_payload = getSignedPayload(keyPair, address, Buffer.from(payload));

  return JSON.stringify(signed_payload);
}

export function setAccessToken(t) {
  localStorage.setItem("accessToken", t)
}

export function getAccessToken() {
  return localStorage.getItem("accessToken");
}

export function clearAccessToken() {
  localStorage.removeItem("accessToken");
}

const authProvider: any = {
  login: async ({ password }) => {
    const conf = getStorage();
    await getKeyPair(conf.encrypted_key, password, conf.environment);
    window.pass = password;
    return true;
  },
  checkError: (error, graphQLErrors) => {
    const graphQLFail = graphQLErrors?.[0].message === "401";
    const status = error.status || error?.networkError?.statusCode;
    const httpFail = status === 401 || status === 403

    if (!getAccessToken() && !getSignupData() && (graphQLFail || httpFail) ) {
      return Promise.reject(graphQLFail ? graphQLErrors : error);
    }

    return Promise.resolve();
  },
  checkAuth: () => {
    return window.pass && checkStorage()
    ? Promise.resolve()
    : Promise.reject( { redirectTo: '/login' } )
  },
  injectAuthorization: async (url: string, req) => {
    let token = getAccessToken();
    if(token) {
      req.headers["Access-Token"] = token;
    } else {
      req.headers["Authentication"] = await getRawAuthorization(url, req.method, req.body);
    }
    const signup = getSignupData();
    if(signup) {
      req.headers = { ...signup, ...req.headers }
    }
  },
  logout: () => {
    window.pass = undefined;
    return Promise.resolve();
  },
  getPermissions: () => {
    return Promise.resolve("full");
  }
};

export default authProvider
