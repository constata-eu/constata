import { useEffect } from 'react';
import { Dialog, DialogContent, DialogActions, Card, CardContent, Typography, Container, Box, LinearProgress, Button, Link } from '@mui/material';
import { Datagrid, UrlField, TextField, FunctionField, List, useNotify, useCreateController, useTranslate, useDataProvider, Form, NumberInput, required, minValue, maxValue, TextInput, SelectInput, SimpleForm, useRefresh, useRecordContext, ReferenceInput, ReferenceField } from 'react-admin';
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

const VcPromptDashboard = () => {
  const translate = useTranslate();

  return (<Container maxWidth="md" id="vc-prompt-dashboard">
    <Box mb={3}>
      <Head1 sx={{ mb:2 }}>Verification points</Head1>
      <Typography>
        Credential verification points let you request verifiable credentials from a mobile device on physical locations.
      </Typography>
    </Box>
    <NewPromptDialog />
    <VcPromptList />
    <br/>
    <VcRequestList />
    <br/>
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
      Create Verification Point
    </Button>
    <Dialog open={open} fullWidth onClose={handleClose}>
      <SimpleForm onSubmit={handleSubmit} record={{}}>
        <TextInput fullWidth source="name" />
        <ReferenceInput source="vcRequirementId" reference="VcRequirement">
          <SelectInput fullWidth label="Rules" optionText="name" />
        </ReferenceInput>
      </SimpleForm>
    </Dialog>
  </Box>);
}

function VcPromptList() {
  const translate = useTranslate();

  return (
    <Card>
      <CardTitle text="Active verification points" />
      <List
        empty={<Box sx={{m: 1}}>Create a new verification point. You can have as many as you want!</Box>}
        resource="VcPrompt"
        perPage={20}
        sort= {{ field: 'id', order: 'DESC' }}
        pagination={false}
        actions={false}
      >
        <Datagrid
          bulkActionButtons={false}
          sx={{ '& .column-name': { width: "100%" } }}
        >
          <TextField source="id" sortable={false} />
          <Box source="name" width="100%">
            <TextField source="name" sortable={false} />
            <br/>
            <ReferenceField label="VcRequirement" source="vcRequirementId" reference="VcRequirement">
              <TextField source="name" variant="caption"/>
            </ReferenceField>
          </Box>
          <ConfigureVcPrompt />
        </Datagrid>
      </List>
    </Card>
  );
}

const ConfigureVcPrompt = (url) => {
  const record = useRecordContext();
  const [open, setOpen] = useSafeSetState(false);

  const handleClose = () => {
    setOpen(false);
  };
  
  return (<Box id="recipients" sx={{ width:"150px"}}>
    <Button fullWidth size="small" variant="outlined" onClick={() => setOpen(true) }>
      Setup mobile device
    </Button>
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidht>
      <DialogContent>
        <Typography mb={3}>
          Visite esta URL en cualquier dispositivo para convertirlo en un punto de verificación móvil.
        </Typography>
        <Box>
          <QRCode
            size={256}
            style={{ height: "auto", maxWidth: "100%", width: "100%" }}
            value={record.fullUrl}
            viewBox={`0 0 256 256`}
          />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button sx={{m: 1}} variant="contained" fullWidth href={record.fullUrl} target="_blank">Abrir aquí</Button>
        <Button sx={{m: 1}} variant="outlined" fullWidth onClick={handleClose}>Cerrar</Button>
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
      <CardTitle text="Latest attempts" />
      <List
        empty={<Box sx={{m: 1}}>When people start trying to access any of your verification points you'll see it here</Box>}
        resource="VcRequest"
        perPage={100}
        pagination={false}
        sort = {{ field: 'id', order: 'DESC' }}
        actions={false}
      >
        <Datagrid bulkActionButtons={false}>
          <TextField source="id" sortable={false} />
          <TextField source="state" sortable={false}/>
          <TextField source="startedAt" sortable={false}/>
        </Datagrid>
      </List>
    </Card>
  );
}

export { VcPromptDashboard };
