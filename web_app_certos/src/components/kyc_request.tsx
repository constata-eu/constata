import { useEffect } from 'react';
import {
    Typography, Box, Button, Card, CardContent, CardActions, Container, Link, Grid
} from '@mui/material';
import { convertBase64 } from '../components/utils'
import { useNavigate } from 'react-router-dom';
import _ from 'lodash';
import CardTitle from '../components/card_title';
import ConstataSkeleton from './skeleton';
import {
  Form,
  FileInput,
  FileField,
  TextInput,
  AutocompleteInput,
  DateInput,
  BooleanInput,
  email,
  required,
  useDataProvider,
  useTranslate,
  useSafeSetState,
  useCheckAuth,
  RecordContextProvider,
} from 'react-admin';
import countrily from 'countrily';

const KycTextField = ({ source, ...props}) => 
  <TextInput
    {...props }
    fullWidth
    size="small"
    source={source}
    required={props.required}
    label={ `certos.kyc_request.fields.${source}` }
    helperText={ `certos.kyc_request.helper_text.${source}` }
    variant="filled"
  />

export default function KycRequest() {
  const translate = useTranslate();
  const checkAuth = useCheckAuth();
  const navigate = useNavigate();
  const [countries, setCountries] = useSafeSetState([]);
  const [emailWillShow, setEmailWillShow] = useSafeSetState(true);
  const [initialValue, setInitialValue] = useSafeSetState({});
  const [loading, setLoading] = useSafeSetState(true);
  const dataProvider = useDataProvider();

  useEffect(() => {
    async function init(){
      try { await checkAuth(); } catch { return; }
      const previous = (await dataProvider.getList('KycRequest', {
        sort: { field: 'createdAt', order: 'DESC' },
        pagination: { page: 1, perPage: 1 },
        filter: {}
      })).data[0];
      
      if (previous?.state === "pending") {
        navigate("/")
      }

      const manifest = (await dataProvider.getOne('EndorsementManifest', { id: 1 }))
      const kyc = manifest.data.kyc;

      let initial = (kyc?.updatedAt > previous?.createdAt) ? kyc : previous;
      initial = _.pick(initial,
        "name",
        "lastName",
        "idNumber",
        "idType",
        "birthdate",
        "nationality",
        "country",
        "jobTitle",
        "legalEntityName",
        "legalEntityCountry",
        "legalEntityRegistration",
        "legalEntityTaxId",
      );

      const email = manifest.data.email;
      if (email) {
        initial.email = email.address;
        initial.keepPrivate = email.keepPrivate;
      }

      setInitialValue(initial);
      
      setCountries(countrily.all()
        .filter((c) => !_.isEmpty(c.translations))
        .map((c) => ({ name: c.translations['es'], id: c.ISO[3]}) )
        .sort((c) => c.label)
      )
      setLoading(false);
    }
    init();
  }, [checkAuth, dataProvider, setCountries, setInitialValue, navigate, setLoading]);

  const onSubmit = async (values) => {
    let input = { evidence: [], ..._.omit(values, 'evidence', 'birthdate') };
    input.birthdate = values.birthdate && new Date(values.birthdate).toISOString();
    for (const e of (values.evidence || [])) {
      input.evidence.push({
        filename: e.title,
        payload: (await convertBase64(e.rawFile)).split('base64,')[1]
      })
    }
    await dataProvider.create('KycRequest', { data: { input }});
    navigate("/");
  }

  if (loading) {
    return <ConstataSkeleton title={"certos.kyc_request.title"} />;
  }

  return ( <Container maxWidth="md">
    <RecordContextProvider value={initialValue}>
      <Form onSubmit={onSubmit} noValidate>
        <Card sx={{my:2}}>
          <CardTitle text="certos.kyc_request.title" />
          <CardContent>
            { translate("certos.kyc_request.text") }
            &nbsp;
            <Link href="https://api.constata.eu/terms_acceptance/show/#privacy_policies">
              { translate("certos.kyc_request.privacy_policy") }
            </Link>
          <CardContent>
          </CardContent>
            <Grid container spacing={1} alignItems="center" >
              <Grid item xs={12} md={6}>
                <KycTextField source="email" validate={[required(), email()]} />
              </Grid>
              <Grid item xs={12} md={6}>
                <BooleanInput 
                  label="certos.kyc_request.fields.keepPrivate"
                  source="keepPrivate"
                  onChange={ (e) => setEmailWillShow(!e.target.checked) }
                  helperText={ emailWillShow ? 
                    translate("certos.kyc_request.helper_text.emailWillShow") :
                    translate("certos.kyc_request.helper_text.emailWontShow")
                  }/>
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="name" validate={required()} />
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="lastName" validate={required()}/>
              </Grid>
              <Grid item xs={12} md={6}>
                <AutocompleteInput
                  source="nationality"
                  label="certos.kyc_request.fields.nationality"
                  helperText="certos.kyc_request.helper_text.nationality"
                  choices={ countries } />
              </Grid>
              <Grid item xs={12} md={6}>
                <DateInput fullWidth source="birthdate"
                  label="certos.kyc_request.fields.birthdate"
                  helperText="certos.kyc_request.helper_text.birthdate"
                  variant="filled"
                 />
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="idNumber" />
              </Grid>
              <Grid item xs={12} md={6}>
                <AutocompleteInput
                  source="country"
                  label="certos.kyc_request.fields.country"
                  helperText="certos.kyc_request.helper_text.country"
                  choices={ countries } />
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="jobTitle" />
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="legalEntityName" />
              </Grid>
              <Grid item xs={12} md={6}>
                <AutocompleteInput
                  source="legalEntityCountry"
                  label="certos.kyc_request.fields.legalEntityCountry"
                  helperText="certos.kyc_request.helper_text.legalEntityCountry"
                  choices={ countries } />
              </Grid>
              <Grid item xs={12} md={6}>
                <KycTextField source="legalEntityTaxId" />
              </Grid>
              <Grid item xs={12}>
                <Box>
                  <FileInput
                    sx={{
                      '& .RaFileInput-dropZone': { background: 'inherit', p: 0 },
                    }}
                    source="evidence"
                    label={false}
                    placeholder={ 
                      <>
                      <Typography variant="body2" sx={{ textAlign: 'left', margin: "1em 0" }}>
                        { translate("certos.kyc_request.evidence_text.start") }
                      </Typography>
                      <Typography variant="body2" sx={{ textAlign: 'left', margin: "1em 0" }}>
                        { translate("certos.kyc_request.evidence_text.end") }
                      </Typography>
                      <Button fullWidth variant="outlined">{ translate("certos.kyc_request.helper_text.evidence") }</Button>
                      </>
                    }
                    multiple
                    maxSize={20000000}
                  >
                    <FileField source="src" title="title" />
                  </FileInput>
                </Box>
              </Grid>
            </Grid>
          </CardContent>
          <CardActions>
            <Button fullWidth type="submit" variant="contained">{ translate("certos.kyc_request.submit") }</Button>
          </CardActions>
        </Card>
      </Form>
    </RecordContextProvider>
  </Container>);
}

