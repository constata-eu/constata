import React from "react";
import { List, Datagrid, TextField, TextInput, SimpleShowLayout, Create, SimpleForm, required,
         SelectInput, ShowButton, EditButton, Show, Edit, ReferenceField,
         FileInput, useDataProvider, useNotify, FileField, useRedirect } from 'react-admin'
import { PostPagination, demonymList, convertBase64, defaultSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import {OnlySaveToolbar} from '../components/bottom_toolbars';
import CountryList from "country-list";
import { useNavigate } from "react-router-dom";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import TextFieldIdNumber from "../components/text_field_id_number";
import ParsedDateInput from "../components/parsed_dateinput";
import PersonTextField from "../components/person_textfield";
import _ from "lodash";


function KycEndorsementList() {

  const kycEndorsementFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="nameLike" />,
    <FilterTextInput source="lastNameLike" />,
    <FilterTextInput source="idTypeLike" />,
    <FilterTextInput source="idNumberLike" />,
    <FilterTextInput source="countryLike" />,
    <FilterTextInput source="jobTitleLike" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={kycEndorsementFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault />}
    >
      {kycEndorsementGrid}
    </List>
  );
}

const kycEndorsementGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id'/>
    <ReferenceField source="personId" reference="Person" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <ReferenceField source="orgId" reference="Org" link="show">
      <PersonTextField source="id" />
    </ReferenceField>
    <ReferenceField source="storyId" reference="Story" link="show">
      <TextField source="id" />
    </ReferenceField>
    <TextField source='name'/>
    <TextField source='lastName'/>
    <TextFieldIdNumber source='idNumber' />
    <TextField source='country'/>
    <ParsedDateTextField source='createdAt' />
    <ShowButton />
    <EditButton />
  </Datagrid>;


function KycEndorsementShow(){
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
        <ReferenceField source="storyId" reference="Story" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ParsedDateTextField source='createdAt' />
        <TextField source='name' />
        <TextField source='lastName' />
        <TextFieldIdNumber source='idNumber' />
        <ParsedDateTextField source='birthdate' />
        <TextField source='nationality' />
        <TextField source='country' />
        <TextField source='jobTitle' />
        <TextField source='legalEntityName' />
        <TextField source='legalEntityCountry' />
        <TextField source='legalEntityRegistration' />
        <TextField source='legalEntityTaxId' />
        <TextField source='legalEntityLinkedinId' />
      
      </SimpleShowLayout>
    </Show>
  );
};

const transformValues = async values => {
  let form = _.omit(values, ["evidenceZip", "id", "personId", "orgId", "createdAt", "storyId"]);
  form.evidence = [];
  const ev = values.evidenceZip?.rawFile;
  if(ev) {
    form.evidence.push({
      filename: "",
      payload: (await convertBase64(ev)).split('base64,')[1]
    });
  }
  
  return { personId: values.personId, input: form };
  }

const KycEndorsementCreate = () => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const redirect = useRedirect();
  const countries = CountryList.getData();

  const save = async values => {
    try {
      let kycForm = await transformValues(values);
      let {data} = await dataProvider.create('KycEndorsement', { data: kycForm });
      notify('resources.actions.created');
      redirect(`/KycEndorsement/${data.id}/show`);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  };

  return (
    <Create title='resources.KycEndorsement.create'>
        <SimpleForm onSubmit={save}>
          <TextInput source='personId' disabled validate={required()} />
          <TextInput source='name' validate={required()} />
          <TextInput source='lastName' />
          <TextInput source='idType' />
          <TextInput source='idNumber' />
          <ParsedDateInput source='birthdate' />
          <SelectInput source='country' choices={countries} optionValue="name" />
          <SelectInput source='nationality' choices={demonymList()} optionValue="name" />
          <TextInput source='jobTitle' />
          <TextInput source='legalEntityName' />
          <SelectInput source='legalEntityCountry' choices={countries} optionValue="name" />
          <TextInput source='legalEntityRegistration' />
          <TextInput source='legalEntityTaxId' />
          <TextInput source='legalEntityLinkedinId' />
          <FileInput source="evidenceZip" accept=".zip,.rar,.7zip" maxSize={52000000}
            placeholder="Drag the file or click here to select it (.zip)"
          >
            <FileField source="src" title="title" />
          </FileInput>
        </SimpleForm>
    </Create>
  );
}

const KycEndorsementEdit = () => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const navigate = useNavigate();
  const countries = CountryList.getData();

  const save = async values => {
    try {
      let kycForm = await transformValues(values);
      let {data} = await dataProvider.update('KycEndorsement', { data: kycForm });
      notify('resources.actions.updated');
      navigate(`/KycEndorsement/${data.id}/show`);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  };

  return (
  <Edit>
      <SimpleForm
        onSubmit={save}
        toolbar={<OnlySaveToolbar />}
      >
        <TextInput disabled source="id" validate={required()} />
        <TextInput disabled source="personId" validate={required()} />
        <TextInput source='name' validate={required()} />
        <TextInput source='lastName' />
        <TextInput source='idType' />
        <TextInput source='idNumber' />
        <ParsedDateInput source='birthdate' />
        <SelectInput source='country' choices={countries} optionValue="name" />
        <SelectInput source='nationality' choices={demonymList()} optionValue="name" />
        <TextInput source='jobTitle' />
        <TextInput source='legalEntityName' />
        <SelectInput source='legalEntityCountry' choices={countries} optionValue="name" />
        <TextInput source='legalEntityRegistration' />
        <TextInput source='legalEntityTaxId' />
        <TextInput source='legalEntityLinkedinId' />
        <FileInput source="evidenceZip" accept=".zip,.rar,.7zip" maxSize={52000000}
          placeholder="Drag the file or click here to select it (.zip)"
        >
          <FileField source="src" title="title" />
        </FileInput>
      </SimpleForm>
  </Edit>
  );
};


export {KycEndorsementList, KycEndorsementShow, KycEndorsementCreate, KycEndorsementEdit, kycEndorsementGrid};
