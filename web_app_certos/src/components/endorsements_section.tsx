import {
  Box, Button, Card, CardContent, CardActions, Typography, Chip, Alert,
} from '@mui/material';
import { useTranslate } from 'ra-core';
import { useGetOne, useGetList } from 'react-admin';
import CardTitle from '../components/card_title';
import ConstataSkeleton from './skeleton';


export default function Endorsements() {
  const translate = useTranslate();
  const {data: manifest, isLoading: isLoadingEndorsement} = useGetOne( 'EndorsementManifest', { id: 1 });
  const {total: requestsTotal, isLoading: isLoadingKycRequests} = useGetList(
    'KycRequest',
    { sort: { field: 'id', order: 'DESC' },
      pagination: { page: 1, perPage: 1 },
      filter: { stateEq: "pending" },
    }
  );
  
  if (isLoadingEndorsement && isLoadingKycRequests) {
     return <ConstataSkeleton title={"certos.dashboard.endorsement.not_yet.title"} />;
  }

  const endorsement = manifest.text;
  const hasKycRequest = requestsTotal > 0;

  return (<>
    { !endorsement && <Card sx={{ mb: 5 }} id="section-endorsement-not-yet">
        <CardTitle text="certos.dashboard.endorsement.not_yet.title" />
        <CardContent>
          <Box mb={1}><Typography variant="body1">{ translate('certos.dashboard.endorsement.not_yet.lead') }</Typography></Box>
          <Box mb={1}><Typography variant="body1">{ translate('certos.dashboard.endorsement.not_yet.description') }</Typography></Box>
          <Box mb={2}><Typography variant="body1">{ translate('certos.dashboard.endorsement.not_yet.cost') }</Typography></Box>
          <Box mb={1}>
            <Typography variant="body2" component="span">
              <Chip label={ translate("certos.dashboard.endorsement.not_yet.verification") } variant="filled" color="highlight" size="small"/> =
              500 EUR
            </Typography>
          </Box>
          <Box mb={1}>
            <Typography variant="body2" component="span">
              <Chip label={ translate("certos.dashboard.endorsement.not_yet.verification") } variant="filled" color="highlight" size="small" />
              &nbsp;+&nbsp;
              <Chip label={ translate("certos.dashboard.endorsement.not_yet.custom_template") } variant="filled" color="highlight" size="small" /> =
              1200 EUR
            </Typography>
          </Box>
        </CardContent>
        { !hasKycRequest && <CardActions>
            <Button fullWidth variant="outlined" href="#/request_verification" color="highlight">
              { translate("certos.dashboard.endorsement.not_yet.call_to_action") }
            </Button>
          </CardActions>
        }
        { hasKycRequest &&
          <Alert severity="info">
            { translate("certos.dashboard.endorsement.not_yet.received") }
          </Alert>
        }
      </Card>
    }
    { endorsement && <Card sx={{ mb: 2 }} id="section-endorsement-existing">
        <CardTitle text="certos.dashboard.endorsement.existing.title" />
        <CardContent>
          <Box mb={1}>
            <Typography dangerouslySetInnerHTML={{ __html: endorsement}}></Typography>
          </Box>
        </CardContent>
        { !hasKycRequest && <CardActions>
            <Button fullWidth href="#/request_verification" variant="outlined">
              { translate("certos.dashboard.endorsement.existing.call_to_action") }
            </Button>
          </CardActions>
        }
        { hasKycRequest &&
          <Alert severity="info">
            { translate("certos.dashboard.endorsement.existing.received") }
          </Alert>
        }
      </Card>
    }
  </>)
};
