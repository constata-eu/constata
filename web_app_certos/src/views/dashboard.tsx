import { Box, Button, Typography, Container } from '@mui/material';
import { useEffect } from "react";
import { useTranslate, useCheckAuth } from 'ra-core';
import AccountStateSection from '../components/account_state_section';
import Balance from '../components/balance_section';
import RequestsSection from '../components/requests_section';
import EndorsementsSection from '../components/endorsements_section';
import EmailAddress from '../components/email_address_section';
import { Head1 } from '../theme';

export default function Dashboard() {
  const translate = useTranslate();
  const checkAuth = useCheckAuth();

  useEffect(() => {
    async function checkVerification() {
      try {
        await checkAuth();
      } catch {
        return;
      }
    }
    checkVerification();
  }, [checkAuth]);

  return (<Container maxWidth="md" id="constata_dashboard">
    <AccountStateSection />
    <Box mb={3}>
      <Head1 sx={{ mb:2 }}> { translate("certos.dashboard.issue.title") } </Head1>
      <Typography>
        { translate("certos.dashboard.issue.text") }
      </Typography>
    </Box>
    <Button fullWidth size="large" variant="contained" href="#/wizard" sx={{ fontSize: 20, mb: 5 }}>
      { translate("certos.dashboard.issue.button") }
    </Button>
    <EndorsementsSection/>
    <EmailAddress/>
    <RequestsSection/>
    <Balance />
  </Container>)
};
