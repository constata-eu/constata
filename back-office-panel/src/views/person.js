import React,{useState} from "react";
import { List, Datagrid, ShowButton, TextField, FunctionField, Show, NumberField, TopToolbar,
         Tab, TabbedShowLayout, useTranslate, Button, ReferenceField, BooleanField, FilterForm,
         ReferenceManyField } from 'react-admin'
import { PostPagination, defaultSort, documentSort } from "../components/utils";
import {PermIdentity} from '@mui/icons-material';
import {TopToolbarDefault} from "../components/top_toolbars";
import { useParams } from 'react-router-dom';
import { Fingerprint } from '@mui/icons-material';
import FilterTextInput from "../components/filter_textinput";
import { FieldCopyWithUrl } from "../components/copy_to_clipboard";
import ParsedDateTextField from "../components/parsed_date_textfield";
import { documentGrid, documentFilters } from "./document";
import { kycRequestGrid, kycRequestFilters } from "./kyc_request";
import { kycEndorsementGrid } from "./kyc_endorsement";
import { pubkeyDomainEndorsementGrid } from "./pubkey_domain_endorsement";
export const PersonIcon = PermIdentity;


function PersonList() {

  const userFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="orgIdEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={userFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      {personGrid}
    </List>
  );
}

const personGrid = 
  <Datagrid bulkActionButtons={false}>
    <TextField source='id' />
    <ReferenceField source="orgId" reference="Org" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="pubkey" reference="Pubkey" link="show" sortable={false}>
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="emailAddress" reference="EmailAddress" link="show" sortable={false}>
      <TextField source="address" />
    </ReferenceField>
    <ReferenceField source="telegram" reference="Telegram" link="show" sortable={false}>
      <TextField source="firstName" />
    </ReferenceField>
    <BooleanField source='isTermsAccepted' sortable={false} />
    <ShowButton />
  </Datagrid>


function PersonShow(){
  const translate = useTranslate();
  const { id } = useParams();
  const [documentFilter, setDocumentFilter] = useState({});
  const [kycRequestFilter, setKycRequestFilter] = useState({});

  const ListActionsRedirectAutocomplete = () => {
    return (
      <TopToolbar>
        {id && 
          <>
            <Button 
              href={`#/KycEndorsement/create?source={"personId":"${id}"}`}
              label="resources.actions.createKycEndorsement">
              <Fingerprint/>
            </Button>
          </>
        }
      </TopToolbar>
    )
  };
  return (
    <Show actions={<ListActionsRedirectAutocomplete />} >
      <TabbedShowLayout syncWithLocation={false}>
        <Tab label="resources.actions.details" path="details">
          <NumberField source='id' />
          <ReferenceField source="orgId" reference="Org" link="show" sortable={false}>
            <TextField source="id" />
          </ReferenceField>
          <ReferenceField source="pubkey" reference="Pubkey" link="show">
            <TextField source="id" />
          </ReferenceField>
          <ReferenceField source="emailAddress" reference="EmailAddress" link="show">
            <TextField source="address" />
          </ReferenceField>
          <ReferenceField source="telegram" reference="Telegram" link="show" sortable={false}>
            <TextField source="firstName" />
          </ReferenceField>
          <ParsedDateTextField source='registrationDate' />
          <FunctionField source='termsUrl'
            render={record => {
              if (record.isTermsAccepted) {
                return translate("resources.Person.fields.already_accepted");
              } else {
                return <FieldCopyWithUrl
                  text={translate("resources.Person.fields.not_accepted_yet")}
                  url={record.termsUrl}
                />;
              }
            }}
          />;
          <FunctionField source='createCredentialsUrl'
            render={record => {
              if (!record.createCredentialsUrl) {
                return translate("resources.Person.fields.credentials_created");
              } else {
                return <a href={record.createCredentialsUrl} target="_blank" rel="noreferrer">
                  {translate("resources.Person.fields.credentials_to_create")}
                </a>;
              }
            }}
          />;
        </Tab>
        <Tab label="resources.Document.many" path="documents">
          <div className="nested-resource" >
            <FilterForm
              filters={documentFilters}
              setFilters={setDocumentFilter}
              displayedFilters={ {"idLike": false} }
             />
            <ReferenceManyField reference="Document" target="personIdEq" label=""
              sort={documentSort}
              filter={documentFilter}
              perPage={20}
              pagination={<PostPagination />}
            >
              {documentGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.KycRequest.one" path="KycRequest">
          <div className="nested-resource" >
            <FilterForm
              filters={kycRequestFilters}
              setFilters={setKycRequestFilter}
              displayedFilters={ {"idEq": false, "personIdEq": false, "orgIdEq": false, "nameLike": false, "lastNameLike": false} }
             />
            <ReferenceManyField reference="KycRequest" target="personIdEq" label=""
              filter={kycRequestFilter}
              perPage={20}
              pagination={<PostPagination />}
            >
              {kycRequestGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.KycEndorsement.one" path="kycEndorsement">
          <div className="nested-resource" >
            <ReferenceManyField reference="KycEndorsement" target="personIdEq" label=""
              perPage={20}
              pagination={<PostPagination />}
            >
              {kycEndorsementGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.PubkeyDomainEndorsement.many" path="pubkeyDomainEndorsements">
          <div className="nested-resource" >
            <ReferenceManyField reference="PubkeyDomainEndorsement" target="personIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {pubkeyDomainEndorsementGrid}
            </ReferenceManyField>
          </div>
        </Tab>
      </TabbedShowLayout>
    </Show>
  );
}

export {PersonList, PersonShow, personGrid};
