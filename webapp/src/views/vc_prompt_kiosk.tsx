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
import VerifiedIcon from '@mui/icons-material/Verified';
import DoNotDisturbOnIcon from '@mui/icons-material/DoNotDisturbOn';
import ReportProblemIcon from '@mui/icons-material/ReportProblem';
import SendToMobileIcon from '@mui/icons-material/SendToMobile';
import { Head2 } from "../theme";

const VcPromptKiosk = () => {
  const dataProvider = useDataProvider();
  const { access_token } = useParams();
  const [vcRequest, setVcRequest] = useSafeSetState();
  const [doneRequest, setDoneRequest] = useSafeSetState(null);
  const translate = useTranslate();
  const notify = useNotify();

  const create = async () => {
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.create('KioskVcRequest', { data: { input: null } });
      clearAccessToken();
      setDoneRequest(null);
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
    let timeout;
    async function load(){
      if(!vcRequest) { return; }

      setAccessToken(access_token);
      let value = await dataProvider.getOne('KioskVcRequest', { id: vcRequest.id });
      clearAccessToken();

      if( value.data.state != "PENDING" ) {
        setDoneRequest(value.data);
        setVcRequest(null);
        setTimeout(create, 4000);
      } else {
        setVcRequest(value.data);
      }
    }
    timeout = setTimeout(load, 1000);
    return function cleanup() { clearTimeout(timeout); };
  }, [vcRequest]);

  return (<Box sx={{ minHeight: "100vh", display: "flex", flexDirection: "column", }}>
    <CssBaseline/>
    <Container maxWidth="md" id="vc-prompt">
      <Box textAlign="center" mt={4} mb={2}>
        <div style={{ height: "auto", margin: "0 auto", maxWidth: 500, width: "100%" }}>
          { !vcRequest && !doneRequest && <Skeleton variant="rectangular" height="auto" sx={{ maxWidth: "100%", width: "100%"}} /> }
          { vcRequest && !doneRequest && <CurrentVcRequest request={vcRequest} /> }
        
          { doneRequest && <DoneVcRequest request={doneRequest} />}
        </div>
      </Box>
    </Container>
  </Box>)
}

const CurrentVcRequest = ({request}) => {
  const translate = useTranslate();

  return <Box>
    <KioskLogo logo={request.logoUrl} />
    <Head2 textAlign="center" sx={{ mb:"1em" }}>
      { request.description }
    </Head2>
    { request.vidchainUrl ?
      <QRCode
        size={256}
        style={{ height: "auto", maxWidth: "100%", width: "100%" }}
        value={request.vidchainUrl}
        viewBox={`0 0 256 256`}
      />
      : 
      <Skeleton variant="rectangular" sx={{ margin: "auto", height: "256px", width: "256px", maxWidth: "100%", maxHeight: "100%"}} />
    }
    <Typography textAlign="center" mt={3}>
      { translate("vc_validator.kiosk.instructions_1") }
    </Typography>
  </Box>;
}

const DoneVcRequest = ({request}) => {
  const translate = useTranslate();
  const dimensions = { height: 200, width: 200 };

  return <Box textAlign="center">
    <KioskLogo logo={request.logoUrl} />
    { request.state== "APPROVED" &&
      <Box color="#00a975">
        <VerifiedIcon sx={dimensions} />
        <Head2>{ translate("vc_validator.kiosk.accepted") }</Head2>
      </Box>
    }
    { request.state == "REJECTED" &&
      <Box color="#c60042">
        <DoNotDisturbOnIcon sx={dimensions} />
        <Head2>{ translate("vc_validator.kiosk.rejected") }</Head2>
      </Box>
    }
    { request.state == "FAILED" &&
      <Box color="#c60042">
        <ReportProblemIcon sx={dimensions} />
        <Head2>{ translate("vc_validator.kiosk.failed") }</Head2>
      </Box>
    }
  </Box>;
}

const KioskLogo = ({ logo }) => 
  <img src={logo} style={{maxHeight: "10em", marginBottom: "2em" }} />

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

  const dimensions = { height: 200, width: 200 };

  return (<Box sx={{ minHeight: "100vh", display: "flex", flexDirection: "column", }}>
    <CssBaseline/>
    <Container maxWidth="md" sx={{ display: "flex" }}>
      <Box sx={{ margin: "2em auto", alignSelf: "center" }} >
        { !done && <Skeleton variant="rectangular" sx={dimensions} /> }
        { done && <Box textAlign="center">
            <SendToMobileIcon sx={dimensions} />
            <Head2>{ translate("vc_validator.kiosk.redirect_success") }</Head2>
          </Box>
        }
      </Box>
    </Container>
  </Box>)
}

export {VcPromptKiosk, VidChainRedirect};
