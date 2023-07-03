import { useEffect } from 'react';
import { Dialog, DialogContent, DialogTitle, DialogActions, Card, CardContent, Typography, Container, Box, LinearProgress, Button, IconButton, Link } from '@mui/material';
import { Datagrid, UrlField, DateField, TextField, FunctionField, List, useNotify, useCreateController, useTranslate, useDataProvider, Form, NumberInput, required, minValue, maxValue, TextInput, SelectInput, SimpleForm, useRefresh, useRecordContext, ReferenceInput, ReferenceField } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { useParams, useNavigate } from 'react-router-dom';
import { setAccessToken, clearAccessToken } from '../components/auth_provider';
import CardTitle from '../components/card_title';
import ConstataSkeleton from '../components/skeleton';
import { NoLoggedInLayout } from './layout';
import FilterTextInput from "../components/filter_textinput";
import { Head1 } from '../theme';
import QRCode from "react-qr-code";
import {
  ListActionsWithoutCreate, PaginationDefault, downloadFile,
} from '../components/utils';
import logo_cobranding from '../assets/logo_cobranding.png';
import VerifiedIcon from '@mui/icons-material/Verified';
import DoNotDisturbOnIcon from '@mui/icons-material/DoNotDisturbOn';
import ReportProblemIcon from '@mui/icons-material/ReportProblem';
import LinkIcon from '@mui/icons-material/Link';
import PendingOutlinedIcon from '@mui/icons-material/PendingOutlined';
import CloseIcon from '@mui/icons-material/Close';

const VcPromptDashboard = () => {
  const translate = useTranslate();

  return (<Container maxWidth="md" id="vc-prompt-dashboard">
    <Box mb={3}>
      <Head1 sx={{ mb:2 }}>{ translate("vc_validator.dashboard.title") }</Head1>
      <Typography>{ translate("vc_validator.dashboard.subtitle") }</Typography>
    </Box>
    <NewPromptDialog />
    <VcPromptList />
    <br/>
    <VcRequestList />
    <br/>
    <Box textAlign="center" mt={8} mb={4}>
      <img src={logo_cobranding} style={{maxWidth: "200px", width: "50%" }} />
    </Box>
  </Container>)
}

const NewPromptDialog = () => {
  const translate = useTranslate();
  const [open, setOpen] = useSafeSetState(false);
  const dataProvider = useDataProvider();
  const notify = useNotify();

  const handleClose = () => {
    setOpen(false);
  };
  
  const handleSubmit = async (values) => {
    try {
      await dataProvider.create('VcPrompt', { data: { input: values }});
    } catch (e) {
      notify(e.toString(), { type: 'error' });
    }
    handleClose();
  }

  return (<Box id="recipients">
    <Button fullWidth size="large" variant="contained" onClick={() => setOpen(true) } sx={{ fontSize: 20, mb: 5 }}>
      { translate("vc_validator.new_prompt.button_label") }
    </Button>
    <Dialog open={open} fullWidth onClose={handleClose}>
      <SimpleForm onSubmit={handleSubmit}>
        <TextInput fullWidth source="name" label="resources.VcPrompt.name" validate={[required()]}
          helperText="vc_validator.new_prompt.nameHelperText"
        />
        <ReferenceInput source="vcRequirementId" reference="VcRequirement" >
          <SelectInput fullWidth label="resources.VcPrompt.vcRequirementId" optionText="name" validate={[required()]}
            helperText="vc_validator.new_prompt.vcRequirementIdHelperText"
          />
        </ReferenceInput>
      </SimpleForm>
    </Dialog>
  </Box>);
}

function VcPromptList() {
  const translate = useTranslate();

  return (
    <Card>
      <CardTitle text="vc_validator.prompt_list.title" />
      <List
        empty={<Box sx={{m: 1}}>{ translate("vc_validator.prompt_list.empty_message") }</Box>}
        resource="VcPrompt"
        perPage={20}
        sort= {{ field: 'id', order: 'DESC' }}
        actions={false}
      >
        <Datagrid
          bulkActionButtons={false}
          sx={{ '& .column-name': { width: "100%" } }}
        >
          <TextField source="id" sortable={false} />
          <Box source="name" label="resources.VcPrompt.name" display="flex" alignItems="center" flexWrap="wrap" gap="1em">
            <Box flex="2" minWidth="200px">
              <TextField source="name" sortable={false} />
              <br/>
              <ReferenceField label="VcRequirement" source="vcRequirementId" reference="VcRequirement">
                <TextField source="name" variant="caption"/>
              </ReferenceField>
            </Box>
            <ConfigureVcPrompt />
          </Box>
        </Datagrid>
      </List>
    </Card>
  );
}

const ConfigureVcPrompt = () => {
  const translate = useTranslate();
  const notify = useNotify();
  const record = useRecordContext();
  const [open, setOpen] = useSafeSetState(false);

  const handleClose = () => {
    setOpen(false);
  };

  const copyToClipboard = (toCopy) => {
    navigator.clipboard.writeText(toCopy);
    notify("vc_validator.configure.copied_to_clipboard");
  }
  
  return (<Box>
    <Button sx={{whiteSpace:"nowrap"}} fullWidth size="small" variant="outlined" onClick={() => setOpen(true) }>
      { translate("vc_validator.configure.button_label") }
    </Button>
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidht>
      <DialogTitle>
        <Box display="flex" gap="1em">
          <Typography> { translate("vc_validator.configure.description") }</Typography>
          <IconButton sx={{ width: "50px", height: "50px" }} aria-label="close" onClick={handleClose} > <CloseIcon /> </IconButton>
        </Box>
      </DialogTitle>
      <DialogContent>
        <Box>
          <QRCode
            size={256}
            style={{ height: "auto", maxWidth: "100%", width: "100%" }}
            value={record.fullUrl}
            viewBox={`0 0 256 256`}
          />
        </Box>
        <Box>
          <TextField variant="body2" value={record.fullUrl}></TextField>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button sx={{m: 1 }}
          fullWidth
          startIcon={<LinkIcon />}
          onClick={() => copyToClipboard(record.fullUrl)}
          variant="contained"
          id="copy-full-url"
          >
          {translate("vc_validator.configure.copy_to_clipboard")}
        </Button>
      </DialogActions>
    </Dialog>
  </Box>);
}

function VcRequestList() {
  const translate = useTranslate();
  const refresh = useRefresh();

  useEffect(() => {
    let interval = setInterval(() =>  refresh(), 2000);
    return function cleanup() { clearInterval(interval); };
  }, []);

  return (
    <Card>
      <CardTitle text="vc_validator.request_list.title" />
      <List
        empty={<Box sx={{m: 1}}>{ translate("vc_validator.request_list.empty_message")} </Box>}
        resource="VcRequest"
        perPage={10}
        sort = {{ field: 'id', order: 'DESC' }}
        actions={false}
        disableSyncWithLocation
      >
        <Datagrid
          bulkActionButtons={false}
          sx={{ '& .column-promptId': { width: "100%" }, '& .column-finishedAt': { whiteSpace: 'nowrap' } }}
        >
          <FunctionField sortable={false} source="state" label={false} render={ record => {
            const dimensions = { height: 30, width: 30 };
            switch(record.state) {
              case "PENDING":
                return <Box> <PendingOutlinedIcon sx={dimensions} /> </Box>;
              case "APPROVED":
                return <Box color="#00a975"> <VerifiedIcon sx={dimensions} /> </Box>;
              case "REJECTED":
                return <Box color="#c60042"> <DoNotDisturbOnIcon sx={dimensions} /> </Box>;
              case "FAILED":
                return <Box color="#c60042"> <ReportProblemIcon sx={dimensions} /> </Box>;
            }
          }} />
          <ReferenceField sortable={false} source="promptId" label="resources.VcRequest.promptId" reference="VcPrompt">
            <TextField source="name" />
          </ReferenceField>
          <FunctionField sortable={false} source="finishedAt" label="resources.VcRequest.finishedAt" render={ record => {
            if(!record.finishedAt){ return "…"; }

            let treshold = new Date(record.finishedAt).getTime() + (24 * 60 * 60 * 1000);
            return treshold < new Date() ?
              <DateField source="finishedAt"/> :
              <DateField source="finishedAt" showTime={true} showDate={false}/> ;
          }} />
          <TextField source="id" sortable={false} />
        </Datagrid>
      </List>
    </Card>
  );
}

export { VcPromptDashboard };
