import * as bitcoin from "bitcoinjs-lib";


const all = {
  "http://localhost:8000": {
    recaptchaSiteKey: "6Ld9xO0iAAAAALIPY2yJGhshPe3nek4QQIHMgwq-",
    address: "bcrt1qsj2h8ernt4amc674l60vu925flvn57ff9lyry2",
    environment: 'development',
    network: bitcoin.networks.regtest,
    path: `m/49'/1'/0'/0`,
  },
  "http://localhost:3000": {
    recaptchaSiteKey: "6Ld9xO0iAAAAALIPY2yJGhshPe3nek4QQIHMgwq-",
    address: "bcrt1qsj2h8ernt4amc674l60vu925flvn57ff9lyry2",
    environment: 'development',
    network: bitcoin.networks.regtest,
    path: `m/49'/1'/0'/0`,
  },
  "https://api-staging.constata.eu": {
    recaptchaSiteKey: "6Lf6LgQjAAAAAOymc0a-huYHHbdE0uUQ7aqfRYR4",
    address: "tb1qurghvhp8g6he5hsv0en6n59rextfw8kw0wxyun",
    environment: 'staging',
    network: bitcoin.networks.bitcoin,
    path: `m/49'/0'/0'/0`,
  },
  "https://api.constata.eu": {
    recaptchaSiteKey: "6LcX8nMjAAAAADt4A_elIs0aL75kMUF3CckpzyVt",
    address: "bc1qw3ca5pgepg6hqqle2eq8qakejl5wdafs7up0jd",
    environment: 'production',
    network: bitcoin.networks.bitcoin,
    path: `m/49'/0'/0'/0`,
  },
}

export const Settings = all[window.origin];
