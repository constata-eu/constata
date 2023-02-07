import React from "react";
import { List, Datagrid, TextField, ShowButton, SelectInput, Show, Tab,
        FunctionField, NumberField, TabbedShowLayout, ReferenceManyField,
        Button } from 'react-admin'
import {PostPagination, defaultSort, documentSort} from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import {GetApp} from '@mui/icons-material';
import TranslatedTextField from "../components/translated_textfield";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import { documentGrid } from "./document";


const GetTransactionHashLink = (mempoolInfoUrl, transactionHash) => {
  if (!mempoolInfoUrl) return transactionHash;
  return <a href={mempoolInfoUrl} target="_blank" rel="noreferrer">
    {transactionHash}
  </a>
}

function BulletinList() {

  const bulletinFilters = [
    <FilterTextInput source="idEq" />,
    <SelectInput source="stateEq" choices={[
      { id: 'draft', name: "resources.Bulletin.fields.states.draft" },
      { id: 'proposed', name: "resources.Bulletin.fields.states.proposed" },
      { id: 'submitted', name: "resources.Bulletin.fields.states.submitted" },
      { id: 'published', name: "resources.Bulletin.fields.states.published" },
    ]} />,
    <FilterTextInput source='hashEq' />,
    <FilterTextInput source="transactionHashEq" />,
    <FilterTextInput source='blockHashEq' />
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={bulletinFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id'/>
        <TranslatedTextField source="state" translation="resources.Bulletin.fields.states"/>
        <ParsedDateTextField source='startedAt' />
        <TextField source='hash' sortable={false} />
        <FunctionField source='transactionHash' sortable={false}
          render={record => {
            return GetTransactionHashLink(record.mempoolInfoUrl, record.transactionHash)
          }}
        />
        <ShowButton />
      </Datagrid>
    </List>
  );
}


function BulletinShow(){
  return (
    <Show>
      <TabbedShowLayout syncWithLocation={false}>
        <Tab label="resources.actions.details" path="details">
          <NumberField  source='id' />
          <ParsedDateTextField source='startedAt' />
          <TranslatedTextField source="state" translation="resources.Bulletin.fields.states"/>
          <TextField source='hash' />
          <TextField source='documentsCount' />
          <TextField source='transaction' />
          <FunctionField source='transactionHash' sortable={false}
            render={record => {
              return GetTransactionHashLink(record.mempoolInfoUrl, record.transactionHash)
            }}
          />
          <TextField source='blockHash' />
          <TextField source='blockTime' />
          <FunctionField source='documentsHashes'
            render={record => {
              if (record.payload) {
                let name = "bulletin_" + record.id + "_hashes";
                let link = "data:text/plain;charset=utf-8," + record.payload;
                return (
                  <Button
                    href={link}
                    download={name}
                    label="resources.Bulletin.fields.downloadHashes">
                    <GetApp />
                  </Button>
                )
              }
            }}
          />
        </Tab>
        <Tab label="resources.Document.many" path="documents">
          <div className="nested-resource" >
            <ReferenceManyField reference="Document" target="bulletinIdEq" label=""
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
}

export {BulletinList, BulletinShow};