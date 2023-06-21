import { Alert, AlertTitle, Button, Typography } from '@mui/material';
import { useTranslate } from 'ra-core';
import { useGetOne } from 'react-admin';


const PendingTycSection = (pendingTycUrl) => {
  const translate = useTranslate();

  return (<Alert variant="outlined" severity="error" sx={{ mb: 5 }} icon={false} id="section-account-state">
    <AlertTitle>{ translate("certos.account_state.pending_tyc.title") }</AlertTitle>
    <Typography>{ translate('certos.account_state.pending_tyc.text') }</Typography>
    <Button fullWidth sx={{ my: 2}} variant="contained" color="error" href={pendingTycUrl} target="_blank">
      { translate("certos.account_state.pending_tyc.button") }
    </Button>
  </Alert>);
}

const PendingInvoiceSection = ({accountState}) => {
  const translate = useTranslate();
  
  return (<Alert variant="outlined" severity="error" sx={{ mb: 5 }} icon={false}>
    <AlertTitle>{ translate("certos.account_state.pending_invoice.title") }</AlertTitle>
    <Typography>
      { translate('certos.account_state.pending_invoice.text', accountState?.parkedCount) }
      &nbsp;
      { translate('certos.account_state.pending_invoice.text_2', accountState?.missing) }
      &nbsp;
      { translate('certos.account_state.pending_invoice.text_3', accountState?.missing * (accountState?.pricePerToken / 100)) }
    </Typography>
    <Button fullWidth sx={{ my: 2}} variant="contained" color="error"
      id="dashboard-buy-tokens"
      href={accountState?.pendingInvoiceLinkUrl}
      target="_blank"
    >
      { translate("certos.account_state.pending_invoice.button") }
    </Button>
  </Alert>);
}

export default function AccountState() {
  const {data: accountState} = useGetOne(
    'AccountState',
    { id: 1 },
    { refetchInterval: 10000 }
  );
  const {pendingTycUrl} = accountState || {};

  return (<>
    { pendingTycUrl && <PendingTycSection {...{pendingTycUrl}} /> }
    { accountState?.pendingInvoiceLinkUrl && <PendingInvoiceSection {...{accountState}} /> }
  </>)
};
