import { Card, CardContent, Typography, CardActions, Button } from '@mui/material';
import { useList, ListContextProvider, SimpleList, useTheme } from 'react-admin';
import { useEffect } from "react";
import { useTranslate, useDataProvider, useSafeSetState } from 'ra-core';
import CardTitle from './card_title';
import _ from 'lodash';

export default function Issuance() {
  const [theme] = useTheme();
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const [unsignedIssuances, setUnsignedIssuances] = useSafeSetState([]);
  const [signedIssuances, setSignedIssuances] = useSafeSetState([]);
  const [recentIssuances, setRecentIssuances] = useSafeSetState([]);
  const [unsignedIssuancesUnseen, setUnsignedIssuancesUnseen] = useSafeSetState(0);
  const [signedIssuancesUnseen, setSignedIssuancesUnseen] = useSafeSetState(0);
  const [recentIssuancesUnseen, setRecentIssuancesUnseen] = useSafeSetState(0);
  const unsignedListContext = useList({ data: unsignedIssuances });
  const signedListContext = useList({ data: signedIssuances });
  const recentListContext = useList({ data: recentIssuances });

  useEffect(() => {
    async function getIssuances(stateEq: string) {
      const limit = 4;
      let {data} = await dataProvider.getList('Issuance', {
        filter: { stateEq },
        sort: { field: 'createdAt', order: 'DESC' },
        pagination: { page: 1, perPage: 51 + limit },
      });
      const unseenIssuances = data.length > limit ? data.length - limit : 0;
      const slice = _.slice(data, 0, limit);
      return [slice, unseenIssuances];
    }
    async function load(){
      const [unsigned, unsignedUnseen] = await getIssuances("created");
      setUnsignedIssuances(unsigned);
      setUnsignedIssuancesUnseen(unsignedUnseen);
      const [signed, signedUnseen] = await getIssuances("signed");
      setSignedIssuances(signed);
      setSignedIssuancesUnseen(signedUnseen);
      const [recent, recentUnseen] = await getIssuances("completed");
      setRecentIssuances(recent);
      setRecentIssuancesUnseen(recentUnseen);
    }

    load();
    const interval = setInterval(load, 10000);
    return function cleanup() { clearInterval(interval); };
  }, [dataProvider, setRecentIssuances, setSignedIssuances, setUnsignedIssuances, setUnsignedIssuancesUnseen, setSignedIssuancesUnseen, setRecentIssuancesUnseen]);

  return (<>
    { unsignedIssuances.length > 0 &&
      <Card sx={{ mb: 5 }} id="issuance-section-unsigned">
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
        { unsignedIssuancesUnseen > 0 && <ActionSeeMore unseen={unsignedIssuancesUnseen} state="created" /> }
      </Card>
    }
    { signedIssuances.length > 0 &&
      <Card sx={{ mb: 5 }} id="issuance-section-signed">
        <CardTitle text="certos.dashboard.signed.title" />
        <CardContent>
          <Typography variant="body1">{ translate('certos.dashboard.signed.text') }</Typography>
        </CardContent>
        <ListContextProvider value={signedListContext}>
          <SimpleList
            primaryText={(record) => `${record.name} #${record.id}`}
            linkType={(_, id) => `/Issuance/${id}/show`}
            secondaryText={ (record) => record.templateName }
            tertiaryText={ (record) => translate(`certos.wizard.kind_numbered.${record.templateKind}`, record.entries.length)}
            rowStyle={() => ({borderTop: "1px solid", borderColor: theme?.palette?.grey[200]})}
          />
        </ListContextProvider>
        { signedIssuancesUnseen > 0 && <ActionSeeMore unseen={signedIssuancesUnseen} state="signed" /> }
      </Card>
    }
    { recentIssuances.length > 0 &&
      <Card sx={{ mb: 5 }} id="issuance-section-recent">
        <CardTitle text="certos.dashboard.recent.title" />
        <CardContent>
          <Typography variant="body1">{ translate('certos.dashboard.recent.text') }</Typography>
        </CardContent>
        <ListContextProvider value={recentListContext}>
          <SimpleList
            primaryText={(record) => `${record.name} #${record.id}`}
            linkType={(_, id) => `/Issuance/${id}/show`}
            secondaryText={ (record) => record.templateName }
            tertiaryText={ (record) => translate(`certos.wizard.kind_numbered.${record.templateKind}`, record.entries.length)}
          />
        </ListContextProvider>
        { recentIssuancesUnseen > 0 && <ActionSeeMore unseen={recentIssuancesUnseen} state="completed" /> }
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
      href={`#/Issuance?displayedFilters=%7B%7D&filter=%7B"stateEq"%3A"${state}"%7D&order=DESC&page=1&perPage=20&sort=id`}
      color="highlight"
    >
      { translate("certos.dashboard.unseen.see") }
      { unseen > 50 ? "+50" : unseen }
      { translate("certos.dashboard.unseen.more") }
    </Button>
  </CardActions>
}