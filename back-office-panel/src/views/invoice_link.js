import React from "react";
import { List, Datagrid, TextField, TextInput, ReferenceField, ShowButton, Create,
         Show, SimpleShowLayout, SimpleForm, required} from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import { TopToolbarDefault } from "../components/top_toolbars";
import { OnlySaveToolbar } from "../components/bottom_toolbars";
import FilterTextInput from "../components/filter_textinput";
import MyUrlField from "../components/my_url_field";
import ParsedDateTextField from "../components/parsed_date_textfield";
import MyBooleanFilter from "../components/boolean_filter";
import PersonTextField from "../components/person_textfield";
import TokensAndCountField from "../components/tokens_and_count_field";


const invoiceLinkGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id' />
    <ReferenceField source="orgId" reference="Org" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <ReferenceField source="invoiceId" reference="Invoice" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
    <ParsedDateTextField source='lastParkedDate' sortable={false} />
    <MyUrlField source='url' sortable={false} />
    <ShowButton />
  </Datagrid>

const invoiceLinkFilters = [
  <MyBooleanFilter source="invoiceIdIsSet" resource="InvoiceLink" alwaysOn={true} />,
  <FilterTextInput source="idEq" />,
  <FilterTextInput source="orgIdEq" />,
  <FilterTextInput source="invoiceIdEq" />,
];

function InvoiceLinkList() {
  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={invoiceLinkFilters}
      filterDefaultValues={{ invoiceIdIsSet: false }}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault />}
    >
      {invoiceLinkGrid}
    </List>
  );
}


function InvoiceLinkShow(){
  return (
    <Show>
      <SimpleShowLayout >

        <TextField source='id' />
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="invoiceId" reference="Invoice" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
        <ParsedDateTextField source='firstParkedDate' />
        <ParsedDateTextField source='lastParkedDate' />
        <TextField source='missingTokens' />
        <MyUrlField source='url' />

      </SimpleShowLayout>
    </Show>
  );
}


const InvoiceLinkCreate = () => {
  return (
    <Create title='resources.InvoiceLink.create' resource="InvoiceLink" >
        <SimpleForm
          toolbar={<OnlySaveToolbar alwaysEnable={true} />}
          warnWhenUnsavedChanges>
          <TextInput source='orgId' disabled validate={required()} />
        </SimpleForm>
    </Create>
  );
}

export {InvoiceLinkList, InvoiceLinkShow, InvoiceLinkCreate, invoiceLinkGrid, invoiceLinkFilters};
