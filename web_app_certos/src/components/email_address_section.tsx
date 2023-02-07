import { useEffect } from 'react';
import {
  Typography, Box, Button, Card, CardContent, Grid, Skeleton
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import _ from 'lodash';
import CardTitle from '../components/card_title';
import {
  Form,
  TextInput,
  BooleanInput,
  required,
  useDataProvider,
  useTranslate,
  useSafeSetState,
  RecordContextProvider,
} from 'react-admin';
import { EmailAddressSection } from './types';

export default function EmailAddress() {
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const [editing, setEditing] = useSafeSetState(false);
  const [record, setRecord] = useSafeSetState<EmailAddressSection>();

  useEffect(() => {
    async function init(){
      const current = (await dataProvider.getList('EmailAddress', {
        sort: { field: 'id', order: 'DESC' },
        pagination: { page: 1, perPage: 1 },
        filter: {},
      })).data[0];
      
      setRecord(_.pick(current, 'address', 'keepPrivate', 'verifiedAt'));
    }
    init();
  }, [dataProvider, setRecord]);

  const onSubmit = async (values) => {
    let {data} = await dataProvider.create('EmailAddress', { data: { input: _.pick(values, 'address', 'keepPrivate') }});
    setRecord(data);
    setEditing(false);
  }

  if (!record) return <Card sx={{ mb: 5 }}>
    <CardTitle text="certos.dashboard.email.title" />
    <CardContent>
      <Skeleton/>
      <Skeleton/>
      <Skeleton width="30%"/>
    </CardContent>
  </Card>;

  const Edit = () => 
    <RecordContextProvider value={record}>
      { _.isEmpty(record) && <Box my={1}><Typography>{ translate("certos.dashboard.email.no_email_yet") } </Typography></Box> }
      <Form onSubmit={onSubmit} noValidate>
        <Grid container spacing={1} alignItems="center" >
          <Grid item xs={12} md={6}>
            <TextInput
              fullWidth
              size="small"
              source="address"
              label={ `certos.kyc_request.fields.email` }
              helperText={ `certos.kyc_request.helper_text.email` }
              variant="filled"
              validate={required()}
            />
          </Grid>
          <Grid item xs={12} md={6}>
            <BooleanInput label="certos.kyc_request.fields.keepPrivate" source="keepPrivate" />
          </Grid>
          <Grid item xs={12}>
            <Button fullWidth type="submit" variant="contained">{ translate("certos.dashboard.email.save") }</Button>
          </Grid>
        </Grid>
      </Form>
    </RecordContextProvider>;

  const Show = () =>
    <Grid container spacing={1} >
      <Grid item xs={9}>
        <Typography variant="body1" sx={{ wordBreak: "break-all" }}>
          { record.address }
        </Typography>
        <Typography variant="body2">
          { record.keepPrivate ?  translate("certos.kyc_request.helper_text.emailWontShow") : translate("certos.kyc_request.helper_text.emailWillShow") }
        </Typography>
        <Typography variant="body2">
          { record.verifiedAt ? translate("certos.dashboard.email.verified") : translate("certos.dashboard.email.not_verified") }
        </Typography>
      </Grid>
      <Grid item xs={3}>
        <Box display="flex" justifyContent="flex-end">
          <Button variant="outlined" onClick={ () => setEditing(true) } ><EditIcon/></Button>
        </Box>
      </Grid>
  </Grid>

  return (<Card sx={{ my: 5 }} id="section-email-address">
    <CardTitle text="certos.dashboard.email.title" />
    <CardContent>
      { editing || _.isEmpty(record) ? <Edit/> : <Show/> }
    </CardContent>
  </Card>);
}
