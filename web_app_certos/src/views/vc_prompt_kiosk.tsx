import { useEffect } from 'react';
import { Skeleton, Card, CardContent, Typography, Container, Box, LinearProgress, Button } from '@mui/material';
import { useNotify, useTranslate, useDataProvider, Form, NumberInput, required, minValue, maxValue } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { useParams, useNavigate } from 'react-router-dom';
import { setAccessToken, clearAccessToken } from '../components/auth_provider';
import CardTitle from '../components/card_title';
import ConstataSkeleton from '../components/skeleton';
import CssBaseline from '@mui/material/CssBaseline';
import QRCode from "react-qr-code";
import { useSearchParams } from "react-router-dom";
import DoneIcon from '@mui/icons-material/Done';

const VcPromptKiosk = () => {
  const dataProvider = useDataProvider();
  const { access_token } = useParams();
  const [vcRequest, setVcRequest] = useSafeSetState();
  const [done, setDone] = useSafeSetState();
  const translate = useTranslate();
  const notify = useNotify();

  const create = async () => {
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.create('KioskVcRequest', { data: { input: null } });
      clearAccessToken();
      setDone(false);
      setVcRequest(data);
    } catch(e) {
      notify(e.toString(), { type: 'error' });
      clearAccessToken();
    }
  }

  useEffect(() => {
    create();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    let interval;
    async function load(){
      if(!vcRequest) { return; }

      setAccessToken(access_token);
      let value = await dataProvider.getOne('KioskVcRequest', { id: vcRequest.id });
      clearAccessToken();

      if( value.data.state == "APPROVED" ) {
        setDone(true);
        setVcRequest(null);
        setTimeout(create, 2000);
      }
    }
    load();
    interval = setInterval(load, 1000);
    return function cleanup() { clearInterval(interval); };
  }, [vcRequest, setVcRequest]);

  return (<Box sx={{ minHeight: "100vh", display: "flex", flexDirection: "column", }}>
    <CssBaseline/>
    <Container maxWidth="md" id="vc-prompt">
      <Box textAlign="center" mt={8} mb={2}>
        <div style={{ height: "auto", margin: "0 auto", maxWidth: 500, width: "100%" }}>
          { !vcRequest && !done && <Skeleton variant="rectangular" height="auto" sx={{ maxWidth: "100%", width: "100%"}} /> }
          { vcRequest && !done &&
            <Box>
              <Typography textAlign="center" mb={3}>
                { vcRequest.description }
              </Typography>
              <QRCode
                size={256}
                style={{ height: "auto", maxWidth: "100%", width: "100%" }}
                value={vcRequest.vidchainUrl}
                viewBox={`0 0 256 256`}
              />
              <Typography textAlign="center" mt={3}>
                Debe tener instalado VidWallet.
              </Typography>
              <Typography textAlign="center">
                Utilice su scanner de QR habitual, no el de VidWallet.
              </Typography>
            </Box>
          }
          { done &&
            <Box>
              <DoneIcon sx={{ height: 200, width: 200, color: "#018264" }} />
              <Typography sx={{ color: "#018264", textAlign: "center" }}>
                Credencial aceptada.
              </Typography>
            </Box>
          }
        </div>
      </Box>
    </Container>
  </Box>)
}

const VidChainRedirect = () => {
  const dataProvider = useDataProvider();
  const [searchParams,] = useSearchParams();
  const translate = useTranslate();
  const notify = useNotify();
  const [done, setDone] = useSafeSetState(false);
  const access_token = searchParams.get("state").replaceAll(" ", "+");
  const code = searchParams.get("code");

  const update = async () => {
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.update('KioskVcRequest', { data: { code } });
      setDone(true);
    } catch(e) {
      notify(e.toString(), { type: 'error' });
    }
    clearAccessToken();
  }

  useEffect(() => {
    update();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (<Box sx={{ minHeight: "100vh", display: "flex", flexDirection: "column", }}>
    <CssBaseline/>
    <Container maxWidth="md" id="submit" sx={{ display: "flex" }}>
      <Box sx={{ margin: "2em auto", alignSelf: "center" }} >
        { !done && <Skeleton variant="rectangular" sx={{ height: 200, width: 200 }} /> }
        { done && <Box>
            <DoneIcon sx={{ height: 200, width: 200, color: "#018264" }} />
            <Typography sx={{ color: "#018264", textAlign: "center" }}>
              Credencial aceptada.
              <br/>
              Puede cerrar esta ventana.
            </Typography>
          </Box>
        }
      </Box>
    </Container>
  </Box>)
}

export {VcPromptKiosk, VidChainRedirect};
