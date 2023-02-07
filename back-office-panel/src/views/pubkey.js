import React from "react";
import { List, Datagrid, TextField, Show, SimpleShowLayout, ShowButton, 
         ReferenceField } from 'react-admin'
import { PostPagination } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";
import PersonTextField from "../components/person_textfield";


function PubkeyList() {

  const pubkeyFilters = [
    <FilterTextInput source="idLike" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="bulletinIdEq" />,
  ];

  return (
    <List
      empty={false}
      sort={{ field: 'personId', order: 'DESC' }}
      filters={pubkeyFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id' />
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='hash' sortable={false} />
        <ShowButton />
      </Datagrid>
    </List>
  );
}

function PubkeyShow(){

  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='id' />
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='evidencePayload' />
        <TextField source='hash' />
        <TextField source='evidenceSignature' />
        <TextField source='signatureHash' />
      </SimpleShowLayout>
    </Show>
  );
};

export {PubkeyList, PubkeyShow};