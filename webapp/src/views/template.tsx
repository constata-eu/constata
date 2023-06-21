import { List, SimpleList, Datagrid, TextField, ShowButton, FunctionField, ShowBase,
  SimpleShowLayout, useNotify, useTranslate, RichTextField,
  useGetRecordId, WithRecord, BooleanField, useRefresh
} from 'react-admin';
import {
  ListActionsWithoutCreate, PaginationDefault, downloadFile,
} from '../components/utils';
import { Box, Container, Card, Link, useMediaQuery } from '@mui/material';
import CardTitle from '../components/card_title';
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";
import ArchiveTemplateAction from '../components/archive_template_action';


function TemplateList(props) {
  const translate = useTranslate();
  const refresh = useRefresh();
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('sm'));


  const templateFilters = [
    <FilterTextInput source="nameLike" alwaysOn />,
  ];

  return (
    <Container maxWidth={false} sx={{mb:3}}>
      <Card>
        <CardTitle text={<>
          <Link href="#/Issuance"> { translate("resources.Issuance.admin_title") } </Link>
          &gt;
          { translate("resources.Template.admin_title") }
        </>} />
        <List {...props}
          empty={false}
          filters={templateFilters}
          perPage={20}
          pagination={<PaginationDefault />}
          actions={<ListActionsWithoutCreate />}
        >
            { isSmall ?
              <SimpleList 
                primaryText={record => `${record.id} - ${record.name}` }
                tertiaryText={record => new Date(record.createdAt).toLocaleDateString()}
                linkType="show"
              /> :
              <Datagrid bulkActionButtons={false}>
                <TextField source='id' />
                <TextField source="name" />
                <BooleanField source="archived" />
                <ParsedDateTextField source="createdAt" />
                <FunctionField source="adminVisitedCount" sortable={false}
                  render={record => `${record.adminVisitedCount}/${record.entriesCount}`} 
                />  
                <TextField source="publicVisitCount" sortable={false} />
                <ShowButton />
                <FunctionField render={record => {
                  return <ArchiveTemplateAction
                    templateId={record.id}
                    templateArchived={record.archived}
                    variant={"text"}
                    refresh={refresh}
                  />
                }}/>
              </Datagrid>
            }
        </List>
      </Card>
    </Container>
  );
}

function TemplateShow(props){
  const templateId = useGetRecordId();
  const notify = useNotify();
  const translate = useTranslate();
  const refresh = useRefresh();

  const handlePayload = async e => {
    e.preventDefault();
    await downloadFile(`/template/${templateId}/download_payload`, `template_${templateId}.zip`, notify);
  }

  return (
    <ShowBase {...props}>
      <Container maxWidth={false} sx={{mb:3}}>
        <Card sx={{ mb: 3 }}>
          <CardTitle text={<>
            <Link href="#/Issuance"> { translate("resources.Issuance.admin_title") } </Link>
            &gt;
            <Link href="#/Template"> { translate("resources.Template.admin_title") } </Link>
            &gt;
            <WithRecord render={record => <>{`${record.id} - ${record.name}`}</> } />
          </>} />

          <Box py={1}>
            <SimpleShowLayout>
              <FunctionField source="name"
                render={record => { return `${record.id} - ${record.name}`; }}
              />
              <BooleanField source="archived" />
              <ParsedDateTextField source="createdAt" />
              <FunctionField source="adminVisitedCount" render={record => `${record.adminVisitedCount}/${record.entriesCount}`} />  
              <TextField source="publicVisitCount"/>
              <RichTextField source='customMessage' />
              <FunctionField label="certos.template.evidence"
                  render={() => {
                    return <a href="#/Template" onClick={handlePayload}>
                      {translate("certos.template.download_zip")}
                    </a>
                  }}
                />
              <FunctionField render={record => {
                return <ArchiveTemplateAction
                  templateId={record.id}
                  templateArchived={record.archived}
                  variant="outlined"
                  refresh={refresh}
                />
              }}/>
            </SimpleShowLayout>
          </Box>
        </Card>
      </Container>
    </ShowBase>
  );
}


export {TemplateList, TemplateShow};
