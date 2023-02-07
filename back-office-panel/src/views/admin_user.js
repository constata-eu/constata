import React from "react";
import { Create, SimpleForm, TextInput, PasswordInput, required, TextField, List,
         Datagrid, TopToolbar, FilterButton, CreateButton } from 'react-admin';
import { PostPagination, defaultSort } from "../components/utils";
import SelectAdminRole from "../components/select_admin_role";
import ParsedDateTextField from "../components/parsed_date_textfield";
import FilterTextInput from "../components/filter_textinput";

function AdminUserList() {

  const TopToolbarWithCreateButton = () => {
    return (
    <TopToolbar>
      <FilterButton />
      <CreateButton />
    </TopToolbar> 
  )};

  const adminUserFilters = [
    <FilterTextInput source="usernameLike" />,
    <SelectAdminRole source="roleEq" />
  ];
  
  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={adminUserFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarWithCreateButton/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id' />
        <TextField source='username' />
        <TextField source='otpSeed' sortable={false} />
        <TextField source='role' />
        <ParsedDateTextField source='createdAt' />
      </Datagrid>
    </List>
  );
}


function AdminUserCreate() {
  return (
    <Create title='resources.AdminUser.create'>
        <SimpleForm >
          <TextInput source='username' autoComplete="off" validate={required()}/>
          <PasswordInput source='password' autoComplete="off" validate={required()}/>
          <SelectAdminRole source="role" validate={required()}/>
        </SimpleForm>
    </Create>
  );
}

export {AdminUserList, AdminUserCreate};