import { useEffect } from 'react';
import { Card, CardContent, Typography, Container, Box, LinearProgress, Button } from '@mui/material';
import { useTranslate, useDataProvider, Form, NumberInput, required, minValue, maxValue } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { useParams, useNavigate } from 'react-router-dom';
import { setAccessToken, clearAccessToken } from '../components/auth_provider';
import CardTitle from '../components/card_title';
import ConstataSkeleton from '../components/skeleton';
import { NoLoggedInLayout } from './layout';


interface InvoiceLinkData {
  id?: number,
  minimumSuggested?: number,
  pricePerToken?: number,
}

const InvoiceLink = () => {
  const dataProvider = useDataProvider();
  const { access_token } = useParams();
  const [state, setState] = useSafeSetState<string>("loading");
  const [invoiceLinkData, setInvoiceLinkData] = useSafeSetState<InvoiceLinkData>({});
  const [tokens, setTokens] = useSafeSetState<number>(0);
  const maxTokenPurchase: number = 100000000;
  const translate = useTranslate();

  useEffect(() => {
    const init = async () => {
      setAccessToken(access_token);
      try {
        const {data} = await dataProvider.getOne('InvoiceLink', { id: 1 });
        setInvoiceLinkData(data);
        setTokens(data.minimumSuggested);
        setState("initial")
      } catch (e) { 
        setState(e.status === 401 ? "warning" : "error")
      }
      clearAccessToken();
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const Error = <Card sx={{ mb: 5 }}>
      <CardTitle text={translate("certos.invoice_link.error.title")} />
      <CardContent>
        <Typography>{translate("certos.invoice_link.error.text")}</Typography>
      </CardContent>
    </Card>;

  const Expired = <Card sx={{ mb: 5 }}>
      <CardTitle text={translate("certos.invoice_link.expired.title")}/>
      <CardContent>
        <Typography> {translate("certos.invoice_link.expired.text")} </Typography>
      </CardContent>
    </Card>;

  const submit = async (values, form) => {
    setState("creating_invoice");
    setAccessToken(access_token);
    try {
      const {data} = await dataProvider.create("InvoiceLink", { data: {
        input: {paymentMethod: form.nativeEvent.submitter.value, tokens: values.amount}
      }});
      if (!data.invoiceUrl) { throw 400; }
      window.location.replace(data.invoiceUrl);
    } catch(e) {
      setState("error");
    }
    clearAccessToken();
  }

  const ToPurchase = <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.invoice_link.purchase.title")} id="buy-tokens-title"/>
    <Form onSubmit={submit}>
      <CardContent>
      
        <Typography sx={{ mb: 1 }}>{translate("certos.invoice_link.purchase.text_1")}</Typography>
        <Typography sx={{ mb: 1 }}>{translate("certos.invoice_link.purchase.text_2")}</Typography>
        <Typography sx={{ mb: 1 }}>{translate("certos.invoice_link.purchase.text_3", invoiceLinkData.pricePerToken / 100)}</Typography>

        <NumberInput
          defaultValue={tokens}
          min={invoiceLinkData.minimumSuggested}
          max={maxTokenPurchase}
          fullWidth
          source={"amount"}
          validate={[required(), minValue(invoiceLinkData.minimumSuggested), maxValue(maxTokenPurchase)]}
          autoFocus
          onChange={e => {
            setTokens(e.target.value)
          }}
          label={translate("certos.invoice_link.purchase.button_label")}
          helperText={translate("certos.invoice_link.purchase.button_help", invoiceLinkData.minimumSuggested)}
          variant="filled"
        />

        <Typography sx={{fontWeight: 600, my: 1}}>
          {translate("certos.invoice_link.purchase.total", tokens * (invoiceLinkData.pricePerToken / 100)) }
        </Typography>

        <Box mt={2} display="flex" alignItems="center" justifyContent="space-evenly">
          <Button
            sx={{m: 2}}
            type="submit"
            fullWidth
            size="large"
            variant="contained"
            value="CreditCard"
            id="pay-with-credit-card"
          >
            {translate("certos.invoice_link.purchase.credit_card")} 
          </Button>
          <Button
            sx={{m: 2}}
            type="submit"
            fullWidth
            size="large"
            variant="contained"
            value="Bitcoin"
            id="pay-with-bitcoin"
          >
            {translate("certos.invoice_link.purchase.bitcoin")}
          </Button>
        </Box>
      </CardContent>
    </Form>
  </Card>

  const CreatingInvoice = <Card sx={{ mb: 5 }}>
    <CardTitle text={translate("certos.invoice_link.purchasing.title")} />
      <CardContent >
        <LinearProgress />
      </CardContent>
    </Card>

  return (<Container maxWidth="md" id="invoice-link-buy">
    { state === "loading" && <ConstataSkeleton title="Cargando" lines={9} /> }
    { state === "warning" && Expired }
    { state === "error" && Error }
    { state === "initial" && ToPurchase }
    { state === "creating_invoice" && CreatingInvoice }
  </Container>
)}

const InvoiceLinkSuccess = () => {
  const translate = useTranslate();
  const navigate = useNavigate();
  
  return <NoLoggedInLayout>
    <Card sx={{ mb: 5 }} id="invoice-link-success">
      <CardTitle text={translate("certos.invoice_link.success.title")} />
      <CardContent>
        <Typography> {translate("certos.invoice_link.success.text")} </Typography>
        <Button
          sx={{my: 2, display: "flex"}}
          fullWidth
          size="large"
          variant="contained"
          onClick={() => navigate("/")}
        >
          { translate("certos.invoice_link.success.go_to_dashboard") }
        </Button>
      </CardContent>
    </Card>
  </NoLoggedInLayout>;
}

const InvoiceLinkError = () => {
  const translate = useTranslate();
  const navigate = useNavigate();

  return <NoLoggedInLayout>
    <Card sx={{ mb: 5 }} id="invoice-link-error">
      <CardTitle text={translate("certos.invoice_link.error_in_purchase.title")} />
      <CardContent>
        <Typography> {translate("certos.invoice_link.error_in_purchase.text_1")} </Typography>
        <Typography>
          {translate("certos.invoice_link.error_in_purchase.text_2")}
          &nbsp;
          <a href="mailto:hello@constata.eu">hello@constata.eu</a>
          &nbsp;
          {translate("certos.invoice_link.error_in_purchase.text_3")}
        </Typography>
        <Button
          sx={{my: 2}}
          fullWidth
          size="large"
          variant="contained"
          onClick={() => navigate("/")}
        >
          {translate("certos.invoice_link.success.go_to_dashboard")}
        </Button>
      </CardContent>
    </Card>
  </NoLoggedInLayout>;
}

export {InvoiceLink, InvoiceLinkSuccess, InvoiceLinkError};
