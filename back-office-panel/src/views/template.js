import React from "react";
import { List, Datagrid, TextField, ShowButton, Show, SimpleShowLayout, ReferenceField,
         FileField, Create, useRedirect, useNotify, useDataProvider, SimpleForm, TextInput,
         Edit, EditButton, required, FileInput, useTranslate, BooleanInput, useSafeSetState,
         Form, FunctionField } from 'react-admin';
import { DialogTitle, DialogContent, DialogActions, Dialog, Box, Button } from '@mui/material';
import { PostPagination, defaultSort, convertBase64 } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import FilterTextInput from "../components/filter_textinput";
import PersonTextField from "../components/person_textfield";
import SelectTemplateKind from "../components/select_template_kind";
import { OnlySaveToolbar } from "../components/bottom_toolbars";
import { FieldCopy } from "../components/copy_to_clipboard";


function TemplateList() {

  const templateFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="personIdEq" />,
    <FilterTextInput source="orgIdEq" />,
    <FilterTextInput source="nameLike" />,
    <SelectTemplateKind source="kindEq" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={templateFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
        {TemplateGrid}
    </List>
  );
}

const TemplateGrid = <Datagrid bulkActionButtons={false}>
  <TextField source='id'/>
  <ReferenceField source="personId" reference="Person" link="show">
    <PersonTextField source="id" />
  </ReferenceField>
  <ReferenceField source="orgId" reference="Org" link="show">
    <TextField source="id" />
  </ReferenceField>
  <TextField source='name'/>
  <TextField source='kind'/>
  <ShowButton />
  <EditButton />
</Datagrid>


function TemplateShow() {
  return (
    <Show>
      <SimpleShowLayout >
        <TextField source='id'/>
        <ReferenceField source="personId" reference="Person" link="show">
          <PersonTextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TextField source='name'/>
        <TextField source='kind'/>
        <FunctionField source='schema'
            render={record => <FieldCopy text={record.schema} /> }
        />
        <TextField source='customMessage'/>
        <TextField source='ogTitleOverride' />
      </SimpleShowLayout>
    </Show>
  );
}


const SchemaDialog = ({schema, setSchema, setUnchangedField, unchangedField}) => {
  const translate = useTranslate();
  const [open, setOpen] = useSafeSetState(false);

  const handleClickOpen = () => {
    setOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
  };
  
  const handleAddSchema = (values) => {
    try {
      if (unchangedField) {
        setSchema(JSON.stringify([...JSON.parse(values.schema), values.addSchema]));
      } else {
        setSchema(JSON.stringify([...JSON.parse(schema), values.addSchema]));
      }
    } catch {
      setSchema(JSON.stringify([values.addSchema]));
    }
    setUnchangedField(false);
  }

  const handleSubmit = (values) => {
    handleClose();
    handleAddSchema(values);
  }

  return (<Box id="add-schema-field">
    <Button fullWidth variant="outlined" onClick={handleClickOpen} sx={{mb: 3}}>
      { translate("resources.Template.add_schema_field.button") }
    </Button>
    <Dialog open={open} onClose={handleClose}>
      <Form onSubmit={handleSubmit}>
        <DialogTitle>{ translate("resources.Template.add_schema_field.button") }</DialogTitle>
        <DialogContent>
        <TextInput
          source="addSchema.name"
          key={"input_name"}
          label={ translate("resources.Template.add_schema_field.name") }
          helperText={ translate("resources.Template.add_schema_field.name_label") }
          variant="filled"
          required
          fullWidth
        />
        <BooleanInput
          source="addSchema.common"
          label={ translate("resources.Template.add_schema_field.common") }
          helperText={ translate("resources.Template.add_schema_field.common_label") }
        />
        <BooleanInput
          source="addSchema.optional"
          label={ translate("resources.Template.add_schema_field.optional") }
          helperText={ translate("resources.Template.add_schema_field.optional_label") }
        />
        <TextInput
          source="addSchema.label"
          key={"input_label"}
          label={ translate("resources.Template.add_schema_field.label") }
          helperText={ translate("resources.Template.add_schema_field.label_label") }
          variant="filled"
          required
          fullWidth
        />
        <TextInput
          source="addSchema.help"
          key={"input_help"}
          label={ translate("resources.Template.add_schema_field.help") }
          helperText={ translate("resources.Template.add_schema_field.help_label") }
          variant="filled"
          required
          fullWidth
        />
        <TextInput
          source="addSchema.sample"
          key={"input_sample"}
          label={ translate("resources.Template.add_schema_field.sample") }
          helperText={ translate("resources.Template.add_schema_field.sample_label") }
          variant="filled"
          required
          fullWidth
        />
        </DialogContent>
        <DialogActions>
          <Button onClick={handleClose}>{ translate("resources.Template.add_schema_field.cancel") }</Button>
          <Button type="submit">{ translate("resources.Template.add_schema_field.accept") }</Button>
        </DialogActions>
      </Form>
    </Dialog>
  </Box>);
}

const TemplateCreate = props => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const redirect = useRedirect();
  const translate = useTranslate();
  const [schema, setSchema] = useSafeSetState("");
  const [unchangedField, setUnchangedField] = useSafeSetState(true);

  const validateJson = () => {
    if (schema) {
      try { JSON.parse(schema)}
      catch { return translate("resources.Template.validateJson")}
    }
    return undefined;
  }

  const save = async values => {
      values.evidence = (await convertBase64(values.evidence?.rawFile)).split('base64,')[1];
      values.schema = unchangedField ? values.schema : schema.length ? schema : undefined;
      let {data} = await dataProvider.create('Template', { data: values });
      notify('resources.actions.created');
      redirect(`/Template/${data.id}/show`);
  };

  return (
    <Create title='resources.Template.create' {...props}>
        <SimpleForm onSubmit={save}>
          <Box>
          <TextInput source="orgId" disabled />
          <SelectTemplateKind source="kind" />
          <TextInput source='name' autoComplete="off" validate={required()} />
          <TextInput source='customMessage' autoComplete="off" validate={required()} />
          <TextInput source='ogTitleOverride' autoComplete="off" />
        </Box>
        <TextInput source='schema' multiline fullWidth validate={validateJson}
          onChange={(e) => {
            setUnchangedField(false);
            setSchema(e.target.value);
          }}
          format={(e) => unchangedField ? e : schema }
        />
        <SchemaDialog schema={schema} setSchema={setSchema} setUnchangedField={setUnchangedField} unchangedField={unchangedField}/>
        <FileInput source="evidence" placeholder="Arrastra el archivo o presiona aquí para seleccionarlo (.zip)"
          accept=".zip,.rar,.7zip"
          maxSize={100000000}
          validate={required()} >
          <FileField source="src" title="title" />
        </FileInput>
        </SimpleForm>
    </Create>
  );
}


const TemplateEdit = () => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const redirect = useRedirect();
  const translate = useTranslate();
  const [schema, setSchema] = useSafeSetState("");
  const [unchangedField, setUnchangedField] = useSafeSetState(true);

  const validateJson = () => {
    if (schema) {
      try { JSON.parse(schema)}
      catch { return translate("resources.Template.validateJson")}
    }
    return undefined;
  }

  const save = async values => {
    try {
      values.schema = unchangedField ? values.schema : schema.length ? schema : undefined;
      let {data} = await dataProvider.update('Template', { data: values });
      notify('resources.actions.updated');
      redirect(`/Template/${data.id}/show`);
    } catch(e) {
      notify('admin.errors.default', {type: 'warning'});
    }
  };

  return (
  <Edit>
      <SimpleForm onSubmit={save} toolbar={<OnlySaveToolbar alwaysEnable={!unchangedField} />}>
        <Box>
          <TextInput source="id" disabled />
          <TextInput source="orgId" disabled />
          <SelectTemplateKind source="kind" />
          <TextInput source='name' autoComplete="off" validate={required()} />
          <TextInput source='customMessage' autoComplete="off" validate={required()} />
          <TextInput source='ogTitleOverride' autoComplete="off" />
        </Box>
        <TextInput source='schema' multiline fullWidth validate={validateJson}
          onChange={(e) => {
            setUnchangedField(false);
            setSchema(e.target.value);
          }}
          format={(e) => unchangedField ? e : schema }
          />
        <SchemaDialog schema={schema} setSchema={setSchema} setUnchangedField={setUnchangedField} unchangedField={unchangedField}/>
      </SimpleForm>
  </Edit>
  );
};


export {TemplateList, TemplateShow, TemplateCreate, TemplateEdit, TemplateGrid};