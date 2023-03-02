import React, {useState, useEffect, useCallback} from "react";
import { List, Datagrid, TextField, SimpleShowLayout, ShowButton, TopToolbar, Button, Confirm, Toolbar,
         Show, ReferenceField, useDataProvider, SimpleForm, Edit, required, EditButton, useTranslate,
         BooleanInput, useNotify, useRedirect, FunctionField, SaveButton } from 'react-admin'
import { useParams } from 'react-router-dom';
import { Dialog, Backdrop, DialogTitle } from '@mui/material';
import { GetApp, Input, CheckCircle, Cancel } from '@mui/icons-material';
import { Divider } from '@mui/material';
import { PostPagination, createZipAndDownload } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import TranslatedTextField from "../components/translated_textfield";
import SelectKycRequestState from "../components/select_kyc_request_state";
import TextFieldIdNumber from "../components/text_field_id_number";
import KycBooleanInput from "../components/kyc_boolean_input";
import PersonTextInput from "../components/person_textinput";

const kycRequestFilters = [
  <SelectKycRequestState source="stateEq" alwaysOn={true} />,
  <FilterTextInput source="idEq" />,
  <FilterTextInput source="personIdEq" />,
  <FilterTextInput source="orgIdEq" />,
  <FilterTextInput source="nameLike" />,
  <FilterTextInput source="lastNameLike" />,
];

function KycRequestList() {
  return (
    <List
      empty={false}
      filters={kycRequestFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault />}
    >
      {kycRequestGrid}
    </List>
  );
}

const kycRequestGrid =
  <Datagrid bulkActionButtons={false}>
    <TextField source='id'/>
    <ReferenceField source="personId" reference="Person" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="orgId" reference="Org" link="show">
      <TextField source="id" />
    </ReferenceField>
    <ReferenceField source="kycEndorsementId" reference="KycEndorsement" link="show" sortable={false}>
      <TextField source="id" />
    </ReferenceField>
    <TranslatedTextField source="state" translation="resources.KycRequest.fields.states" />
    <TextField source='name'/>
    <TextField source='lastName'/>
    <ParsedDateTextField source='createdAt' />
    <ShowButton />
    <FunctionField render={record => { 
        if (record.state === "pending") {
          return <EditButton label={"resources.actions.processKycRequest"} icon={<Input />} />;
        }
    }}/>
  </Datagrid>

function KycRequestShow(){
  const dataProvider = useDataProvider();
  const { id } = useParams();
  const notify = useNotify();
  const [kycRequestState, setKycRequestState] = useState("");

  const ListActionsKycRequest = () => {
    return (
      <TopToolbar>
        {kycRequestState === "pending" && 
          <EditButton label={"resources.actions.processKycRequest"} icon={<Input />} />
        }
      </TopToolbar>
    )
  };

  const reloadKycRequest = useCallback(async () => {
    try {
      let {data} = await dataProvider.getOne('KycRequest', { id })
      setKycRequestState(data.state);
    } catch(error) {
      console.log(error);
    }
  }, [dataProvider, id]);

  useEffect(() => {
    reloadKycRequest();
  }, [reloadKycRequest]);

  const downloadKycRequestEvidence = useCallback(async () => {
    try {
      let {data} = await dataProvider.getList('KycRequestEvidence', {
        pagination: { page: 1, perPage: 200 },
        sort: 'id',
        order: 'ASC',
        filter: { kycRequestIdEq: id },
      });
      await createZipAndDownload(data, id);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  }, [dataProvider, notify, id]);

  return (
    <Show actions={<ListActionsKycRequest />} >
      <SimpleShowLayout >

        <TextField source='id' />
        <ReferenceField source="personId" reference="Person" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="orgId" reference="Org" link="show">
          <TextField source="id" />
        </ReferenceField>
        <ReferenceField source="kycEndorsementId" reference="KycEndorsement" link="show">
          <TextField source="id" />
        </ReferenceField>
        <TranslatedTextField source="state" translation="resources.KycRequest.fields.states" />
        <ParsedDateTextField source='createdAt' />
        <TextField source='name' />
        <TextField source='lastName' />
        <TextFieldIdNumber source='idNumber' />
        <TextField source='birthdate' />
        <TextField source='country' />
        <TextField source='nationality' />
        <TextField source='jobTitle' />
        <TextField source='legalEntityName' />
        <TextField source='legalEntityCountry' />
        <TextField source='legalEntityRegistration' />
        <TextField source='legalEntityTaxId' />
        <TextField source='legalEntityLinkedinId' />
        <Button
          onClick={downloadKycRequestEvidence}
          label='resources.KycRequest.fields.downloadEvidence'>
          <GetApp />
        </Button>


      </SimpleShowLayout>
    </Show>
  );
};



const KycRequestEdit = () => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const redirect = useRedirect();
  const { id } = useParams();
  const translate = useTranslate();
  const [kycRequestEvidence, setKycRequestEvidence] = useState([]);
  const [open, setOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const getKycRequestEvidence = useCallback(async () => {
    try {
      let {data} = await dataProvider.getList('KycRequestEvidence', {
        pagination: { page: 1, perPage: 200 },
        sort: 'id',
        order: 'ASC',
        filter: { kycRequestIdEq: id },
      });
      setKycRequestEvidence(data);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  }, [dataProvider, notify, id]);


  const downloadKycRequestEvidence = useCallback(async () => {
    try {
      await createZipAndDownload(kycRequestEvidence, id);
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  }, [notify, id, kycRequestEvidence]);

  useEffect(() => {
    getKycRequestEvidence();
  }, [getKycRequestEvidence]);

  const handleSelect = bool => {
    document.querySelectorAll(".MuiFormControlLabel-root").forEach(x => {
      if (x.innerHTML.includes("Mui-checked") !== bool) {
        x.click();
      }
    });
  }

  const handleClick = () => setOpen(true);
  const handleDialogClose = () => setOpen(false);
  const handleConfirm = () => {
    document.querySelector("button[type='submit']").click();
  };

  const KycRequestToolbar = () => (
    <Toolbar>
    <Button
      label="resources.actions.processKycRequest"
      onClick={handleClick}
      variant="contained"
      className="custom-save-button" >
      <Input />
    </Button>
      <SaveButton alwaysEnable={true} sx={{ display: 'none' }} />
    </Toolbar>
  );

  const save = async values => {
    try{
      setOpen(false);
      setIsLoading(true);
      values.bool.evidence ||= [];
      values.form = JSON.stringify(values.bool);
      let {data} = await dataProvider.update('KycRequest', { data: values });
      notify('resources.actions.created');
      if (data.kycEndorsementId) {
        redirect(`/KycEndorsement/${data.kycEndorsementId}/show`);
      } else {
        redirect(`/KycRequest/${data.id}/show`);
      }
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
    setIsLoading(false);
  };

  return (
  <Edit>
    <>
      <SimpleForm
        onSubmit={save}
        toolbar={<KycRequestToolbar />}
      >
        <div className="flex-direction-column">
          <ReferenceField source="personId" reference="Person" link="show">
            <PersonTextInput source='id' disabled={true} validate={required()} />
          </ReferenceField>
          <i className="warning-message">
            {translate("resources.KycRequest.messageForPartialAcceptance")}
          </i>
          <div className="flex-direction-row">
            <Button
              startIcon={<CheckCircle />}
              label="resources.actions.acceptKycRequest"
              onClick={() => handleSelect(true)}
              className="button-kyc-request"
            />
            <Button
              startIcon={<Cancel />}
              label="resources.actions.rejectKycRequest"
              onClick={() => handleSelect(false)}
              className="button-kyc-request"
            />
          </div>
          <KycBooleanInput source="name"  />
          <KycBooleanInput source="lastName"  />
          <KycBooleanInput source="idType" />
          <KycBooleanInput source="idNumber" />
          <KycBooleanInput source="birthdate" />
          <KycBooleanInput source="country" />
          <KycBooleanInput source="nationality" />
          <KycBooleanInput source="jobTitle" />
          <KycBooleanInput source="legalEntityName" />
          <KycBooleanInput source="legalEntityCountry" />
          <KycBooleanInput source="legalEntityRegistration" />
          <KycBooleanInput source="legalEntityTaxId" />
          <KycBooleanInput source="legalEntityLinkedinId" />


          <Divider variant="middle" />

          <FunctionField source="evidence"
            render={() => { 
              var fields = [
                <Button key="download"
                  label='resources.KycRequest.fields.downloadEvidence'
                  className="button-kyc-request"
                  onClick={downloadKycRequestEvidence}>
                    <GetApp />
                </Button>
              ];
              for (let evidence in kycRequestEvidence) {
                let source = `bool.evidence.${evidence}`;
                fields.push(<BooleanInput key={evidence} label={kycRequestEvidence[evidence].filename} source={source} defaultValue={true} />);
              }
              return fields;
            }}
          />
        </div>
      </SimpleForm>
      <Confirm
          isOpen={open}
          title={translate(`resources.actions.kycRequestTitle`)}
          content={
            <div >
              {translate("resources.actions.KycRequestContent")}
            </div>}
          onConfirm={handleConfirm}
          onClose={handleDialogClose}
        />

      <Backdrop
        sx={{ color: '#fff', zIndex: (theme) => theme.zIndex.drawer + 1 }}
        open={isLoading}
      >
        <Dialog open={isLoading}>
          <DialogTitle>{translate("resources.actions.loading")}</DialogTitle>
        </Dialog>
      </Backdrop>
    </>
  </Edit>
  );
};

export {KycRequestList, KycRequestShow, KycRequestEdit, kycRequestGrid, kycRequestFilters};