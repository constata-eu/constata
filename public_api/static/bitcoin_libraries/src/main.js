//Import dependencies
const { AEAD } = require("miscreant");
const ecc = require('tiny-secp256k1');
const { BIP32Factory } = require('bip32');
// You must wrap a tiny-secp256k1 compatible implementation
const bip32 = BIP32Factory(ecc);
const bip39 = require('bip39');
const bitcoin = require('bitcoinjs-lib');
const bitcoinMessage = require("bitcoinjs-message");
const buffer = require("buffer");
const request = require('request');
const ECPairFactory = require('ecpair');
const Countrily = require("countrily");
const translate = require("translate");

