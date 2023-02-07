import React from "react";
import { List, Datagrid, TextField, ShowButton, Show, SimpleShowLayout,
         ReferenceField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";
import MyUrlField from "../components/my_url_field";
import ParsedDateTextField from "../components/parsed_date_textfield";
import MyBooleanFilter from "../components/boolean_filter";
import PersonTextField from "../components/person_textfield";
import TokensAndCountField from "../components/tokens_and_count_field";
import TokensField from "../components/tokens_field";
import AcceptedField from "../components/accepted_field";


function TermsAcceptanceList() {

  const termsAcceptanceFilters = [
    <MyBooleanFilter source="acceptedIsSet" resource="TermsAcceptance" alwaysOn={true} />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={termsAcceptanceFilters}
      filterDefaultValues={{ acceptedIsSet: false }}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
        <TokensField source='missingTokens' sortable={false} />
        <ParsedDateTextField source='createdAt' />
        <ParsedDateTextField source='lastParkedDate' sortable={false} />
        <AcceptedField source='accepted' />
        <ShowButton />
      </Datagrid>
    </List>
  );
}


function TermsAcceptanceShow(){
  return (
    <Show>
      <SimpleShowLayout >
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <TokensAndCountField source="parkedTokens" count="parkedCount" />
        <TokensField source='missingTokens' />
        <ParsedDateTextField source='createdAt' />
        <ParsedDateTextField source='firstParkedDate' />
        <ParsedDateTextField source='lastParkedDate' />
        <AcceptedField source='accepted' />
        <MyUrlField source='pendingInvoiceLinkUrl' />
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='hash' />
        <TextField source='evidence' />
      </SimpleShowLayout>
    </Show>
  );
};

export {TermsAcceptanceList, TermsAcceptanceShow};