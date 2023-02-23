import { List, SimpleList, Datagrid, TextField, ShowButton, FunctionField, ShowBase,
  SimpleShowLayout, useNotify, useTranslate, RichTextField,
  useGetRecordId, WithRecord
} from 'react-admin';
import {
  ListActionsWithoutCreate, PaginationDefault, downloadFile,
} from '../components/utils';
import { Box, Container, Card, Link, useMediaQuery } from '@mui/material';
import CardTitle from '../components/card_title';
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";

function TemplateList(props) {
  const translate = useTranslate();
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('sm'));

  const templateFilters = [
    <FilterTextInput source="nameLike" alwaysOn />,
  ];

  return (
    <Container maxWidth="md" sx={{mb:3}}>
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
                <ParsedDateTextField source="createdAt" />
                <ShowButton />
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

  const handlePayload = async e => {
    e.preventDefault();
    await downloadFile(`/template/${templateId}/download_payload`, `template_${templateId}.zip`, notify);
  }

  return (
    <ShowBase {...props}>
      <Container maxWidth="md" sx={{mb:3}}>
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
              <ParsedDateTextField source="createdAt" />
              <RichTextField source='customMessage' />
              <FunctionField label="certos.template.evidence"
                  render={() => {
                    return <a href="#/Template" onClick={handlePayload}>
                      {translate("certos.template.download_zip")}
                    </a>
                  }}
                />
            </SimpleShowLayout>
          </Box>
        </Card>
      </Container>
    </ShowBase>
  );
}


export {TemplateList, TemplateShow};
