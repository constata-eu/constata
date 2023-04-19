import { useEffect, useState, useCallback } from "react";
import { List, Datagrid, TextField, ShowButton, useDataProvider, useTranslate, FunctionField,
         SimpleShowLayout, ShowBase, useNotify, Button, ReferenceManyField, Pagination,
         FilterForm, SimpleList, WithRecord, useGetRecordId, useRecordContext } from 'react-admin'
import { Typography, Container, Box, Card, Link, useMediaQuery,
         Alert, AlertTitle } from '@mui/material';
import CardTitle from '../components/card_title';
import { PaginationDefault, formatJsonInline,
         copyToClipboard, parseDate, openBlob, defaultSort } from '../components/utils'
import { Link as LinkIcon } from '@mui/icons-material';
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
    <Container maxWidth={false} sx={{mb:3}}>
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
              <FunctionField source="adminVisitedCount" sortable={false}
                render={record => `${record.adminVisitedCount}/${record.entriesCount}`} 
              />  
              <TextField source="publicVisitCount" sortable={false} />
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
  let value = await dataProvider.getOne("PreviewEntry", {id});
  let blob = new Blob([value.data.html], {type : 'text/html'});
  await openBlob(blob);
}

const IssuanceErrors = () => {
  const translate = useTranslate();
  const record = useRecordContext();
  if (!record || !record.errors) return null;
  return <Alert sx={{ mb: 2 }} severity="error" variant="outlined" icon={false}>
    <AlertTitle>{ translate("certos.states.failed") }</AlertTitle>
    <TranslatedTextField source="errors" translation="certos.issuance.errors" />
  </Alert>;
};

function IssuanceShow(props){
  const translate = useTranslate();
  const issuanceId = useGetRecordId();
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const [entryFilters, setEntryFilters] = useState({});
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('md'));
  const record = useRecordContext();

  const onEntryClick = (data) => {
    if(data.storyId && data.state === "completed") {
      copyToClipboard(data.adminAccessUrl, notify);
    } else {
      openPreview(dataProvider, data.id)
    }
  };

  const EntryActions = ({ data }) => {
    return <div>
      { data.storyId && data.state === "completed" ?
        <Button
          onClick={() => onEntryClick(data)}
          label="resources.Entry.fields.copyUrl"
        >
          <LinkIcon />
        </Button>
        :
        <Button
          onClick={() => onEntryClick(data) }
          label="certos.entry.preview"
        />
      }
    </div>
  };

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
      <Container maxWidth={false} sx={{mb:3}}>
        <Card sx={{ mb: 3 }}>
          <CardTitle text={<>
            <Link href="#/Issuance"> { translate("resources.Issuance.admin_title") } </Link>
            &gt;
            <WithRecord render={record => <>{`${record.id} - ${record.name}`}</> } />
          </>} />
          <Box py={1}>
            <SimpleShowLayout>
              <FunctionField source="state" render={ record => 
                <>
                  <TranslatedTextField source="state" translation="certos.issuance.states" />
                  <br/>
                  { (record.state !== "completed" && record.state !== "failed") && 
                    <TranslatedTextField
                      source="state"
                      label="resources.Issuance.fields.nextStep"
                      translation="resources.Issuance.fields.nextSteps"
                    />
                  } 
                </>
              }/>
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
              <FunctionField source="adminVisitedCount" render={record => `${record.adminVisitedCount}/${record.entriesCount}`} />  
              <TextField source="publicVisitCount"/>
              <FunctionField label="resources.Issuance.fields.export_csv" render={() => 
                <a id="export_to_csv" href="#/Issuance" onClick={handleExport}>
                  {translate("resources.Issuance.fields.download")}
                </a>
              }/>
            </SimpleShowLayout>
          </Box>
        </Card>
        <IssuanceErrors />
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
              sort={{ field: 'id', order: 'ASC' }}
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
                  <FunctionField source="statistics" sortable={false}
                    render={record => {
                      const keyAdminVisited = translate("resources.Entry.fields.adminVisited");
                      const keyPublicVisitCount = translate("resources.Entry.fields.publicVisitCount");
                      let statisticsJson = {};
                      statisticsJson[keyAdminVisited] = record.adminVisited ? translate("resources.Entry.yes") : translate("resources.Entry.no");
                      statisticsJson[keyPublicVisitCount] = record.publicVisitCount;
                      return formatJsonInline(JSON.stringify(statisticsJson))
                    }}
                  />
                  <FunctionField source="params" render={ record => formatJsonInline(record.params)} sortable={false} />
                  <FunctionField className="column-copyLink"
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
