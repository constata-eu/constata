import { Card, CardContent, Typography, Container, Button, LinearProgress, Alert,
         AlertTitle } from '@mui/material';
import { useTranslate, Form } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import CardTitle from '../components/card_title';
import { UploadFile } from '@mui/icons-material';
import { NoLoggedInLayout } from './layout';
import validate_certificate from '../verify_certificate/validate_certificate';
import { Settings } from '../Settings';
import IframeCertificate from '../components/iframe';


const Safe = () => {
  const [state, setState] = useSafeSetState<string>("toUpload");
  const [urlObject, setUrlObject] = useSafeSetState(null);

  const handleValidateCertificate = async (event) => {
    setState("loading");
    try {
      const signer = Settings.address;
      const ok = await validate_certificate(event.target, signer);
      if(ok) {
        setState("valid");
        setUrlObject(URL.createObjectURL(event.target.files[0]));
        return window.dispatchEvent(new Event("load"));
      }
    }
    catch {}
    setState("invalid");
  };

  const props = { handleValidateCertificate };

  if (state === "valid") {
    return <IframeCertificate url={urlObject} id="iframe-valid-certificate" />;
  }

  return (<NoLoggedInLayout>
    <Container maxWidth="md">
      { state === "toUpload" && <ToUpload {...props} /> }
      { state === "loading" && <LoadingLinear /> }
      { state === "invalid" && <><ToUpload {...props} /><InvalidCertificate /></> }
    </Container>
  </NoLoggedInLayout>
)}

const LoadingLinear = () => {
  const translate = useTranslate();
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.safe.loading")} />
    <CardContent >
      <LinearProgress />
    </CardContent>
  </Card>
};

const ToUpload = ({handleValidateCertificate}) =>  {
  const translate = useTranslate();
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.safe.initial.title")} />
    <Form>
      <CardContent >
      
        <Typography sx={{ mb: 1 }}> {translate("certos.safe.initial.text_1")} </Typography>
        <Typography sx={{ mb: 1 }}> {translate("certos.safe.initial.text_2")} </Typography>

        <Button
          sx={{my: 2}}
          size="large"
          fullWidth
          component="label"
          variant="contained"
          startIcon={<UploadFile />}
        >
          <>
            {translate("certos.safe.initial.button")}
            <input id="certificate" type="file" accept=".html" hidden onChange={handleValidateCertificate} />
          </>
        </Button>
      </CardContent>
    </Form>
  </Card>
};

const InvalidCertificate = () => {
  const translate = useTranslate();
  return <Alert variant="outlined" severity="error" sx={{ mb: 5 }} icon={false}>
    <AlertTitle id="invalid-certificate">{ translate("certos.safe.invalid.title") }</AlertTitle>
    <Typography> {translate("certos.safe.invalid.text_1")}
      &nbsp;
      <a href="mailto:hello@constata.eu?subject=invalid certificate">hello@constata.eu</a>
      &nbsp;
      {translate("certos.safe.invalid.text_2")}
    </Typography>
  </Alert>
};



export default Safe;
