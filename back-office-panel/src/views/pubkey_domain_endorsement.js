import React from "react";
import { List, Datagrid, TextField, ShowButton, SelectInput, Show, SimpleShowLayout,
         ReferenceField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import { TopToolbarDefault } from "../components/top_toolbars";
import TranslatedTextField from "../components/translated_textfield";
import FilterTextInput from "../components/filter_textinput";
import PersonTextField from "../components/person_textfield";


function PubkeyDomainEndorsementList() {

  const pubkeyDomainEndorsementFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <SelectInput source="stateEq"
      choices={[
        { id: 'accepted', name: "resources.PubkeyDomainEndorsement.fields.states.accepted" },
        { id: 'pending', name: "resources.PubkeyDomainEndorsement.fields.states.pending" },
        { id: 'failed', name: "resources.PubkeyDomainEndorsement.fields.states.failed" },
      ]} />,
    <FilterTextInput source="pubkeyIdLike" />,
    <FilterTextInput source="domainLike" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={pubkeyDomainEndorsementFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {pubkeyDomainEndorsementGrid}
    </List>
  );
}

const pubkeyDomainEndorsementGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id'/>
    <ReferenceField source="personId" reference="Person" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <ReferenceField source="orgId" reference="Org" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <TextField source='domain'/>
    <ReferenceField source="pubkeyId" reference="Pubkey" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TranslatedTextField source="state" translation="resources.PubkeyDomainEndorsement.fields.states" />
    <ShowButton />
  </Datagrid>;


function PubkeyDomainEndorsementShow(){
  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='id' />
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <TextField source='domain' />
        <ReferenceField source="pubkeyId" reference="Pubkey" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TranslatedTextField source="state" translation="resources.PubkeyDomainEndorsement.fields.states" />
        <TextField source='requestSignature' sortable={false} />
        <TextField source='attempts' />
        <ReferenceField source="bulletinId" reference="Buletin" link="show">
          <TextField source="id" />
        </ReferenceField>
      </SimpleShowLayout>
    </Show>
  );
};

export {PubkeyDomainEndorsementList, PubkeyDomainEndorsementShow, pubkeyDomainEndorsementGrid};