import { useEffect, useRef} from 'react';
import {
    Alert, AlertTitle, LinearProgress, Typography, Box, Button, Divider, Card, CardContent,
    CardActions, IconButton, Dialog, DialogActions, DialogContent,
    DialogContentText, DialogTitle, Container, Link, Grid
} from '@mui/material';
import { convertBase64, openBlob, handleErrors } from '../components/utils'
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';
import DeleteIcon from '@mui/icons-material/Delete';
import UploadFileIcon from '@mui/icons-material/UploadFile';
import { getStorage } from "../components/auth_provider";
import { useParams, useNavigate } from 'react-router-dom';
import { getKeyPair, getSignedPayload } from "../components/cypher";
import _ from 'lodash';
import {v4 as uuid} from 'uuid';
import { useList, ListContextProvider, SimpleList, required, useTheme, useGetOne, useGetList } from 'react-admin';
import csv from 'csvtojson';
import CardTitle from '../components/card_title';
import { openPreview } from "./issuance"
import { stringify } from 'csv-stringify/sync';
import DiplomaPreview from "../assets/example_diploma.png";
import AttendancePreview from "../assets/example_attendance.png";
import InvitationPreview from "../assets/example_invitation.png";
import {
  Form,
  TextInput,
  AutocompleteInput,
  ReferenceInput,
  useDataProvider,
  useTranslate,
  useNotify,
  useSafeSetState,
  ImageInput,
  ImageField,
  useCheckAuth,
  Pagination,
} from 'react-admin';
import ConstataSkeleton from '../components/skeleton';

interface Error {
  message: string,
  args: any,
}

interface TemplateErrors  {
  newLogoText?: string,
  newName?: Error | string | false,
}

const Template = ({handleNext, wizardState, setWizardState}) => {
  const translate = useTranslate();
  const form = useRef<any>(undefined);
  const dataProvider = useDataProvider();

  const validate = (values) => {
    if(values.templateId){ return {}; }
    
    let errors: TemplateErrors = {};
    if(!values.newLogoImage && !values.newLogoText) {
      const err = wizardState.has_templates ? "error_choose_existing_or_new" : "error_choose_logo_or_text";
      errors.newLogoText = translate(`certos.wizard.template.${err}`)
    }
    
    errors.newName = required()(values.newName, values) || false; 
    
    return errors;
  };
  
  const submit = async (values) => {
    if(values.templateId) {
      const template = await dataProvider.getOne("Template", {id: values.templateId});
      values.templateName = template.data.name;
      values.kind = template.data.kind;
      values.schema = JSON.parse(template.data.schema);
    }

    await setWizardState((s) => ({...s, ...values}));
    handleNext();
  }
  
  const onSelectTemplate = async () => {
    setTimeout(() => form.current.parentNode.requestSubmit() , 1);
  }

  const imagePreview = {
    "DIPLOMA": DiplomaPreview,
    "ATTENDANCE": AttendancePreview,
    "INVITATION": InvitationPreview,
  }[wizardState.kind];

  return (<Form onSubmit={submit} validate={validate} noValidate id="template-wizard-step">
    <Card ref={form} sx={{mb:5}}>
      <CardTitle text="certos.wizard.template.title" />
      <CardContent>
        <Box mb={3}>
          <Typography variant="body1">{ translate("certos.wizard.template.text") }</Typography>
        </Box>
        { wizardState.hasTemplates && <Box>
          <Divider sx={{mb: 2}}>{ translate("certos.wizard.template.choose_template_divider") }</Divider>
          <ReferenceInput sx={{mb: 0, pb: 0}} source="templateId" reference="Template" filter={{archivedEq: false}} >
            <AutocompleteInput
              onChange={onSelectTemplate}
              label="certos.wizard.template.choose_template_label"
              helperText={false}
              optionText={(r) => `${translate(`certos.wizard.kind.${r.kind}`)} - ${r.name}` }
            />
          </ReferenceInput>
          <Divider sx={{my: 3}}>{ translate("certos.wizard.template.new_template_divider") }</Divider>
          </Box>
        }
        <ToggleButtonGroup
          sx={{mb: 1}}
          fullWidth
          exclusive
          value={wizardState.kind}
          onChange={ (value: any) => setWizardState((s) => ({...s, kind: value.target.value})) }
          aria-label="text alignment"
        >
          <ToggleButton value="DIPLOMA" aria-label="left aligned">
            { translate("certos.wizard.kind.DIPLOMA") }
          </ToggleButton>
          <ToggleButton value="ATTENDANCE" aria-label="centered">
            { translate("certos.wizard.kind.ATTENDANCE") }
          </ToggleButton>
          <ToggleButton value="INVITATION" aria-label="right aligned">
            { translate("certos.wizard.kind.INVITATION") }
          </ToggleButton>
        </ToggleButtonGroup>
        <Box component="div" display="flex" position="relative" flexDirection="column" alignItems="center" sx={{backgroundColor:"#333"}} p={2} mb={2}>
          <Box 
            component="div"
            style={{ opacity: "0.3" }}
            position="absolute"
            fontFamily="ManropeExtraBold"
            color="white"
            sx={{backgroundColor:"#333"}}
            fontSize={{
              xs: "2em",
              sm: "4em"
            }}
            margin="auto"
            textAlign="center"
            textTransform="uppercase"
            bottom={0}
            py={3}
            width="100%"
          >
            { translate("certos.wizard.template.sample") }
          </Box>
          <img src={imagePreview} alt={translate("certos.wizard.template.sample")} style={{ maxWidth: "100%", maxHeight: "80vh" }} />
        </Box>
        <TextInput fullWidth label="certos.wizard.template.new_name_label" source="newName" variant="filled" required
          helperText="certos.wizard.template.new_name_helper_text" />
        <TextInput label="certos.wizard.template.new_logo_text_label" source="newLogoText" variant="filled" fullWidth
          helperText="certos.wizard.template.new_logo_text_helper_text" />
        <ImageInput
          sx={{
            my: 2,
            '& .RaFileInput-dropZone': { background: 'inherit', p: 0 },
            '& .RaFileInput-removeButton': { display: 'block' }
          }}
          fullWidth
          source="newLogoImage"
          label={false}
          helperText={ translate("certos.wizard.template.new_logo_image_help") }
          accept="image/png,image/jpeg"
          placeholder={ <Button fullWidth variant="outlined">{ translate("certos.wizard.template.new_logo_image_label") }</Button> }
        >
          <ImageField sx={{ textAlign: "center", maxWidth: "100%" }} source="src" title="title" />
        </ImageInput>
      </CardContent>
      <CardActions>
        <Button fullWidth variant="contained" type="submit">
          { translate("certos.wizard.template.button_next") }
        </Button>
      </CardActions>
    </Card>

    <Alert sx={{ mb: 2 }} severity="info" variant="outlined" icon={false}>
      <AlertTitle>{ translate("certos.wizard.template.custom_template_offer.title") }</AlertTitle>
      { translate("certos.wizard.template.custom_template_offer.text") }
      <Button fullWidth sx={{ my: 2 }} variant="contained" target="_blank"
        href={`mailto:hola@constata.eu?subject=${translate("certos.wizard.template.custom_template_offer.subject")}`}
      >
        { translate("certos.wizard.template.custom_template_offer.button") }
      </Button>
    </Alert>
  </Form>);
}

const TemplateDone = ({wizardState, step, setStep}) => {
  const translate = useTranslate();

  let text: String;
  if(wizardState.templateName) {
    text = translate("certos.wizard.template.done.using_existing", {name: wizardState.templateName});
  } else if (wizardState.newLogoImage) {
    text = translate("certos.wizard.template.done.creating_new_one_with_image");
  } else {
    text = translate("certos.wizard.template.done.creating_new_one_with_text", {text: wizardState.newLogoText});
  }

  return (<DoneStep>
    <Box display="flex" alignItems="center">
      <Typography variant="body1" sx={{ fontSize: 14 }}>
        { translate(`certos.wizard.kind.${wizardState.kind}`) }: { text }
      </Typography>
      <Box component="div" sx={{ flex: 1 }} />
      { step < 2 && <Button size="small" variant="outlined" onClick={ () => setStep(0) }>{ translate("certos.wizard.template.done.change") }</Button> }
    </Box>
  </DoneStep>);
}

const RecipientDialog = ({wizardState, handleAddRecipient, validate}) => {
  const translate = useTranslate();
  const [open, setOpen] = useSafeSetState(false);

  const handleClickOpen = () => {
    setOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
  };
  
  const handleSubmit = (values) => {
    handleClose();
    handleAddRecipient(values);
  }
  
  let schema = wizardState.schema;
  if (!wizardState.canSendEmail) {
    schema = _.filter(schema, (x) => x.name !== 'email')
  }
  const fields = schema.map((o) => <TextInput
    defaultValue={wizardState.defaultSchemaValues[o.name]}
    source={o.name}
    key={"input_" + o.name}
    label={ o.label || translate(`certos.wizard.recipients.schema.${wizardState.kind}.${o.name}.label`) }
    helperText={ o.help || translate(`certos.wizard.recipients.schema.${wizardState.kind}.${o.name}.help`) }
    variant="filled"
    required={!o.optional}
    fullWidth
  /> );

  return (<Box id="recipients">
    <Button fullWidth variant="outlined" onClick={handleClickOpen}>
      { translate("certos.wizard.recipients.form.open") }
    </Button>
    <Dialog open={open} onClose={handleClose}>
      <Form onSubmit={handleSubmit} validate={validate} noValidate>
        <DialogTitle>{ translate("certos.wizard.recipients.form.title") }</DialogTitle>
        <DialogContent>{fields}</DialogContent>
        <DialogActions>
          <Button onClick={handleClose}>{ translate("certos.wizard.recipients.form.cancel") }</Button>
          <Button type="submit">{ translate("certos.wizard.recipients.form.submit") }</Button>
        </DialogActions>
      </Form>
    </Dialog>
  </Box>);
}

const Recipients = (props) => {
  const [theme] = useTheme();
  const translate = useTranslate();
  const [formKey, setFormKey] = useSafeSetState(uuid())
  const [uploadErrors, setUploadErrors] = useSafeSetState(null)
  const commonSchema = props.wizardState.schema.filter((i) => i.common).map((i) => i.name);
  const listContext = useList({
    perPage: 10,
    data: props.wizardState.recipients
  });

  const handleAddRecipient = (data) => {
    props.setWizardState((s) => {
      data.id = uuid();
      s.recipients.push(data);
      s.defaultSchemaValues = _.pick(data, commonSchema);
      return { ...s, ...{} };
    })
    setFormKey(uuid())
  }

  const handleRemoveRecipient = (id) => {
    props.setWizardState((s) => {
      return { ...s, ...{recipients: s.recipients.filter((x) => x.id !== id)}};
    })
  }

  const handleDownloadExampleCsv = async () => {
    const header = props.wizardState.schema.map((r) => r.name).join(";");
    const row = props.wizardState.schema.map((r) =>
      r.sample ||
      translate(`certos.wizard.recipients.schema.${props.wizardState.kind}.${r.name}.sample`)
    ).join(";");
      
    const blob = new Blob([header, "\n", row], {type : 'text/csv'});
    await openBlob(blob);
  }

  const validate = (values) => {
    const errors: any = {};
    for( const n of props.wizardState.schema) {
      if(n.optional) { continue; }
      if(!values[n.name] || values[n.name].trim() === "") {
        errors[n.name] = "Es requerido";
      }
    }
    return errors;
  };

  const handleFileUpload = async (event) => {
    const buffer = await event.target.files[0].arrayBuffer();
    let text;
    try {
      text = new TextDecoder("utf-8", { fatal: true }).decode(buffer);
    } catch(e) {
      text = new TextDecoder("latin1").decode(buffer);
    }

    const rows = await csv({delimiter: [",", ";"]}).fromString(text);
    let errors = [];

    if(_.isEmpty(rows)){
      errors.push(translate("certos.wizard.recipients.errors.invalid_file"));
    }

    _.forEach(rows, (row, i) => {
      _.forEach(_.toPairs(validate(row)), (v) => {
        errors.push(translate("certos.wizard.recipients.errors.in_row", {index: i, column: v[0], err: v[1]}));
      });
    });

    if(_.isEmpty(errors)) {
      _.forEach(rows, handleAddRecipient)
    } else {
      setUploadErrors(errors);
    }
    event.target.value = '';
  }

  return (<Box>
    <Card sx={{mb:5, mt:2}}>
      <CardTitle text="certos.wizard.recipients.title"/>
      <CardContent>
        <Typography>
          { translate("certos.wizard.recipients.text") }
          &nbsp;
          <Link onClick={handleDownloadExampleCsv}>
            { translate("certos.wizard.recipients.example_csv") }
          </Link>
        </Typography>
        <Button
          sx={{my:2}}
          fullWidth
          component="label"
          variant="outlined"
          startIcon={<UploadFileIcon />}
        >
          { translate("certos.wizard.recipients.add_many_button") }
          <input type="file" accept=".csv" hidden onChange={handleFileUpload} />
        </Button>

        <Dialog open={!_.isEmpty(uploadErrors)} onClose={() => setUploadErrors(null) }>
          <DialogTitle>{ translate("certos.wizard.recipients.csv_dialog.title") }</DialogTitle>
          <DialogContent>
            { !_.isEmpty(uploadErrors) && _.map(_.slice(uploadErrors, 0, 10), (e) => 
              <DialogContentText key={uuid()}>{ e }</DialogContentText>
            )}
            <DialogContentText sx={{my: 2}}>
              { translate("certos.wizard.recipients.csv_dialog.please_use_example") }
              <Button onClick={handleDownloadExampleCsv}>
                { translate("certos.wizard.recipients.example_csv") }
              </Button>
            </DialogContentText>
          </DialogContent>
          <DialogActions>
            <Button variant="contained" onClick={() => setUploadErrors(null)}>
              { translate("certos.wizard.recipients.csv_dialog.close") }
            </Button>
          </DialogActions>
        </Dialog>

        <div key={formKey}>
          <RecipientDialog wizardState={props.wizardState} validate={validate} handleAddRecipient={handleAddRecipient} />
        </div>
        <ListContextProvider value={listContext}>
          <SimpleList
            primaryText={(record) => <Grid container justifyContent="space-between" alignItems="center" spacing="auto">
              <Grid item xs={11}> <Typography> {record.name} </Typography> </Grid>
              <Grid item xs={1}>
                <IconButton size="small" aria-label="delete" onClick={() => handleRemoveRecipient(record.id)}>
                  <DeleteIcon />
                </IconButton>
              </Grid>
            </Grid>}
            linkType={false}
            rowStyle={() => ({borderTop: "1px solid", borderColor: theme?.palette?.grey[200]})}
          />
          <Pagination rowsPerPageOptions={[]}  limit={
            <Box my={2} textAlign="center">
              <Typography variant="caption">{translate("certos.wizard.recipients.no_recipients_yet")}</Typography>
            </Box>
          } />
        </ListContextProvider>
      </CardContent>
      <CardActions>
        <Button disabled={ props.wizardState.recipients.length === 0 } id="continue" fullWidth variant="contained" onClick={props.handleNext}>
          { translate("certos.wizard.recipients.button_continue") }
        </Button>
      </CardActions>
    </Card>
    { !props.wizardState.canSendEmail && <Alert variant="outlined" severity="warning" sx={{ mb: 5 }} icon={false}>
      <AlertTitle>{ translate("certos.wizard.recipients.cannot_send_email_warning.title") }</AlertTitle>
      <Typography>{ translate("certos.wizard.recipients.cannot_send_email_warning.text") }</Typography>
      <Button
        fullWidth
        sx={{ my: 2 }}
        variant="contained"
        color="warning"
        href="#/"
      >
        { translate("certos.wizard.recipients.cannot_send_email_warning.button") }
      </Button>
    </Alert> }
  </Box>);
};

const RecipientsDone = ({wizardState}) => {
  const translate = useTranslate();

  return (<DoneStep>
    <Typography variant="body1" sx={{ fontSize: 14 }}>
      { translate("certos.wizard.recipients.done.text", wizardState.recipients.length) }
    </Typography>
  </DoneStep>);
}

const CreateFailed = ({errors}) => {
  const translate = useTranslate();

  return (<Card>
    <CardTitle text="certos.wizard.creating.error" />
    <CardContent>
      <Typography variant="body1" mb={1}>
        { translate("certos.wizard.creating.error_text") }
      </Typography>
      <Typography variant="body1">
        { errors }
      </Typography>
    </CardContent>
  </Card>);
}

const CreateLoading = () => {
  const translate = useTranslate();

  return (<Card>
    <CardTitle text="certos.wizard.creating.title"/>
    <CardContent>
      <Typography variant="body1">
        { translate("certos.wizard.creating.text") }
      </Typography>
    </CardContent>
    <LinearProgress />
  </Card>);
}

const Creating = ({setCreatedIssuance, ...props}) => {
  const dataProvider = useDataProvider();

  useEffect(() => {
    let interval;
    async function load(){
      let value = await dataProvider.getOne('Issuance', { id: props.wizardState.issuance.id  });
      switch(value.data.state) {
        case "failed":
          props.setWizardState((s) => ({...s, ...{issuance: value.data}}));
          clearInterval(interval)
          break;
        case "created":
          setCreatedIssuance(value.data);
          clearInterval(interval)
          break;
      }
    }
    load();
    interval = setInterval(load, 1000);
    return function cleanup() { clearInterval(interval); };
  }, [dataProvider, props, setCreatedIssuance]);

  let state = props.wizardState.issuance.state;

  return (<Box my={2}>
    { state === "received" && <CreateLoading/> }
    { state === "failed" && <CreateFailed errors={props.wizardState.issuance.errors} /> }
  </Box>);
}

type Schema = {
  name: string,
  optional: boolean,
  common: boolean,
}

type WizardState = {
  name?: string,
  newName?: string,
  templateId?: number,
  templateName?: string,
  newLogoImage?: any,
  newLogoText?: string,
  kind: string,
  schema: Array<Schema>,
  hasTemplates: boolean,
  defaultSchemaValues: Array<Schema>,
  recipients: Array<Map<any, string>>,
  issuance?: any,
  canSendEmail: boolean,
};

const defaultSchema = [
  { name: 'name', optional: false, common: false, label: false, help: false, sample: false },
  { name: 'email', optional: true, common: false, label: false, help: false, sample: false },
  { name: 'recipient_identification', optional: true, common: false, label: false, help: false, sample: false },
  { name: 'custom_text', optional: true, common: false, label: false, help: false, sample: false },
  { name: 'motive', optional: false, common: true, label: false, help: false, sample: false },
  { name: 'date', optional: true, common: true, label: false, help: false, sample: false },
  { name: 'place', optional: true, common: true, label: false, help: false, sample: false },
  { name: 'shared_text', optional: true, common: true, label: false, help: false, sample: false },
];

const CreateIssuance = ({canSendEmail, setCreatedIssuance}) => {
  const translate = useTranslate();
  const dataProvider = useDataProvider();
  const [wizardState, setWizardState] = useSafeSetState<WizardState>( {
    name: null,
    templateId: null,
    templateName: null,
    newLogoImage: null,
    newLogoText: null,
    kind: "DIPLOMA",
    schema: defaultSchema,
    hasTemplates: false,
    defaultSchemaValues: [],
    recipients: [],
    issuance: [],
    canSendEmail: canSendEmail
  });
  const notify = useNotify();
  const [step, setStep] = useSafeSetState(0);

  useEffect(() => {
    const countTemplates = async () => {
      try {
        let {total} = await dataProvider.getList('Template', {
          pagination: { page: 1, perPage: 1 },
          sort: null,
          filter: { archivedEq: false },
        });
        setWizardState((s) => ({...s, ...{hasTemplates: total > 0}}));
      } catch(e) {
        handleErrors(e, notify);
      }
    };
    countTemplates()
  }, [dataProvider, setWizardState, notify]);

  const submitWizard = async () => {
    const newLogoImage = wizardState.newLogoImage &&
      (await convertBase64(wizardState.newLogoImage.rawFile)).split('base64,')[1];

    let matrix = [wizardState.schema.map((o: any) => o.name )];
    for (let recipient of wizardState.recipients) {
      let row = [];
      for (let field of wizardState.schema) {
        row.push(recipient[field.name] || wizardState.defaultSchemaValues[field.name]);
      }
      matrix.push(row)
    }
    const csv = stringify(matrix);
    
    let {data} = await dataProvider.create('CreateIssuanceFromCsv', { data: { input: {
      templateId: wizardState.templateId,
      newKind: wizardState.kind,
      newName: wizardState.newName,
      newLogoText: wizardState.newLogoText,
      newLogoImage,
      name: translate(`certos.wizard.kind.${wizardState.kind}`),
      csv,
    }}});

    setWizardState((s) => ({...s, ...{issuance: data}}));
  }  

  const props = {
    handleNext: async () => {
      try {
        if(step === 1) {
          await submitWizard();
        }
        setStep((prev) => prev + 1);
      } catch(e) {
        handleErrors(e, notify);
      }
    },
    wizardState,
    setWizardState,
  };

  const doneProps = {
    wizardState,
    setWizardState,
    step,
    setStep,
  }

  let show = (i: number, current, done) => step && step > i ? done : ( step === i && current );

  return (
    <Box id="create-issuance-container">
      { show(0, <Template { ...props } />, <TemplateDone { ...doneProps } />) }
      { show(1, <Recipients { ...props } />, <RecipientsDone { ...doneProps }/>) }
      { show(2, <Creating setCreatedIssuance={setCreatedIssuance} { ...props } />, <></>) }
    </Box>
  );
}

const Preview = (props) => {
  const [theme] = useTheme();
  const translate = useTranslate();
  const dataProvider = useDataProvider();
  const [discardOpen, setDiscardOpen] = useSafeSetState(false);
  const {data} = useGetList('Entry', { filter: {issuanceIdEq: props.createdIssuance.id } } );
  const listContext = useList({ perPage: 10, data });

  const onSubmit = (values) => {
    props.setPassword(values.password);
    props.handleNext();
  };
  const validate = (values) => {
    if (values.password !== window.pass) {
      return { password: translate("certos.errors.password") }
    }
  };
  const handleDiscard = async () => {
    setDiscardOpen(false);
    await props.handleDiscard();
  }

  return (<Box id="preview_container">
    <Card sx={{mb:5}}>
      <CardTitle text="certos.wizard.review_and_sign.title" />
      <CardContent>
        <Typography variant="body1">
          { translate("certos.wizard.review_and_sign.text") }
          &nbsp;
          { translate(`certos.wizard.kind_numbered.${props.createdIssuance.templateKind}`, props.createdIssuance.entriesCount) }.
        </Typography>
        <Typography variant="body1">
        { translate("certos.wizard.review_and_sign.tokens_needed", props.createdIssuance.tokensNeeded) }
        </Typography>
      </CardContent>
      <ListContextProvider value={listContext}>
        <SimpleList
          primaryText={(record) => 
            <Box component="div" sx={{cursor:"pointer", px: 2, mt: 1.5}} onClick={() => openPreview(dataProvider, record.id)}>
              <Typography variant="button" id={`preview-` + record.id}>
                { translate("certos.wizard.review_and_sign.review_label")}
                &nbsp;
                { translate(`certos.wizard.kind.${props.createdIssuance.templateKind}`)}
                &nbsp;
                #{record.id}
              </Typography>
            </Box> 
          }
          rowStyle={() => ({borderTop: "1px solid", borderColor: theme?.palette?.grey[200]})}
          linkType={false}
        />
        { props.createdIssuance.entriesCount > 10 && <Pagination rowsPerPageOptions={[]} /> }
      </ListContextProvider>
    </Card>
    <Card sx={{mb:5}}>
      <Form onSubmit={onSubmit} validate={validate} noValidate>
        <CardContent>
          <TextInput fullWidth source="password" label="certos.wizard.review_and_sign.enter_password_divider" type="password" helperText={false} />
        </CardContent>
        <CardActions>
          <Button fullWidth variant="contained" type="submit">
            { translate("certos.wizard.review_and_sign.sign_button") }
          </Button>
        </CardActions>
      </Form>
    </Card>

    <Alert variant="outlined" severity="error" sx={{ mb: 5 }} icon={false}>
      <AlertTitle>{ translate("certos.wizard.review_and_sign.discard.divider_text") }</AlertTitle>
      <Typography>{ translate("certos.wizard.review_and_sign.discard.dialog_text") }</Typography>
      <Button id="discard-button" fullWidth sx={{ my: 2 }} variant="contained" color="error" onClick={() => setDiscardOpen(true) } >
        { translate("certos.wizard.review_and_sign.discard.first_button_text") }
      </Button>
    </Alert>
    <Dialog open={discardOpen}>
      <DialogTitle>{ translate("certos.wizard.review_and_sign.discard.dialog_title") }</DialogTitle>
      <DialogActions>
        <Button variant="outlined" onClick={() => setDiscardOpen(false) }>{ translate("certos.wizard.review_and_sign.discard.continue_reviewing") }</Button>
        <Button id="confirm-discard-button" variant="contained" color="error" onClick={ handleDiscard }>{ translate("certos.wizard.review_and_sign.discard.button_discard") }</Button>
      </DialogActions>
    </Dialog>
  </Box>);
}

const Signing = ({password, ...props}) => {
  const entryCount = props.createdIssuance.entriesCount;
  const [signedCount, setSignedCount] = useSafeSetState(0);
  const notify = useNotify();
  const dataProvider = useDataProvider();

  const {createdIssuance, handleNext} = props;

  useEffect(() => {
    const handleSign = async () => {
      let entryId = null;
      let signature: string | null = null;
      let counter = 0;
      const conf = getStorage();
      const [keyPair, address] = await getKeyPair(conf.encrypted_key, password, conf.environment);

      while(true){
        try {
          const next = await dataProvider.create('SigningIterator',
            { data: { input: { issuanceId: createdIssuance.id, entryId, signature } } }
          );
          
          if(next.data.done) {
            break;
          }

          counter += 1;
          setSignedCount(counter);

          const signed_payload = getSignedPayload(keyPair, address, Buffer.from(next.data.payload, "base64"));
          entryId = next.data.id;
          signature = signed_payload.signature;

        } catch(error) {
          handleErrors(error, notify);
          return;
        }
      }

      handleNext();
    }

    handleSign();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (<Card>
    <CardTitle text="certos.wizard.signing.title" />
    <CardContent>
      <Box sx={{ display: 'flex', alignItems: 'center' }}>
        <Box sx={{ width: '100%', mr: 1 }}>
          <LinearProgress variant="determinate" value={ (signedCount / entryCount) * 100}/>
        </Box>
        <Box sx={{ minWidth: 35 }}>
          <Typography variant="body2" id="count-entries" color="text.secondary">{ signedCount }/{entryCount}</Typography>
        </Box>
      </Box>
    </CardContent>
  </Card>);
}

const NoTokensNeeded = ({hasEmails, templateKind, entries}) => {
  const translate = useTranslate();
  
  return (<Card id="no_tokens_needed_container">
    <CardTitle text="certos.wizard.done.title" color="success"/>
    <CardContent>
      <Box mb={1}>
        <Typography variant="body1">
          { translate("certos.wizard.done.begin") }
          &nbsp;
          { translate(`certos.wizard.done.kind_text.${templateKind}.base`, entries.length) }
          { hasEmails && (<>
            &nbsp;
            {translate(`certos.wizard.done.kind_text.${templateKind}.and_we`, entries.length)}
            &nbsp;
            {translate("certos.wizard.done.email")}
          </>)}
          . { translate("certos.wizard.done.see_in_dashboard") }
        </Typography>
      </Box>
    </CardContent>
    <CardActions>
      <Button fullWidth variant="contained" size="large" href="#/">
        { translate("certos.wizard.done.go_to_dashboard") }
      </Button>
    </CardActions>
  </Card>);
}

const TokensNeeded = ({hasEmails, templateKind, entries, pendingInvoiceLinkUrl}) => {
  const translate = useTranslate();
  return (<Box id="tokens_needed_container">
    <Card sx={{mb: 5}}>
      <CardTitle text="certos.wizard.done.title_need_token" color="success"/>
      <CardContent>
        <Box mb={1}>
          <Typography variant="body1">
            { translate("certos.wizard.done.begin_need_token") }
            &nbsp;
            { translate(`certos.wizard.done.kind_text.${templateKind}.base`, entries.length) }
            { hasEmails && (<>
              &nbsp;
              {translate(`certos.wizard.done.kind_text.${templateKind}.and_we`, entries.length)}
              &nbsp;
              {translate("certos.wizard.done.email")}
            </>)}
            . { translate("certos.wizard.done.see_in_dashboard") }
          </Typography>
        </Box>
      </CardContent>
      <CardActions>
        <Button fullWidth variant="contained" size="large" id="wizard-buy-tokens" href={pendingInvoiceLinkUrl} target="_blank">
          { translate("certos.wizard.done.buy_tokens") }
        </Button>
      </CardActions>
    </Card>
    <Button fullWidth variant="outlined" href="#/">
      { translate("certos.wizard.done.go_to_dashboard") }
    </Button>
  </Box>);
}

const Done = ({...props}) => {
  const {templateKind} = props.createdIssuance;
  const {data: entries} = useGetList( 'Entry', { filter: {issuanceIdEq: props.createdIssuance.id } });
  const hasEmails = _.some(entries, (i) => _.isEmpty(i.has_email_callback));

  const {data: accountState, refetch: refetchAccount } = useGetOne(
    'AccountState',
    { id: 1 },
    { enabled: false }
  );
  useGetOne(
    'Issuance',
    { id: props.createdIssuance.id },
    { 
      refetchInterval: (d) => d?.state === "signed" ? 1e10 : 1000, // Returning false does not prevent refetch, we use a large number.
      onSuccess: (d) => d?.state === "signed" && refetchAccount()
    }
  );

  const {pendingInvoiceLinkUrl} = accountState || {};

  if (!accountState) return <ConstataSkeleton title="certos.wizard.done.title" />;

  return (<Box id="done_container_loaded">{pendingInvoiceLinkUrl ?
    <TokensNeeded {...{hasEmails, templateKind, entries, pendingInvoiceLinkUrl}} /> :
    <NoTokensNeeded {...{hasEmails, templateKind, entries}} />
  }</Box>);
}

const PreviewAndSign = ({createdIssuance, handleDiscard}) => {
  const [step, setStep] = useSafeSetState(0);
  const [password, setPassword] = useSafeSetState(0);

  const props = {
    handleNext: async () => {
      setStep((prev) => prev + 1);
    },
    createdIssuance,
  };

  return (
    <Box my={2}>
      { step === 0 && <Preview setPassword={setPassword} handleDiscard={handleDiscard} { ...props } /> }
      { step === 1 && <Signing password={password} { ...props } /> }
      { step === 2 && <Done { ...props } /> }
    </Box>
  );
}

export default function Wizard() {
  let { id } = useParams();
  const dataProvider = useDataProvider();
  const [createdIssuance, setCreatedIssuance] = useSafeSetState(null);
  const [canSendEmail, setCanSendEmail] = useSafeSetState(null);
  const [loading, setLoading] = useSafeSetState(true);
  const notify = useNotify();
  const navigate = useNavigate();
  const checkAuth = useCheckAuth();

  useEffect(() => {
    async function init(){
      try { await checkAuth(); } catch { return; }

      const {data} = await dataProvider.getOne('EndorsementManifest', { id: 1 })
      setCanSendEmail(data.canSendEmail)

      if (_.isEmpty(id)) {
        setLoading(false)
        return;
      }

      let value = await dataProvider.getOne('Issuance', { id });
      if (value.data) {
        setCreatedIssuance(value.data);
      } else {
        handleErrors("Not found", notify);
      }
      setLoading(false)
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  
  const handleDiscard = async () => {
    await dataProvider.update('Issuance', { id: createdIssuance.id, data: {}, previousData: {} });
    setCreatedIssuance(null);
    navigate("/wizard")
  }

  return (<Container maxWidth="md">
    { loading && <LinearProgress sx={{mt: 3}}/> }
    { !loading && createdIssuance &&
      <PreviewAndSign createdIssuance={createdIssuance} handleDiscard={handleDiscard}/> 
    }
    { !loading && !createdIssuance && 
      <CreateIssuance canSendEmail={canSendEmail} setCreatedIssuance={setCreatedIssuance}/> 
    }
  </Container>);
}

const DoneStep = ({children}: { children: JSX.Element } ) => 
  <Card sx={{my: 2}}>
    <CardContent sx={{ py: "1em !important" }}>
      { children }
    </CardContent>
  </Card>;
