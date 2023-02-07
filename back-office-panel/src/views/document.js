import React, { useCallback } from "react";
import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { List, Datagrid, TextField, ShowButton, Show, SimpleShowLayout, useDataProvider, 
         Button, useNotify, ReferenceField, BooleanField } from 'react-admin';
import {GetApp} from '@mui/icons-material';
import { PostPagination, documentSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import ParsedDateTextField from "../components/parsed_date_textfield";
import TranslatedTextField from "../components/translated_textfield";
import FilterTextInput from "../components/filter_textinput";
import MyBooleanFilter from "../components/boolean_filter";
import SelectDocumentSources from "../components/select_document_source";

const documentFilters = [
  <MyBooleanFilter source="fundedEq" resource="Document" alwaysOn={true} />,
  <FilterTextInput source="idLike" />,
  <FilterTextInput source="orgIdEq" />,
  <FilterTextInput source="personIdEq" />,
  <FilterTextInput source="bulletinIdEq" />,
  <FilterTextInput source="storyIdEq" />,
  <SelectDocumentSources source="sourcedFromEq" />,
];

function DocumentList() {
  return (
    <List
      empty={false}
      sort={documentSort}
      filters={documentFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {documentGrid}
    </List>
  );
}

const documentGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id' />
    <ReferenceField source="orgId" reference="Org" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="personId" reference="Person" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="storyId" reference="Story" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="bulletinId" reference="Bulletin" link="show">
      <TranslatedTextField source="state" translation="resources.Bulletin.fields.states" />
    </ReferenceField>
    <TranslatedTextField source="sourcedFrom" translation="resources.Document.fields.sourcedFroms" />
    <ParsedDateTextField source='createdAt' />
    <ParsedDateTextField source='fundedAt' />
    <ShowButton />
  </Datagrid>;

function DocumentShow() {
  const { id } = useParams();
  const dataProvider = useDataProvider();
  const permissions = localStorage.getItem('permissions');
  const [downloadProofLink, setDownloadProofLink] = useState();
  const notify = useNotify();

  const getDownloadProof = useCallback(async () => {
    try {
      let {data} = await dataProvider.getOne('DownloadProofLink', { id });
      setDownloadProofLink(data);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  }, [dataProvider, notify, id]);


  useEffect(() => {
    if (permissions === 'SuperAdmin') {
      getDownloadProof()
    }
  }, [permissions, getDownloadProof]);

  return (
    <Show>
      <SimpleShowLayout>
        <TextField source='id' />

        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="storyId" reference="Story" link="show">
          <TextField source="id" />
        </ReferenceField>

        <TranslatedTextField source="sourcedFrom" translation="resources.Document.fields.sourcedFroms" />
        <ParsedDateTextField source='createdAt' />
        <BooleanField source='funded' />
        <ParsedDateTextField source='fundedAt' />
        <ReferenceField source="bulletinId" reference="Bulletin" link="show">
          <TranslatedTextField source="state" translation="resources.Bulletin.fields.states" />
        </ReferenceField>
        <TextField source='cost' />
        <ReferenceField source="giftId" reference="Gift" link="show">
          <TextField source="id" />
        </ReferenceField>
        
        { downloadProofLink?.url &&
          permissions === 'SuperAdmin' &&
              <Button
                href={downloadProofLink.url}
                label='resources.Document.fields.downloadHtml'
                target="_blank">
              <GetApp />
              </Button>
        }
      </SimpleShowLayout>
    </Show>
  );
}

export {DocumentList, DocumentShow, documentGrid, documentFilters};
