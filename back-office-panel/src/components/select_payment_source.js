import { SelectInput } from 'react-admin';

const SelectPaymentSources = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} choices={[
    { id: 'BankBbva', name: 'resources.Payment.fields.paymentSources.BANK_BBVA' },
    { id: 'Stripe', name: 'resources.Payment.fields.paymentSources.STRIPE' },
    { id: 'BtcPay', name: 'resources.Payment.fields.paymentSources.BTC_PAY' },
  ]} />
)};

export default SelectPaymentSources;