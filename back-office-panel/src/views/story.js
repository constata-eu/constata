import React from "react";
import { List, Datagrid, TextField, ShowButton, Show, ReferenceManyField,
         ReferenceField, Tab, TabbedShowLayout } from 'react-admin'
import { PostPagination, defaultSort, documentSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";
import { documentGrid } from "./document";

function StoryList() {

  const storyFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="orgIdEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={storyFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {storyGrid}
    </List>
  );
}

const storyGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id'/>
    <ReferenceField source="orgId" reference="Org" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TextField source='markers' />
    <TextField source='openUntil' />
    <TextField source='totalDocumentsCount' sortable={false} />
    <ShowButton />
  </Datagrid>;


function StoryShow(){
  return (
    <Show>
      <TabbedShowLayout syncWithLocation={false}>
        <Tab label="resources.actions.details" path="details">
          <TextField source='id' />
          <ReferenceField source="orgId" reference="Org" link="show">
            <TextField source="id" />
          </ReferenceField>
          <TextField source='totalDocumentsCount' />
          <TextField source='publishedDocumentsCount' />
          <TextField source='markers' />
          <TextField source='openUntil' />
          <TextField source='privateMarkers' />
        </Tab>
        <Tab label="resources.Document.many" path="documents">
          <div className="nested-resource" >
            <ReferenceManyField reference="Document" target="storyIdEq" label=""
              sort={documentSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {documentGrid}
            </ReferenceManyField>
          </div>
        </Tab>
      </TabbedShowLayout>
    </Show>
  );
};

export {StoryList, StoryShow, storyGrid};