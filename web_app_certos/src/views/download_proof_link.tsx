import { useEffect } from 'react';
import { Card, CardContent, Typography, Container, Button, Alert,
         AlertTitle, Box } from '@mui/material';
import { useTranslate, useDataProvider, useNotify } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import CardTitle from '../components/card_title';
import { useParams, useNavigate } from 'react-router-dom';
import { NoLoggedInLayout } from './layout';
import { setAccessToken, clearAccessToken } from '../components/auth_provider';
import IframeCertificate from '../components/iframe';
import Loading from './loading'
import ConstataSkeleton from '../components/skeleton';
import { parseDate } from '../components/utils';
import { ShareToLinkedin, ShareToTwitter } from '../components/share_to_social_media';
import LinkIcon from '@mui/icons-material/Link';
import LaunchIcon from '@mui/icons-material/Launch';
import { StopCircle } from '@mui/icons-material';
import DeleteDownloadProofLink from '../components/delete_download_proof_link_modal';


interface IDownloadProofLink {
  id?: number,
  validUntil?: string,
  pendingDocCount?: number,
  lastDocDate?: string,
  publicCertificateUrl?: string,
  sharedText?: string,
}
  
const DownloadProofLink = () => {
  const { access_token } = useParams();
  const [state, setState] = useSafeSetState<string>("loading");
  const [downloadProofLink, setDownloadProofLink] = useSafeSetState<IDownloadProofLink>({});
  const navigate = useNavigate();
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const notify = useNotify();

  useEffect(() => {
    const init = async () => {
      setAccessToken(access_token);
      try {
        const {data} = await dataProvider.getOne('DownloadProofLink', { id: 1 });
        setDownloadProofLink(data);
        setState("init");
      } catch (e) { 
        setState(e.status === 401 ? "warning" : "error")
      }
      clearAccessToken();
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps  
  }, []);

  const handleDownload = async () => {
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.getOne('Proof', { id: 1 });
      const blob = new Blob([data.html], { type: 'text/html' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = "constata_certificate.html";
      a.click();
    } catch (e) { 
      notify("certos.download_proof_link.error_download", { type: 'error' });
    }
    clearAccessToken();
  }


  const handleChangePublicCertificateState = async (action: string) => {
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.update('DownloadProofLink', { id: 1, data: {input: {action}}, previousData: {} });
      setDownloadProofLink(data);
    } catch (e) {
      notify("certos.download_proof_link.error_public_certificate_state", { type: 'error' });
    }
    clearAccessToken();
  }

  const props = {
    handleView: () => { navigate("/safe/" + access_token + "/show") },
    handleDownload,
    downloadProofLink,
    handleChangePublicCertificateState,
    setState,
  }

  return (<NoLoggedInLayout>
    <Container maxWidth="md">
      { state === "loading" && <ConstataSkeleton title={translate("certos.download_proof_link.title")} lines={3} /> }
      { state === "warning" && <Expired /> }
      { state === "error" && <Error /> }
      { state === "delete" && <Deleted /> }
      { state === "init" && <Administrate {...props} /> }
    </Container>
  </NoLoggedInLayout>
)}

const Administrate = (props) => {
  return <Box>
    <DownloadOrView {...props} />
    <Share {...props} />
    <DeleteAdminAccess {...props} />
  </Box>
}
  
const DownloadOrView = ({downloadProofLink, handleDownload, handleView}) => {
  const translate = useTranslate();
  
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.download_proof_link.title")} />
    <CardContent >
      <Typography>
        {downloadProofLink.validUntil ?
          translate("certos.download_proof_link.text", {validUntil: parseDate(downloadProofLink.validUntil)})
          :
          translate("certos.download_proof_link.text_without_date")
        } {translate("certos.download_proof_link.text_2")}
      </Typography>
      {downloadProofLink.pendingDocCount > 0 && <Box>
        <Typography variant="h5" sx={{mt: 2, fontWeight: 500}} id="pending_docs_title">{ translate("certos.download_proof_link.more_title") }</Typography>
        <Typography mt={0.5}>{ translate("certos.download_proof_link.more_text_1") }</Typography>
        <Typography mt={0.5}>{ translate("certos.download_proof_link.more_text_2", {lastDocDate: parseDate(downloadProofLink.lastDocDate)}) }</Typography>
        <Typography mt={0.5}>{ translate("certos.download_proof_link.more_text_3", downloadProofLink.pendingDocCount) }</Typography>
      </Box>
      }

      <Box mt={2}>
        <Button
          sx={{mt: 1}}
          id="safe-button-view"
          fullWidth
          size="large"
          variant="contained"
          onClick={handleView}
        >
          {translate("certos.download_proof_link.button_view")}
        </Button>
        <Button
          sx={{mt: 1}}
          id="safe-button-download"
          fullWidth
          size="large"
          variant="outlined"
          onClick={handleDownload}
        >
          {translate("certos.download_proof_link.button_download")}
        </Button>
      </Box>
    </CardContent>
  </Card>
};


const Share = ({downloadProofLink, handleChangePublicCertificateState}) => {
  const translate = useTranslate();
  const notify = useNotify();

  const copyToClipboard = (toCopy) => {
    navigator.clipboard.writeText(toCopy);
    notify("certos.actions.copy_to_clipboard");
  }
  
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.download_proof_link.share.title")} />
    <CardContent >
      {downloadProofLink.publicCertificateIsActive ?
        <Box>
          <ShareToLinkedin
            url={downloadProofLink.publicCertificateUrl}
            text={downloadProofLink.shareOnSocialNetworksCallToAction}
          />
          <ShareToTwitter
            url={downloadProofLink.publicCertificateUrl}
            text={downloadProofLink.shareOnSocialNetworksCallToAction}
          />
          <Button
            sx={{mx: 0.5, my: 1}}
            startIcon={<LinkIcon />}
            onClick={() => copyToClipboard(downloadProofLink.publicCertificateUrl)}
            variant="contained"
            id="copy-certificate-to-clipboard"
            >
            {translate("certos.download_proof_link.copy_to_clipboard")}
          </Button>
          <Button
            sx={{mx: 0.5, my: 1}}
            href={downloadProofLink.publicCertificateUrl}
            target="_blank"
            startIcon={<LaunchIcon/>}
            variant="outlined"
            id="go-to-public-certificate"
            >
            {translate("certos.download_proof_link.link_to_public_certificate")}
          </Button>
          <Button
            sx={{mx: 0.5, my: 1}}
            onClick={() => handleChangePublicCertificateState("unpublish")}
            variant="outlined"
            id="safe-button-change-public-certificate-state"
            startIcon={<StopCircle />}
            color="error"
          >
            { translate("certos.download_proof_link.button_deactivate_public_certificate") }
          </Button>
        </Box>
      :
      <Box>
        <Typography>{ translate("certos.download_proof_link.share.text") }</Typography>
        <Button
          sx={{mt: 2}}
          id="safe-button-change-public-certificate-state"
          fullWidth
          size="large"
          variant="contained"
          onClick={() => handleChangePublicCertificateState("publish")}
        >
          { translate("certos.download_proof_link.button_activate_public_certificate") }
        </Button>
      </Box>
      }
    </CardContent>
  </Card>
};

const DeleteAdminAccess = ({setState}) => {
  const translate = useTranslate();
  const { access_token } = useParams();
  
  return (<Alert variant="outlined" severity="error" sx={{ mb: 5 }} icon={false}>
  <AlertTitle>{ translate("certos.download_proof_link.delete.title") }</AlertTitle>
  <Typography sx={{mt: 1}}>{ translate('certos.download_proof_link.delete.text') }</Typography>
  <DeleteDownloadProofLink setState={setState} access_token={access_token} />
</Alert>);
}

const CertificateShow = () => {
  const { access_token } = useParams();
  const [state, setState] = useSafeSetState<string>("loading");
  const [urlObject, setUrlObject] = useSafeSetState(null);
  const dataProvider = useDataProvider();

  useEffect(() => {
    const init = async () => {
      setAccessToken(access_token);
      try {
        const {data} = await dataProvider.getOne('Proof', { id: 1 });
        const blob = new Blob([data.html], { type: 'text/html' });
        setUrlObject(URL.createObjectURL(blob));
        window.dispatchEvent(new Event("load"));
        setState("valid");
      } catch (e) { 
        setState(e.status === 401 ? "warning" : "error")
      }
      clearAccessToken();
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps  
  }, []);

  return (<>
    { state === "loading" && <Loading /> }
    { state === "warning" && <Expired /> }
    { state === "error" && <Error /> }
    { state === "valid" && <IframeCertificate url={urlObject} id="iframe-valid-certificate" /> }
  </>)
}

const Error = () => {
  const translate = useTranslate();
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.download_proof_link.error.title")} />
    <CardContent>
      <Typography>{translate("certos.download_proof_link.error.text")}</Typography>
    </CardContent>
  </Card>
};

const Expired = () => {
  const translate = useTranslate();
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.download_proof_link.expired.title")}/>
    <CardContent>
      <Typography> {translate("certos.download_proof_link.expired.text_1")} </Typography>
      <Typography> {translate("certos.download_proof_link.expired.text_2")} </Typography>
    </CardContent>
  </Card>
}

const Deleted = () => {
  const translate = useTranslate();
  return <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.download_proof_link.deleted.title")} id="deleted-link"/>
    <CardContent>
      <Typography> {translate("certos.download_proof_link.deleted.text")} </Typography>
    </CardContent>
  </Card>
}


export {DownloadProofLink, CertificateShow};
