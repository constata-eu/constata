import React from "react";
import { List, Datagrid, ShowButton, TextField, BooleanField, required, Create, SimpleForm,
         Show, SimpleShowLayout, BooleanInput, ReferenceField, TextInput, useTranslate,
         FunctionField, TopToolbar, useShowController, useDataProvider, useNotify, useRedirect,
         FileInput, FileField } from 'react-admin'
import { PostPagination, convertBase64, defaultSort } from "../components/utils";
import { TopToolbarDefault } from "../components/top_toolbars";
import { OnlySaveToolbar } from "../components/bottom_toolbars";
import FilterTextInput from "../components/filter_textinput";
import ParsedDateTextField from "../components/parsed_date_textfield";
import SelectDeletionReason from "../components/select_deletion_reason";
import TranslatedTextField from "../components/translated_textfield";
import PhysicalDeletionModal from "../components/physical_deletion_modal";
import PersonTextField from "../components/person_textfield";


function OrgDeletionList() {
  const permissions = localStorage.getItem('permissions');

  const orgDeletionFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="storyIdEq" />,
    <SelectDeletionReason source="reasonEq" />,
    <BooleanInput source="completedEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={orgDeletionFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id' />
        <ReferenceField source="orgId" reference="Org" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="storyId" reference="Story" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TranslatedTextField source="reason" translation="resources.OrgDeletion.fields.reasons" />
        <BooleanField source='completed' />
        <TextField source='approvingAdminUser' sortable={false} />
        <ParsedDateTextField source='startedAt' />
        <ShowButton />
        <FunctionField
          render={record => {
            if (!record.completed && permissions === 'SuperAdmin') {
              return <PhysicalDeletionModal orgDeletionId={record.id} />
            }
          }}
        />
      </Datagrid>
    </List>
  );
}

function OrgDeletionShow(){
  const permissions = localStorage.getItem('permissions');
  const { record } = useShowController();

  const TopToolbarPersonDelete = () => {
    if (!record) return <TopToolbar/>
    return (
      <TopToolbar>
        {!record.completed && permissions === 'SuperAdmin' &&
          <PhysicalDeletionModal orgDeletionId={record.id} />
        }
      </TopToolbar>
    )
  };

  return (
    <Show actions={<TopToolbarPersonDelete />}>
      <SimpleShowLayout >
        <TextField source='id' />
        <ReferenceField source="orgId" reference="Org" link="show" sortable={false}>
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="storyId" reference="Story" link="show" sortable={false}>
          <TextField source="id" />
        </ReferenceField>
        <TranslatedTextField source="reason" translation="resources.OrgDeletion.fields.reasons" />
        <TextField source='description' />
        <BooleanField source='completed' />
        <TextField source='approvingAdminUser' sortable={false} />
        <ParsedDateTextField source='startedAt' />
      </SimpleShowLayout>
    </Show>
  );
}

const OrgDeletionCreate = () => {
  const translate = useTranslate();
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const redirect = useRedirect();

  const save = async values => {
    try {
      let raw_base64 = values.evidence?.rawFile ? await convertBase64(values.evidence?.rawFile) : undefined;
      values.evidence = raw_base64 ? raw_base64.split('base64,')[1] : undefined;
      let {data} = await dataProvider.create('OrgDeletion', { data: values });
      notify('resources.actions.updated');
      redirect(`/OrgDeletion/${data.id}/show`);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  };

  return (
    <Create title='resources.OrgDeletion.create' resource="OrgDeletion">
      <>
        <i className='warning-message'>
          {translate('resources.actions.orgDeletionMessage')}
        </i>
        <SimpleForm
          onSubmit={save}
          toolbar={<OnlySaveToolbar alwaysEnable={true} />}
          >
          <TextInput source='orgId' disabled validate={required()} />
          <SelectDeletionReason source="reason" validate={required()} />
          <TextInput source='description' multiline resettable autoComplete="off" validate={required()} />
          <FileInput source="evidence" accept=".zip,.rar,.7zip" maxSize={52000000} validate={required()}
           placeholder="Drag the file or click here to select it (.zip)"
          >
            <FileField source="src" title="title" />
          </FileInput>
        </SimpleForm>
      </>
    </Create>
  );
}

export {OrgDeletionList, OrgDeletionShow, OrgDeletionCreate};
