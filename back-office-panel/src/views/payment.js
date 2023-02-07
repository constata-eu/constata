import React from "react";
import { List, Datagrid, TextField, ShowButton, Show, SimpleShowLayout,
        NumberField, ReferenceField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import { TopToolbarDefault } from "../components/top_toolbars";
import SelectPaymentSources from "../components/select_payment_source";
import TranslatedTextField from "../components/translated_textfield";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import PersonTextField from "../components/person_textfield";


const paymentGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id' />
    <ReferenceField source="orgId" reference="Org" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <ReferenceField source="invoiceId" reference="Invoice" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TranslatedTextField source="paymentSource" translation="resources.Payment.fields.paymentSources" />
    <NumberField source="amount" options={{ style: 'currency', currency: 'EUR' }} />
    <TextField source='tokens' />
    <ParsedDateTextField source='createdAt' />
    <ShowButton />
  </Datagrid>


function PaymentList() {

  const paymentFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="invoiceIdEq" />,
    <SelectPaymentSources source="paymentSourceEq" />,
    <FilterTextInput source="tokensEq" />,
    <FilterTextInput source="tokensGt" />,
    <FilterTextInput source="tokensLt" />,
    <FilterTextInput source="amountEq" />,
    <FilterTextInput source="amountGt" />,
    <FilterTextInput source="amountLt" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={paymentFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {paymentGrid}
    </List>
  );
}


function PaymentShow(){
  return (
  <Show>
    <SimpleShowLayout >
      <TextField source='id'/>
      <ReferenceField source="orgId" reference="Org" link="show">
        <PersonTextField source="id" />
      </ReferenceField>
      <ReferenceField source="invoiceId" reference="Invoice" link="show">
        <TextField source="id" />
      </ReferenceField>
      <NumberField source="amount" options={{ style: 'currency', currency: 'EUR' }} />
      <TextField source='tokens' />
      <TextField source='fees' />
      <TranslatedTextField source="paymentSource" translation="resources.Payment.fields.paymentSources" />
      <ParsedDateTextField source='createdAt' />
      <TextField source='clearingData' />
    </SimpleShowLayout>
  </Show>
  );
};

export {PaymentList, PaymentShow, paymentGrid};