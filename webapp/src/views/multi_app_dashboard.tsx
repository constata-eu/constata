import { Box, Button, Typography, Container, Skeleton } from '@mui/material';
import { useEffect } from "react";
import { useTranslate, useCheckAuth, useSafeSetState, useGetOne } from 'ra-core';
import AccountStateSection from '../components/account_state_section';
import Balance from '../components/balance_section';
import IssuancesSection from '../components/issuances_section';
import EndorsementsSection from '../components/endorsements_section';
import EmailAddress from '../components/email_address_section';
import OtherActions from '../components/other_actions';
import Dashboard from './dashboard';
import { Head1 } from '../theme';

export default function MultiAppDashboard() {
  const {isLoading, data: accountState} = useGetOne( 'AccountState', { id: 1 });

  if (isLoading) return <Container maxWidth="md" id="constata_dashboard_loading">
    <Skeleton/>
    <Skeleton/>
    <Skeleton/>
  </Container>;

  return <Dashboard/>;
};
