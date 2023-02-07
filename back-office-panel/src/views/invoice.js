import React from "react";
import { List, Datagrid, TextField, SimpleShowLayout, ReferenceField, ShowButton, 
         BooleanField, Show, NumberField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import SelectPaymentSources from "../components/select_payment_source";
import TranslatedTextField from "../components/translated_textfield";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import MyUrlField from "../components/my_url_field";
import MyBooleanFilter from "../components/boolean_filter";

const invoiceFilters = [
  <MyBooleanFilter source="paidEq" resource="Invoice" alwaysOn={true} />,
  <MyBooleanFilter source="expiredEq" resource="Invoice" alwaysOn={true} />,
  <FilterTextInput source="idEq" />,
  <FilterTextInput source="orgIdEq" />,
  <FilterTextInput source="paymentIdEq" />,
  <SelectPaymentSources source="paymentSourceEq" />,
  <FilterTextInput source="tokensEq" />,
  <FilterTextInput source="tokensGt" />,
  <FilterTextInput source="tokensLt" />,
  <FilterTextInput source="amountEq" />,
  <FilterTextInput source="amountGt" />,
  <FilterTextInput source="amountLt" />,
];

function InvoiceList() {
  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={invoiceFilters}
      filterDefaultValues={{ paidEq: false, expiredEq: false }}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {invoiceGrid}
    </List>
  );
}

const invoiceGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id'/>
    <ReferenceField source="orgId" reference="Org" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="paymentId" reference="Payment" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TranslatedTextField source="paymentSource" translation="resources.Payment.fields.paymentSources" />
    <NumberField source="amount" options={{ style: 'currency', currency: 'EUR' }} />
    <TextField source='tokens' />
    <BooleanField source='paid' />
    <BooleanField source='expired' />
    <ParsedDateTextField source='createdAt' />
    <MyUrlField source='url' sortable={false} />
    <ShowButton />
  </Datagrid>;


function InvoiceShow(){
  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='id' />
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="paymentId" reference="Payment" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ParsedDateTextField source='createdAt' />
        <NumberField source="amount" options={{ style: 'currency', currency: 'EUR' }} />
        <TextField source='tokens' />
        <TranslatedTextField source="paymentSource" translation="resources.Payment.fields.paymentSources" />
        <TextField source='description' />
        <TextField source='externalId' />
        <MyUrlField source='url' />
        <BooleanField source='paid' />
        <TextField source='notifiedOn' />
        <BooleanField source='expired' />
      </SimpleShowLayout>
    </Show>
  );
}

export {InvoiceList, InvoiceShow, invoiceGrid, invoiceFilters};