import { useEffect } from 'react';
import { Card, CardContent, Box, Typography, Alert, Button, LinearProgress } from '@mui/material';
import { useTranslate, useDataProvider } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { useParams } from 'react-router-dom';
import { setAccessToken, clearAccessToken } from '../components/auth_provider';
import { BareLayout } from './layout';

const VerifyEmail = () => {
  const dataProvider = useDataProvider();
  const { access_token } = useParams();
  const [state, setState] = useSafeSetState<string>("loading");
  const translate = useTranslate();

  useEffect(() => {
    const init = async () => {
      setAccessToken(access_token);
      try {
        await dataProvider.create('EmailAddressVerification', { data: {}});
        setState("success")
      } catch (e) { 
        setState(e.status === 401 ? "warning" : "error")
      }
      clearAccessToken();
    }
    init();
  }, [access_token, dataProvider, setState]);

  return (<BareLayout>
    <Box mt={5}>
      { state === "success" && <Alert
          severity="success"
          variant="outlined"
          action={
            <Button color="success" href="/" size="small">
              { translate("certos.verify_email.go_to_dashboard") }
            </Button>
          }
        >
          { translate("certos.verify_email.success") }
        </Alert>
      }

      { state === "error" && <Alert severity="error" variant="outlined" >
          { translate("certos.verify_email.other_error") }
        </Alert>
      }

      { state === "warning" && <Alert variant="outlined" severity="warning" >
          { translate("certos.verify_email.expired") }
        </Alert>
      }

      { state === "loading" && <Card>
          <CardContent>
            <Typography>{ translate("certos.verify_email.loading") }</Typography>
          </CardContent>
          <LinearProgress />
        </Card>
      }
    </Box>
  </BareLayout>);
}

export default VerifyEmail;
