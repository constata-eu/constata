import { Box, Button, Typography, Container, Skeleton } from '@mui/material';
import { useEffect } from "react";
import { useTranslate, useCheckAuth, useSafeSetState } from 'ra-core';
import AccountStateSection from '../components/account_state_section';
import Balance from '../components/balance_section';
import IssuancesSection from '../components/issuances_section';
import EndorsementsSection from '../components/endorsements_section';
import EmailAddress from '../components/email_address_section';
import OtherActions from '../components/other_actions';
import Dashboard from './dashboard';
import { VcPromptDashboard } from "./vc_prompt";
import { Head1 } from '../theme';

export default function MultiAppDashboard() {
  const translate = useTranslate();
  const checkAuth = useCheckAuth();
  const [ready, setReady] = useSafeSetState(false);

  useEffect(() => {
    async function checkVerification() {
      try {
        await checkAuth();
      } catch {
        return;
      }
      setReady(true);
    }
    checkVerification();
  }, [checkAuth]);

  if (!ready) return <Container maxWidth="md" id="constata_dashboard_loading">
    <Skeleton/>
    <Skeleton/>
    <Skeleton/>
  </Container>;

  return (<Container maxWidth="md" id="constata_dashboard">
    <Dashboard/>
    <VcPromptDashboard/>
  </Container>)
};
