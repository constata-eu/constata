import { useEffect, useState, useCallback } from "react";
import { List, Datagrid, TextField, ShowButton, useDataProvider, useTranslate, FunctionField,
         SimpleShowLayout, ShowBase, useNotify, Button, ReferenceManyField, Pagination,
         FilterForm, SimpleList, WithRecord, useGetRecordId, BooleanField } from 'react-admin'
import { Typography, Container, Box, Card, Link, useMediaQuery } from '@mui/material';
import CardTitle from '../components/card_title';
import { PaginationDefault, formatJsonInline,
         downloadFile, parseDate, openBlob, defaultSort } from '../components/utils'
import { TaskAlt } from '@mui/icons-material';
import SelectTemplates from "../components/select_templates";
import TranslatedTextField from "../components/translated_textfield";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import SelectIssuanceState from "../components/select_issuance_state";

function IssuanceList(props) {
  const dataProvider = useDataProvider();
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('sm'));

  const issuanceFilters = [
    <FilterTextInput source="nameLike" alwaysOn />,
    <SelectTemplates source="templateIdEq" alwaysOn />,
    <SelectIssuanceState source="stateEq" alwaysOn />,
  ];

  const hasNoTemplates = useCallback(async () => {
    await dataProvider.getList('Template', {
      pagination: { page: 1, perPage: 1 }, sort: {field: 'id', order: 'ASC'}, filter: {},
    });
  }, [dataProvider]);

  useEffect(() => {
    hasNoTemplates();
  }, [hasNoTemplates]);

  return (
    <Container maxWidth="md" sx={{mb:3}}>
      <Card>
        <CardTitle text="resources.Issuance.admin_title"/>
        <List {...props}
          empty={false}
          sort={defaultSort}
          filters={issuanceFilters}
          perPage={20}
          pagination={<PaginationDefault />}
          actions={false}
        >
          { isSmall ?
            <SimpleList 
              primaryText={record => `${record.id} - ${record.name}` }
              secondaryText={record => `${record.state} | ${record.templateName}` }
              tertiaryText={record => new Date(record.createdAt).toLocaleDateString()}
              linkType="show"
            /> :
            <Datagrid bulkActionButtons={false}>
              <TextField source='id' />
              <TextField source="name" />
              <TranslatedTextField source="state" translation="certos.issuance.states" />
              <FunctionField
                source='templateId'
                render={record => {
                let href = `#/Template/${record.templateId}/Show`;
                return <a href={href}>{record.templateId} - {record.templateName}</a>}
                }
              />
              <ParsedDateTextField source="createdAt" />
              <FunctionField source="adminVisitCount" render={record => `${record.adminVisitedCount}/${record.entriesCount}`} />  
              <TextField source="publicVisitCount"/>
              <FunctionField
                render={record => {
                  if (record.state === "created") {
                    return <>
                      <ShowButton record={record}/>
                      <br/>
                      <Button href={ `#/wizard/${record.id}` } />
                    </>
                  } else {
                    return <ShowButton record={record}/>
                  }
                }}
              />
            </Datagrid>
          }
        </List>
      </Card>
    </Container>
  );
}

export const openPreview = async (dataProvider, id) => {
  let value = await dataProvider.getOne("Preview", {id});
  let blob = new Blob([value.data.html], {type : 'text/html'});
  await openBlob(blob);
}

function IssuanceShow(props){
  const translate = useTranslate();
  const issuanceId = useGetRecordId();
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const [entryFilters, setEntryFilters] = useState({});
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('md'));

  const onEntryClick = (data) => {
    if(data.storyId && data.state === "completed") {
      downloadProof(data.storyId)
    } else {
      openPreview(dataProvider, data.id)
    }
  };

  const EntryActions = ({ data }) => {
    return <div>
      { data.storyId && data.state === "completed" ?
        <Button
          sx={{display: "block"}}
          onClick={() => onEntryClick(data)}
          label="resources.Entry.fields.downloadProof"
        >
          <TaskAlt />
        </Button>
        :
        <Button
          onClick={() => onEntryClick(data) }
          label="certos.entry.preview"
        />
      }
    </div>
  };

  const downloadProof = async (storyId) => {
    await downloadFile(`/stories/${storyId}/html_proof`, `proof_${storyId}.html`, notify);
  }

  function handleChange(e){
    if(e.target.value === ""){
      setEntryFilters({})
    }
  }

  const entryFilterFields = [
    <FilterTextInput source="paramsLike" onChange={handleChange} alwaysOn />,
  ];

  const handleExport = async (e) => {
    e.preventDefault();
    const {data} = await dataProvider.getOne('IssuanceExport', { id: issuanceId });
    openBlob(
      new Blob([data.csv], {type: "text/csv" }),
      translate("certos.issuance.export_filename", {id: data.id})
    )
  }


  return (
    <ShowBase {...props} actions={false}>
      <Container maxWidth="md" sx={{mb:3}}>
        <Card sx={{ mb: 3 }}>
          <CardTitle text={<>
            <Link href="#/Issuance"> { translate("resources.Issuance.admin_title") } </Link>
            &gt;
            <WithRecord render={record => <>{`${record.id} - ${record.name}`}</> } />
          </>} />
          <Box py={1}>
            <SimpleShowLayout>
              <TranslatedTextField source="state" translation="certos.issuance.states" />
              <WithRecord render={ record => {
                if (record.state !== "completed" && record.state !== "failed") {
                  return (<TranslatedTextField
                    source="state"
                    label="resources.Issuance.fields.nextStep"
                    translation="resources.Issuance.fields.nextSteps"
                  />);
                } else {
                  return <></>;
                }
              }}/>
              <FunctionField source="name"
                render={record => `${record.id} - ${record.name}` }
              />
              <FunctionField source="templateId"
                render={record => {
                  let href = `#/Template/${record.templateId}/Show`;
                  return <a href={href}>{record.templateId} - {record.templateName}</a>}
                }
              />
              <ParsedDateTextField source="createdAt" />
              <FunctionField source="adminVisitCount" render={record => `${record.adminVisitedCount}/${record.entriesCount}`} />  
              <TextField source="publicVisitCount"/>
              <FunctionField label="resources.Issuance.fields.export_csv" render={() => 
                <a id="export_to_csv" href="#/Issuance" onClick={handleExport}>
                  {translate("resources.Issuance.fields.download")}
                </a>
              }/>

              <WithRecord render={ record => 
                record.errors && <TranslatedTextField source="errors" translation="certos.issuance.errors" />
              } />
            </SimpleShowLayout>
          </Box>
        </Card>
        <Card>
          <Box mt={2}>
            <FilterForm
              filters={entryFilterFields} 
              setFilters={setEntryFilters}
              displayedFilters={ {"paramsLike": true} }
             />
          </Box>

          <WithRecord render={ issuance => 
            <ReferenceManyField reference="Entry" target="issuanceIdEq" label=""
              pagination={<Pagination rowsPerPageOptions={[25]} />}
              filter={ entryFilters }
              perPage={25}
            >
              { isSmall ?
                  <SimpleList
                    id="review-entries-small"
                    primaryText={(record) => 
                      <Box component="div" sx={{cursor:"pointer", px: 2 }} onClick={() => onEntryClick(record) }>
                        <Typography variant="button" id={`preview-` + record.id}>
                          { translate("certos.wizard.review_and_sign.review_label")}
                          &nbsp;
                          { translate(`certos.wizard.kind.${issuance.templateKind}`)}
                          &nbsp;
                          #{record.id}
                        </Typography>
                        <FunctionField source="params" render={ record => formatJsonInline(record.params)} />
                      </Box> 
                    }
                    linkType={false}
                  />
                :
                <Datagrid bulkActionButtons={false} id="review-entries-big">
                  <TextField source='id' />
                  <div>
                    <TranslatedTextField
                      source="errors"
                      label="resources.Entry.fields.state"
                      translation="certos.entry.states"
                    />
                    <TextField source='errors' label="resources.Entry.fields.errors" />
                  </div>

                  <TextField source='rowNumber' label="resources.Entry.fields.rowNumber"/>
                  <FunctionField source="email_callback" 
                    label="resources.Entry.fields.hasEmailCallback"
                    render={record => {
                      let emailParam = JSON.parse(record.params.toLowerCase())["email"];
                      if (record.state === "created" && emailParam?.includes("@")) return translate("certos.entry.notify_at_sign");
                      else if (!record.hasEmailCallback ) return translate("certos.entry.without_notify");
                      else if (!record.emailCallbackSentAt) return translate("certos.entry.with_notify");
                      else return `${translate("certos.entry.notified")} ${parseDate(record.emailCallbackSentAt)}.`;
                    }}
                  />
                  <BooleanField source="adminVisited" />
                  <TextField source="publicVisitCount"/>
                  <FunctionField source="params" render={ record => formatJsonInline(record.params)} sortable={false} />
                  <FunctionField
                    render={record => <EntryActions data={record} {...props} /> }
                  />
                </Datagrid>
              }
            </ReferenceManyField>
          }/>
        </Card>
      </Container>
    </ShowBase>
  );
}

export {IssuanceList, IssuanceShow};
