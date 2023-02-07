import React from "react";
import { List, Datagrid, ShowButton, FunctionField, ReferenceField, TextField } from 'react-admin'
import { PostPagination } from "../components/utils";
import ParsedDateTextField from "../components/parsed_date_textfield";
import { FieldCopyWithUrl } from "../components/copy_to_clipboard";
import TokensAndCountField from "../components/tokens_and_count_field";
import TokensField from "../components/tokens_field";


function MissingTokenList() {
  const userFilters = [];

  return (
    <List
      empty={false}
      filters={userFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={null}
    >
      <Datagrid bulkActionButtons={false}>
        <ReferenceField source="id" reference="Org" link="show" sortable={false} >
          <TextField source="nameForOnBehalfOf" />
        </ReferenceField>
        <TextField source="publicName" sortable={false}/>
        <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
        <TokensField source='tokenBalance' sortable={false} />
        <TokensField source='missingTokens' sortable={false} />
        <ParsedDateTextField source='lastParkedDate' sortable={false} />
        <FunctionField source='invoiceUrl' sortable={false}
          render={record => { record.invoiceUrl ?
          <FieldCopyWithUrl text={'Invoice'} url={record.invoiceUrl}/> :
          <FieldCopyWithUrl text={'InvoiceLink'} url={record.invoiceLinkUrl}/>
          }}
        />;
        <ShowButton resource="Org" />
      </Datagrid>
    </List>
  );
}

export default MissingTokenList;