import { Card, CardContent, Typography, CardActions, Button } from '@mui/material';
import { useList, ListContextProvider, SimpleList, useTheme } from 'react-admin';
import { useEffect } from "react";
import { useTranslate, useDataProvider, useSafeSetState } from 'ra-core';
import CardTitle from '../components/card_title';
import _ from 'lodash';

export default function Request() {
  const [theme] = useTheme();
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const [unsignedRequests, setUnsignedRequests] = useSafeSetState([]);
  const [signedRequests, setSignedRequests] = useSafeSetState([]);
  const [recentRequests, setRecentRequests] = useSafeSetState([]);
  const [unsignedRequestsUnseen, setUnsignedRequestsUnseen] = useSafeSetState(0);
  const [signedRequestsUnseen, setSignedRequestsUnseen] = useSafeSetState(0);
  const [recentRequestsUnseen, setRecentRequestsUnseen] = useSafeSetState(0);
  const unsignedListContext = useList({ data: unsignedRequests });
  const signedListContext = useList({ data: signedRequests });
  const recentListContext = useList({ data: recentRequests });

  useEffect(() => {
    async function getRequests(stateEq: string) {
      const limit = 4;
      let {data} = await dataProvider.getList('Request', {
        filter: { stateEq },
        sort: { field: 'createdAt', order: 'DESC' },
        pagination: { page: 1, perPage: 51 + limit },
      });
      const unseenRequests = data.length > limit ? data.length - limit : 0;
      const slice = _.slice(data, 0, limit);
      return [slice, unseenRequests];
    }
    async function load(){
      const [unsigned, unsignedUnseen] = await getRequests("created");
      setUnsignedRequests(unsigned);
      setUnsignedRequestsUnseen(unsignedUnseen);
      const [signed, signedUnseen] = await getRequests("signed");
      setSignedRequests(signed);
      setSignedRequestsUnseen(signedUnseen);
      const [recent, recentUnseen] = await getRequests("completed");
      setRecentRequests(recent);
      setRecentRequestsUnseen(recentUnseen);
    }

    load();
    const interval = setInterval(load, 10000);
    return function cleanup() { clearInterval(interval); };
  }, [dataProvider, setRecentRequests, setSignedRequests, setUnsignedRequests, setUnsignedRequestsUnseen, setSignedRequestsUnseen, setRecentRequestsUnseen]);

  return (<>
    { unsignedRequests.length > 0 &&
      <Card sx={{ mb: 5 }} id="request-section-unsigned">
        <CardTitle text="certos.dashboard.pending_signatures.title" color="warning" />
        <CardContent>
          <Typography variant="body1">{ translate('certos.dashboard.pending_signatures.text') }</Typography>
        </CardContent>
        <ListContextProvider value={unsignedListContext}>
          <SimpleList
            primaryText={(record) => `${record.name} #${record.id}`}
            linkType={(_, id) => `/wizard/${id}`}
            secondaryText={ (record) => record.templateName }
            tertiaryText={ (record) => translate(`certos.wizard.kind_numbered.${record.templateKind}`, record.entries.length)}
            rowStyle={() => ({borderTop: "1px solid", borderColor: theme?.palette?.grey[200]})}
          />
        </ListContextProvider>
        { unsignedRequestsUnseen > 0 && <ActionSeeMore unseen={unsignedRequestsUnseen} state="created" /> }
      </Card>
    }
    { signedRequests.length > 0 &&
      <Card sx={{ mb: 5 }} id="request-section-signed">
        <CardTitle text="certos.dashboard.signed.title" />
        <CardContent>
          <Typography variant="body1">{ translate('certos.dashboard.signed.text') }</Typography>
        </CardContent>
        <ListContextProvider value={signedListContext}>
          <SimpleList
            primaryText={(record) => `${record.name} #${record.id}`}
            linkType={(_, id) => `/Request/${id}/show`}
            secondaryText={ (record) => record.templateName }
            tertiaryText={ (record) => translate(`certos.wizard.kind_numbered.${record.templateKind}`, record.entries.length)}
            rowStyle={() => ({borderTop: "1px solid", borderColor: theme?.palette?.grey[200]})}
          />
        </ListContextProvider>
        { signedRequestsUnseen > 0 && <ActionSeeMore unseen={signedRequestsUnseen} state="signed" /> }
      </Card>
    }
    { recentRequests.length > 0 &&
      <Card sx={{ mb: 5 }} id="request-section-recent">
        <CardTitle text="certos.dashboard.recent.title" />
        <CardContent>
          <Typography variant="body1">{ translate('certos.dashboard.recent.text') }</Typography>
        </CardContent>
        <ListContextProvider value={recentListContext}>
          <SimpleList
            primaryText={(record) => `${record.name} #${record.id}`}
            linkType={(_, id) => `/Request/${id}/show`}
            secondaryText={ (record) => record.templateName }
            tertiaryText={ (record) => translate(`certos.wizard.kind_numbered.${record.templateKind}`, record.entries.length)}
          />
        </ListContextProvider>
        { recentRequestsUnseen > 0 && <ActionSeeMore unseen={recentRequestsUnseen} state="completed" /> }
      </Card>
    }
  </>)
};

const ActionSeeMore = ({unseen, state}) => {
  const translate = useTranslate();

  return <CardActions>
    <Button
      fullWidth
      variant="outlined"
      href={`#/Request?displayedFilters=%7B%7D&filter=%7B"stateEq"%3A"${state}"%7D&order=DESC&page=1&perPage=20&sort=id`}
      color="highlight"
    >
      { translate("certos.dashboard.unseen.see") }
      { unseen > 50 ? "+50" : unseen }
      { translate("certos.dashboard.unseen.more") }
    </Button>
  </CardActions>
}