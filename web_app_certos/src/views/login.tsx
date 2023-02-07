import { useEffect } from 'react';
import PropTypes from 'prop-types';
import { Form, TextInput, useRedirect, required, minLength } from 'react-admin';
import {
  Button, Card, CardContent, CardActions, Box, Alert, LinearProgress, Typography, TextField
} from '@mui/material';
import {
  getStorage, setStorageFromString, checkStorage, clearStorage
} from '../components/auth_provider';
import { useTranslate, useLogin, useSafeSetState } from 'ra-core';
import { Create, UploadFile } from '@mui/icons-material';
import ForgetAccountModal from "../components/forget_account_modal";
import { useNavigate } from 'react-router-dom';
import { useCheckAuth } from 'ra-core';
import { Head1 } from '../theme';
import { BareLayout } from './layout';
import _ from 'lodash';

interface FormData {
  password?: string;
  conf?: string;
}

const UnknownUser = ({setHasCredentials}) => {
  const translate = useTranslate();
  const redirect = useRedirect();
  const [error, setError] = useSafeSetState(false);
  
  const handleSetCredentials = async (event) => {
    setError(false);
    const text = await event.target.files[0].text();
    if (setStorageFromString(text)){
      setHasCredentials(true);
    } else {
      setError(true);
    }
  };

  return (<Box>
    <Head1 sx={{my: 3}}> { translate("certos.login.unknown_user.title") } </Head1>
    <Box mb={5}>
      <Typography> { translate("certos.login.unknown_user.text") } </Typography>
    </Box>
    <Button sx={{mb:5}}
      id="button_create_new_signature"
      variant="contained"
      size="large"
      fullWidth
      startIcon={<Create />}
      onClick={() => redirect("/signup")}
    >
      { translate("certos.login.create_signature") }
    </Button>
    <Button
      id="button_upload_credentials"
      size="large"
      fullWidth
      component="label"
      variant="outlined"
      startIcon={<UploadFile />}
    >
      { translate("certos.login.unknown_user.select_config_file") }
      <input type="file" accept=".json" hidden onChange={handleSetCredentials} />
    </Button>
    { error && <Alert severity="error">{ translate("certos.login.unknown_user.error_invalid_file") }</Alert> }
  </Box>);
}

const KnownUser = ({setHasCredentials}) => {
  const [loading, setLoading] = useSafeSetState(false);
  const [confirmForgetOpen, setConfirmForgetOpen] = useSafeSetState(false);
  const [errorMessage, setErrorMessage] = useSafeSetState(null);
  const login = useLogin();
  const translate = useTranslate();
  const navigate = useNavigate();

  const validate = async (values: FormData) => {
    setErrorMessage(null);
    return {
      password: required()(values.password, values) || minLength(8)(values.password, values) || false
    };
  };

  const submit = async (values: FormData) => {
    setLoading(true);
    try {
      const password = values.password;
      await login({ password });
      navigate("/")
    } catch (error) {
      setErrorMessage(translate((_.isString(error) && error) || _.get(error, 'message') || 'ra.auth.sign_in_error' ));
    }
    setLoading(false);
  };

  const key = getStorage().public_key;

  return ( <Form onSubmit={submit} validate={validate}>
    <Head1 sx={{ my: 3 }}> { translate("certos.login.known_user.title") } </Head1>
    <Box mb={5}>
      <Typography> { translate("certos.login.known_user.text") } </Typography>
    </Box>
    <Card sx={{mb: 5}}>
      <CardContent>
        <TextField
          fullWidth
          disabled
          label={ translate('certos.login.known_user.using_signature') }
          variant="filled"
          value={key}
        />
        <TextInput autoFocus fullWidth source="password" type="password" label='ra.auth.password' disabled={loading} helperText={false} />
      </CardContent>
      <CardActions>
        <Button id="button_access_with_password" fullWidth size="large" type="submit" variant="contained" disabled={loading}>
          { translate('certos.login.known_user.access')}
        </Button>
      </CardActions>
      { errorMessage && <Alert severity="error">{ errorMessage }</Alert> }
      { loading && <LinearProgress sx={{ height: "0.5rem" }} color="highlight" /> }
    </Card>
    <Button id="button_use_another_signature" fullWidth variant="outlined" onClick={() => setConfirmForgetOpen(true) } >
      { translate('certos.login.known_user.use_another_signature')}
    </Button>
    <ForgetAccountModal 
      open={confirmForgetOpen}
      handleConfirm={ () => {setHasCredentials(false); clearStorage()} }
      handleCancel={ () => setConfirmForgetOpen(false) }
    />
  </Form>);
}

const Login = () => {
  const [hasCredentials, setHasCredentials] = useSafeSetState(checkStorage());
  const checkAuth = useCheckAuth();
  const navigate = useNavigate();

  useEffect(() => {
    const check = async () => {
      try {
        await checkAuth(undefined, false);
        navigate("/")
      } catch (e){}
    }
    check();
  }, [checkAuth, navigate]);

  return (<BareLayout>
    {hasCredentials && <KnownUser setHasCredentials={setHasCredentials}/>}
    {!hasCredentials && <UnknownUser setHasCredentials={setHasCredentials}/> }
  </BareLayout>);
};


Login.propTypes = {
  redirectTo: PropTypes.string,
};

export default Login;
