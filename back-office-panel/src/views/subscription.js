import React from "react";
import { Datagrid, TextField, TextInput, BooleanField, ReferenceField,
         ShowButton, FunctionField, EditButton, Show, SimpleShowLayout, NumberInput, 
         required, Edit, SimpleForm, BooleanInput } from 'react-admin'
import {OnlySaveToolbar} from "../components/bottom_toolbars";
import TranslatedTextField from "../components/translated_textfield";
import PersonTextField from "../components/person_textfield";


const subscriptionGrid =
  <Datagrid bulkActionButtons={false}>
    <ReferenceField source="orgId" reference="Org" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <TextField source='planName' />
    <TranslatedTextField source="defaultPaymentSource" translation="resources.Payment.fields.paymentSources" />
    <FunctionField source='maxMonthlyGift'
      render={record => `${record.maxMonthlyGift} tokens` }
    />
    <FunctionField source='monthlyGiftRemainder' sortable={false}
      render={record => `${record.monthlyGiftRemainder} tokens` }
    />
    <FunctionField source='pricePerToken'
      render={record => `€ ${record.pricePerToken / 100}` }
    />
    <BooleanField source='isActive'/>
    <ShowButton />
    <EditButton />
  </Datagrid>;


const SubscriptionShow = () => {
  return (
    <Show>
      <SimpleShowLayout >
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <TextField source='planName' />
        <TranslatedTextField source="defaultPaymentSource" translation="resources.Payment.fields.paymentSources" />
        <FunctionField source='maxMonthlyGift'
          render={record => `${record.maxMonthlyGift} tokens` }
        />
        <FunctionField source='monthlyGiftRemainder'
          render={record => `${record.monthlyGiftRemainder} tokens` }
        />
        <FunctionField source='pricePerToken'
          render={record => `€ ${record.pricePerToken / 100}` }
        />
        <BooleanField source='isActive' />
        <TextField source='invoicingDay' />
        <TextField source='requiredTokenPurchase' />
        <TextField source='stripeSubscriptionId' />
      </SimpleShowLayout>
    </Show>
  );
}

const SubscriptionEdit = () => {
  return (
  <Edit redirect="show">
      <SimpleForm 
        toolbar={<OnlySaveToolbar />}
        warnWhenUnsavedChanges>
        <TextInput disabled source="id" validate={required()} />
        <TextInput disabled source="orgId" validate={required()} />
        <NumberInput source="maxMonthlyGift" validate={required()} />
        <NumberInput source="pricePerToken" validate={required()} />
        <BooleanInput source="isActive" />
      </SimpleForm>
  </Edit>
  );
};


export {SubscriptionShow, SubscriptionEdit, subscriptionGrid};