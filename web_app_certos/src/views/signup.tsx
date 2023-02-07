import { useEffect, useCallback } from 'react';
import type { ReactElement } from 'react';
import PropTypes from 'prop-types';
import {
  Chip, Card, CardContent, CardActions, Box, Typography,
  Avatar,
  Button,
  Alert
} from '@mui/material';
import HourglassBottomIcon from '@mui/icons-material/HourglassBottom';
import HistoryEduIcon from '@mui/icons-material/HistoryEdu';
import LoginIcon from '@mui/icons-material/Login';
import FingerprintIcon from '@mui/icons-material/Fingerprint';
import CookieIcon from '@mui/icons-material/Cookie';
import NoAccountsIcon from '@mui/icons-material/NoAccounts';
import GavelIcon from '@mui/icons-material/Gavel';
import BadgeIcon from '@mui/icons-material/Badge';
import KeyboardDoubleArrowRightIcon from '@mui/icons-material/KeyboardDoubleArrowRight';
import LockIcon from '@mui/icons-material/Lock';
import SupportAgentIcon from '@mui/icons-material/SupportAgent';
import ReportProblemIcon from '@mui/icons-material/ReportProblem';
import Home from '@mui/icons-material/Home';
import PasswordIcon from '@mui/icons-material/Password';
import Check from '@mui/icons-material/Check';
import Email from '@mui/icons-material/Email';
import { Form, TextInput, useTheme, useTranslate, email, required, minLength, useNotify,
         BooleanInput, useDataProvider} from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { AEAD } from 'miscreant';
import * as ecc from 'tiny-secp256k1';
import { BIP32Factory } from 'bip32';
import * as bip39 from 'bip39';
import { Buffer } from "buffer";
import type { Credentials } from '../components/types';
import type { BIP32Interface } from 'bip32';
import {
  GoogleReCaptchaProvider,
  GoogleReCaptcha
} from 'react-google-recaptcha-v3';
import { useNavigate } from 'react-router-dom';
import { clearStorage, setStorage, checkStorage, unsetSignupMode, setSignupMode } from '../components/auth_provider';
import { BareLayout } from './layout';
import { Settings } from '../Settings';
import { Head1 } from '../theme';
import CardTitle from '../components/card_title';
import _ from 'lodash';

declare global {
  interface Window { pass?: string; }
}

interface FormData {
  password?: string;
  confirmPassword?: string;
  email?: string;
  keepPrivate: boolean;
  tycAccepted: boolean;
  privacyPolicyAccepted: boolean;
  step: number;
  keyPair?: BIP32Interface;
  mnemonic?: string;
  token?: string;
  emailTaken: boolean;
  loading: boolean;
}

const IntroStep = () => {
  const translate = useTranslate();
  return (<Box>
    <BulletPoint label="resources.SignUp.IntroStep.for_login" icon={<LoginIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.for_signing" icon={<HistoryEduIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.no_account" icon={<NoAccountsIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.verify_later" icon={<BadgeIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.checklist" icon={<HourglassBottomIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.unique" icon={<FingerprintIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.safekeep" icon={<LockIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.no_recovery" icon={<ReportProblemIcon/>} />
    <BulletPoint label="resources.SignUp.IntroStep.help" icon={<SupportAgentIcon/>} />
    <Button autoFocus sx={{my: 3}} fullWidth size="large" variant="contained" id="signup-intro-submit" type="submit">
      { translate("resources.SignUp.IntroStep.start_btn")}
    </Button>
    <Button fullWidth variant="outlined" href="https://constata.eu">
      { translate("resources.SignUp.IntroStep.ill_do_it_later")}
    </Button>
  </Box>);
}

const IntroStepDone = () => {
  return (<DoneStep>
    <BulletPoint noGutter label="resources.SignUp.IntroStep.done" icon={<Home/>} />
  </DoneStep>);
}

const TycStep = () => {
  const translate = useTranslate();
  return (<Card sx={{mb: 2, textAlign: "justify"}}>
    <CardTitle text="resources.SignUp.TycStep.title" />
    <CardContent>
      <BulletPoint label="resources.SignUp.TycStep.issue_documents"/>
      <BulletPoint label="resources.SignUp.TycStep.bitcoin_backed"/>
      <BulletPoint label="resources.SignUp.TycStep.tokens_needed"/>
      <BulletPoint label="resources.SignUp.TycStep.giveaway"/>
      <BulletPoint label="resources.SignUp.TycStep.valid_forever_free"/>
      <BulletPoint label="resources.SignUp.TycStep.we_keep_a_copy"/>
      <BulletPoint label="resources.SignUp.TycStep.accept_our_policies"/>
      <Button sx={{ my: 2 }} href="https://api.constata.eu/terms_acceptance/show/" target="_blank">
        { translate("resources.SignUp.TycStep.link") }
      </Button>
      <BooleanInput autoFocus helperText={false} label="resources.SignUp.TycStep.acceptTyc" source="tycAccepted" />
    </CardContent>
    <CardActions>
      <ButtonNext id="signup-tyc-submit"/>
    </CardActions>
  </Card>);
}

const TycStepDone = ({setStep}) => {
  const translate = useTranslate();
  return (<DoneStep>
    <BulletPoint noGutter label="resources.SignUp.TycStep.done" icon={<GavelIcon/>}>
      <Button size="small" sx={{ ml: 1 }} variant="outlined" onClick={() => setStep(1)}>{ translate("resources.SignUp.TycStep.review")}</Button> 
    </BulletPoint>
  </DoneStep>);
}

const PrivacyStep = () => {
  const translate = useTranslate();
  return (<Card sx={{mb: 2, textAlign: "justify"}}>
    <CardTitle text="resources.SignUp.PrivacyStep.title" />
    <CardContent>
      <BulletPoint label="resources.SignUp.PrivacyStep.data_privacy" />
      <BulletPoint label="resources.SignUp.PrivacyStep.one_cookie" />
      <BulletPoint label="resources.SignUp.PrivacyStep.owner" />
      <BulletPoint label="resources.SignUp.PrivacyStep.mandatory" />
      <BulletPoint label="resources.SignUp.PrivacyStep.accept_our_policy" />
      <Button sx={{ my: 2 }} href="https://api.constata.eu/terms_acceptance/show/#privacy_policies" target="_blank">
        { translate("resources.SignUp.PrivacyStep.link") }
      </Button>
      <BooleanInput autoFocus helperText={false} label="resources.SignUp.PrivacyStep.acceptPrivacyPolicy" source="privacyPolicyAccepted" />
    </CardContent>
    <CardActions>
      <ButtonNext id="signup-privacy-submit"/>
    </CardActions>
  </Card>);
}

const PrivacyStepDone = ({setStep}) => {
  const translate = useTranslate();
  return (<DoneStep>
    <BulletPoint noGutter label="resources.SignUp.PrivacyStep.done" icon={<CookieIcon/>}>
      <Button size="small" sx={{ ml: 1 }} variant="outlined" onClick={() => setStep(2)}>{ translate("resources.SignUp.TycStep.review")}</Button> 
    </BulletPoint>
  </DoneStep>);
}

const PasswordStep = () => {
  return (<Card sx={{mb: 2}} >
    <CardTitle text="resources.SignUp.PasswordStep.title" />
    <CardContent >
      <BulletPoint label="resources.SignUp.PasswordStep.will_encrypt"/>
      <BulletPoint label="resources.SignUp.PasswordStep.dont_remember"/>
      <BulletPoint label="resources.SignUp.PasswordStep.make_it_easy"/>
      <BulletPoint label="resources.SignUp.PasswordStep.phrase"/>
      <BulletPoint label="resources.SignUp.PasswordStep.no_change"/>
      <BulletPoint label="resources.SignUp.PasswordStep.no_recovery"/>
      <BulletPoint label="resources.SignUp.PasswordStep.we_remember"/>
      <TextInput
        sx={{ mt: 2 }}
        autoFocus
        source="password"
        fullWidth
        type="password"
        label="resources.SignUp.PasswordStep.label"
        helperText={false}
      />
    </CardContent>
    <CardActions>
      <ButtonNext id="signup-password-submit"/>
    </CardActions>
  </Card>);
}

const PasswordStepDone = ({setStep}) => {
  const translate = useTranslate();
  return (<DoneStep>
    <BulletPoint noGutter label="resources.SignUp.PasswordStep.done" icon={<PasswordIcon/>}>
      <Button variant="outlined" sx={{ ml: 1 }} size="small" onClick={() => setStep(3)}>{ translate("resources.SignUp.PasswordStep.change") }</Button>
    </BulletPoint>
  </DoneStep>);
}

const WordsStep = ({ mnemonic }) => {
  const [theme] = useTheme();
  return (<Card sx={{mb: 2}}>
    <CardTitle text="resources.SignUp.WordsStep.title" />
    <CardContent>
      <BulletPoint label="resources.SignUp.WordsStep.store_physically"/>
      <BulletPoint label="resources.SignUp.WordsStep.optional"/>
      <BulletPoint label="resources.SignUp.WordsStep.if_you_lose"/>
      <BulletPoint label="resources.SignUp.WordsStep.write_it_down"/>
      <BulletPoint label="resources.SignUp.WordsStep.must_contact_us"/>
      <Box my={3} >
        {mnemonic.split(" ").map((e: string, i: number) => {
          return (<Chip
            sx={{ mt: 1, ml: 1 }}
            variant="filled"
            color="highlight"
            label={e}
            key={e}
            avatar={<Avatar sx={{ bgcolor: theme?.palette?.inverted?.main, color: theme?.palette?.inverted?.contrastText }} >{i + 1}</Avatar>}
          />)
        })}
      </Box>
    </CardContent>
    <CardActions>
      <ButtonNext autoFocus id="signup-words-submit"/>
    </CardActions>
  </Card>);
}
const WordsStepDone = ({setStep}) => {
  const translate = useTranslate();
  return (<DoneStep>
    <BulletPoint noGutter label="resources.SignUp.WordsStep.done" icon={<LockIcon/>}>
      <Button variant="outlined" sx={{ ml: 1 }} size="small" onClick={() => setStep(4)}>{ translate("resources.SignUp.WordsStep.view_again") }</Button>
    </BulletPoint>
  </DoneStep>);
}

const EmailAddressStep = ({emailTaken}) => {
  const translate = useTranslate();

  return (<Card sx={{mb: 2, textAlign: "justify"}}>
    <CardTitle text="resources.SignUp.EmailAddressStep.title" />
    <CardContent>
      <BulletPoint label="resources.SignUp.EmailAddressStep.send_copy" />
      <BulletPoint label="resources.SignUp.EmailAddressStep.support" />
      <BulletPoint label="resources.SignUp.EmailAddressStep.news" />
      <BulletPoint label="resources.SignUp.EmailAddressStep.poll" />
      <BulletPoint label="resources.SignUp.EmailAddressStep.endorse" />
      { emailTaken && <Alert variant="filled" severity="error">{ translate("resources.SignUp.email_taken") }</Alert> }
      <TextInput
        autoFocus
        sx={{ mt: 3 }}
        fullWidth
        size="small"
        source="email"
        label="resources.SignUp.EmailAddressStep.label"
        helperText="resources.SignUp.EmailAddressStep.helper"
        variant="filled"
      />
      <BooleanInput 
        label="certos.kyc_request.fields.keepPrivate"
        source="keepPrivate"
        helperText={false}
      />
    </CardContent>
    <CardActions>
      <ButtonNext id="signup-email-submit"/>
    </CardActions>
  </Card>)
}

const EmailAddressStepDone = ({setStep, email}) => {
  const translate = useTranslate();
  return (<DoneStep>
    <BulletPoint noGutter icon={<Email/>}
      label={
        <>
          {translate("resources.SignUp.EmailAddressStep.done")}: {email ? email : translate("resources.SignUp.EmailAddressStep.no_email")}
        </>
      }
    >
      <Button size="small" sx={{ ml: 1 }} variant="outlined" onClick={() => setStep(5)}>{ translate("resources.SignUp.EmailAddressStep.change")}</Button> 
    </BulletPoint>
  </DoneStep>);
}

const ConfirmPassStep = () => {
  return (<Card sx={{mb: 2}}>
    <CardContent>
      <TextInput autoFocus source="confirmPassword" fullWidth type="password" helperText={false} label="resources.SignUp.ConfirmPassStep.label" />
    </CardContent>
    <CardActions>
      <ButtonNext id="signup-confirm-password-submit"/>
    </CardActions>
  </Card>);
}

const ConfirmPassStepDone = () => {
  return (<DoneStep><BulletPoint noGutter label="resources.SignUp.ConfirmPassStep.done" icon={<Check/>} /></DoneStep>);
}

const DownloadStep = ({onVerify, token }) => {
  const translate = useTranslate();

  return (<Box>
    <Card sx={{mb: 2, textAlign: "justify"}}>
      <CardTitle text="resources.SignUp.DownloadStep.title" />
      <CardContent>
        <Typography variant="body2">{ translate("resources.SignUp.DownloadStep.usage") }</Typography>
      </CardContent>
      <CardActions>
        <Button fullWidth variant="contained" type="submit" disabled={!token}>
          { translate("resources.SignUp.DownloadStep.download") }
        </Button>
      </CardActions>
    </Card>
    <GoogleReCaptchaProvider reCaptchaKey={ Settings.recaptchaSiteKey }>
      <GoogleReCaptcha onVerify={onVerify} />
    </GoogleReCaptchaProvider>
  </Box>);
}

let DownloadStepDone = () => <></>;

const Signup = () => {
  const navigate = useNavigate();
  const translate = useTranslate();
  const notify = useNotify();
  const dataProvider = useDataProvider();
  const [wizardState, setWizardState] = useSafeSetState<FormData>({
    keepPrivate: false,
    tycAccepted: false,
    privacyPolicyAccepted: false,
    step: 0,
    emailTaken: false,
    loading: true,
  });
  const onVerify = useCallback((token) => { setWizardState((s) => ({ ...s, token })) }, [setWizardState]);

  useEffect(() => {
    if (checkStorage()) {
      return navigate("/");
    }
    setWizardState((s) => ({
      ...s,
      mnemonic: bip39.generateMnemonic(),
      loading: false,
    }));
  }, [setWizardState, navigate]);

  const setStep = (i) => setWizardState((s) => ({ ...s, step: i}))

  const validate = (values: FormData) => {
    const errors: any = {};
    
    switch (wizardState.step) {
      case 1:
        errors.tycAccepted = !values.tycAccepted && translate("resources.SignUp.TycStep.mustAcceptTyc");
        break;
      case 2:
        errors.privacyPolicyAccepted = !values.privacyPolicyAccepted && translate("resources.SignUp.PrivacyStep.mustAcceptPrivacyPolicy");
        break;
      case 3:
        errors.password = required()(values.password, values) || minLength(8)(values.password, values) || false;
        break;
      case 5:
        errors.email = email()(values.email) || false;
        break;
      case 6:
        if (values.password !== values.confirmPassword) {
          errors.confirmPassword = "resources.SignUp.mismatch_passwords";
        }
        break;
    }

    return errors;
  };

  const submit = async (values: FormData) => {
    let updates;

    if(wizardState.step === 7) {
      updates = await handleFinishSignup();
      if(!updates){ return; }
    } else {
      updates = {
        emailTaken: false,
        step: Math.min(wizardState.step + 1, 7)
      };
    }

    if(wizardState.step === 0) {
      updates.keyPair = handleCreateKeyPair();
    }

    setWizardState((s) => ({...s, ...values, ...updates}));
  };
  
  const handleCreateKeyPair = () => {
    let {mnemonic, password} = wizardState;
    const {network, path} = Settings;
    const seed = bip39.mnemonicToSeedSync(mnemonic, password);
    const bip32 = BIP32Factory(ecc);
    const rootKey = bip32.fromSeed(seed, network);
    const account = rootKey.derivePath(path);
    return account.derive(0).derive(0);
  }

  const handleFinishSignup = async () => {
    let {email, keepPrivate, keyPair, password, token} = wizardState;
    let encryptedKey = await generateEncryptedKey(password, keyPair);
    let creds: Credentials = {
      public_key: keyPair?.publicKey.toString('hex'),
      encrypted_key: encryptedKey,
      environment: Settings.environment,
    };
    let updates;

    try {
      setSignupMode(token, encryptedKey)
      window.pass = password;
      setStorage(creds);
      await dataProvider.create('Signup', { data: { input: {email, keepPrivate}}});
      downloadCredentials(creds, translate("resources.SignUp.DownloadStep.filename"));
      navigate("/")
      return null;
    } catch (e) {
      clearStorage();
      window.pass = undefined;
      if(e.body?.graphQLErrors[0]?.extensions?.error?.field === "uniqueness") {
        updates = {emailTaken: true, step: 5};
      } else {
        updates = {emailTaken: false, step: 0};
        notify(e.message || "certos.errors.default", { type: "error", autoHideDuration: 5000 })
      }
    } finally {
      unsetSignupMode();
    }
    return updates;
  }

  const show = (i: number, current, done) => wizardState.step && wizardState.step > i ? done : ( wizardState.step === i && current );
  
  if (wizardState.loading) return (<></>);

  return (<BareLayout>
    <Form onSubmit={submit} validate={validate}>
      <Head1 sx={{my: 3}}> { translate("resources.SignUp.title") } </Head1>
      <Box mb={5}>
        <Typography> { translate("resources.SignUp.description") } </Typography>
      </Box>
      { show(0, <IntroStep/>, <IntroStepDone/>) }
      { show(1, <TycStep/>, <TycStepDone setStep={setStep} />) }
      { show(2, <PrivacyStep/>, <PrivacyStepDone setStep={setStep} />) }
      { show(3, <PasswordStep/>, <PasswordStepDone setStep={setStep} />) }
      { show(4, <WordsStep mnemonic={wizardState.mnemonic}/>, <WordsStepDone setStep={setStep}/>) }
      { show(5, <EmailAddressStep emailTaken={wizardState.emailTaken} />, <EmailAddressStepDone setStep={setStep} email={wizardState.email}/>) }
      { show(6, <ConfirmPassStep/>, <ConfirmPassStepDone/>) }
      { show(7, <DownloadStep onVerify={onVerify} token={wizardState.token}/>, <DownloadStepDone/>) }
    </Form>
  </BareLayout>);
};

Signup.propTypes = {
  redirectTo: PropTypes.string,
};

export const DoneStep = ({children}: { children: JSX.Element } ) => 
  <Card sx={{mb: 2}}>
    <CardContent sx={{ py: "0.5em !important" }}>
      { children }
    </CardContent>
  </Card>;

interface BulletPointInterface {
  label: any,
  icon?: any,
  children?: ReactElement,
  noGutter?: boolean,
}

const BulletPoint = ({label, icon, children, noGutter} : BulletPointInterface) => {
  const translate = useTranslate();
  const opts = { sx: { my: (noGutter ? 0 : 0.5) } };
  const text = _.isString(label) ? translate(label) : label;

  return (<Box { ...opts } display="flex" alignItems="center">
    { icon || <KeyboardDoubleArrowRightIcon/> }
    <Typography variant="body2" sx={{ml:1}}>{text}</Typography>
    <Box component="div" sx={{ flex: 1 }} />
    { children }
  </Box>);
};

const ButtonNext = ({label, autoFocus, id }: {label?: string, autoFocus?: boolean, id?: string}) => {
  let translate = useTranslate();
  return <Button id={id} autoFocus={autoFocus} fullWidth variant="outlined" type="submit">{ label || translate("resources.SignUp.ready")}</Button>
}

const downloadCredentials = (config: Credentials, filename: string) => {
  const a = document.createElement('a');
  a.href = "data:application/json;charset=utf-8," + JSON.stringify(config);
  a.download = filename;
  a.click();
}

let generateEncryptedKey = async (password, keyPair) => {
  const pass = (new TextEncoder()).encode(password);
  let keyData = new Uint8Array(32);
  keyData.set(pass, 0);

  const key = await AEAD.importKey(keyData, "AES-CMAC-SIV");
  const serialized = Buffer.from((new TextEncoder()).encode(keyPair.toWIF()));
  
  let nonce_data = new Uint8Array(16);
  window.crypto.getRandomValues(nonce_data);
  const encripted_key_encoded = await key.seal(serialized, nonce_data);

  let encrypted_key_buffer = new Uint8Array([...nonce_data, ...new Uint8Array(8),...encripted_key_encoded]);
  let encrypted_key = Buffer.from(encrypted_key_buffer).toString("hex");
  return encrypted_key
}

export default Signup;
