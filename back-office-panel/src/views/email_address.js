import React from "react";
import { List, Datagrid, TextField, BooleanField, ShowButton, Show, SimpleShowLayout, 
         ReferenceField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";

function EmailList() {

  const emailFilters = [
    <FilterTextInput source="addressLike" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="bulletinIdEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={emailFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='address'/>
        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <BooleanField source='maybeSpoofed' sortable={false} />
        <ShowButton />
      </Datagrid>
    </List>
  );
}


function EmailShow() {
  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='address'/>
        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <BooleanField source='maybeSpoofed' />
        <TextField source='evidence' />
      </SimpleShowLayout>
    </Show>
  );
}

export {EmailList, EmailShow};