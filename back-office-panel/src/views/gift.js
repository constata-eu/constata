import React from "react";
import { List, Datagrid, TextField, TextInput, Show, Create, SimpleForm, required, ShowButton,
         NumberInput, ReferenceField, SimpleShowLayout } from 'react-admin';
import { PostPagination } from "../components/utils";
import { TopToolbarDefault } from "../components/top_toolbars";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import PersonTextField from "../components/person_textfield";

const giftGrid =
  <Datagrid bulkActionButtons={false}>
      <TextField source='id' />
      <ReferenceField source="orgId" reference="Org" link="show">
        <PersonTextField source="id" />
      </ReferenceField>
      <TextField source='tokens' />
      <ParsedDateTextField source='createdAt' />
      <ShowButton />
  </Datagrid>


function GiftList() {

  const GiftFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="tokensEq" />,
    <FilterTextInput source="tokensGt" />,
    <FilterTextInput source="tokensLt" />,
  ];

  return (
    <List
      empty={false}
      sort={{ field: 'createdAt', order: 'DESC' }}
      filters={GiftFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault />}
      >
        {giftGrid}
    </List>
  );
}


function GiftShow(){
  return (
    <Show>
      <SimpleShowLayout>
        <TextField source='id' />
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <TextField source='tokens' />
        <ParsedDateTextField source='createdAt' />
        <TextField source='reason' />
      </SimpleShowLayout>
    </Show>
  );
}

const GiftCreate = () => {
  return (
    <Create title='resources.Gift.create' resource="Gift">
        <SimpleForm warnWhenUnsavedChanges>
          <TextInput source='orgId' disabled validate={required()} />
          <NumberInput source='tokens' validate={required()} />
          <TextInput source='reason' autoComplete="off" validate={required()} />
        </SimpleForm>
    </Create>
  );
}

export {GiftList, GiftShow, GiftCreate, giftGrid};