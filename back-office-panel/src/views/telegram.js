import React from "react";
import { List, Datagrid, TextField, ShowButton, Show, SimpleShowLayout, 
         ReferenceField } from 'react-admin'
import { PostPagination, defaultSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";

function TelegramList() {

  const telegramFilters = [
    <FilterTextInput source="idLike" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="usernameLike" />,
    <FilterTextInput source="firstNameLike" />,
    <FilterTextInput source="lastNameLike" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={telegramFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id'/>
        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='username'/>
        <TextField source='firstName'/>
        <TextField source='lastName'/>
        <ShowButton />
      </Datagrid>
    </List>
  );
}


function TelegramShow() {
  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='id'/>
        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='username'/>
        <TextField source='firstName'/>
        <TextField source='lastName'/>
      </SimpleShowLayout>
    </Show>
  );
}

export {TelegramList, TelegramShow};
