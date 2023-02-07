import React from "react";
import {List, Datagrid, ShowButton, TextField, ReferenceField} from 'react-admin'
import {PostPagination} from "../components/utils";
import TokensAndCountField from "../components/tokens_and_count_field";


function TopTenList() {
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
        <TextField source='publicName' sortable={false} />
        <TokensAndCountField source="fundedTokens" count="fundedCount" sortable={false} />
        <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
        <TextField source='tokenBalance' sortable={false} />
        <TextField source='missingTokens' sortable={false} />
        <ShowButton resource="Org" />
      </Datagrid>
    </List>
  );
}

export default TopTenList;