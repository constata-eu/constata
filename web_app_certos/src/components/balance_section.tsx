import { Card, CardContent, Typography, Box } from '@mui/material';
import { useTranslate } from 'ra-core';
import { useGetOne } from 'react-admin';
import CardTitle from './card_title';
import ConstataSkeleton from './skeleton';

export default function Balance() {
  const translate = useTranslate();
  const { data: accountState, isLoading } = useGetOne(
    'AccountState',
    { id: 1 },
    { refetchInterval: 10000, refetchIntervalInBackground: true }
  );

  if (isLoading) return <ConstataSkeleton title={"certos.balance.title"} />;

  return (<Card sx={{ mb: 5 }}>
    <CardTitle text={"certos.balance.title"} color="warning" />
    <CardContent>
      <Box mb={1}><Typography> { translate("certos.balance.text") } </Typography></Box>
      <Box mb={1}>
        <Typography>
          {accountState?.tokenBalance <= 0 ?
            translate("certos.balance.no_tokens") :
            translate("certos.balance.balance_amount", accountState?.tokenBalance)
          }
        </Typography>
      </Box>
      {accountState?.monthlyGiftRemainder > 0 &&
        <Box mb={1}>
          <Typography>
            { translate("certos.balance.monthly_gift_remainder", accountState?.monthlyGiftRemainder) }
          </Typography>
        </Box>
      }
    </CardContent>
  </Card>)
};
