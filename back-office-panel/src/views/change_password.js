import React from "react";
import { SimpleForm, PasswordInput, useDataProvider, Create, useNotify, minLength, maxLength,
         useLogout, required } from 'react-admin';
import {OnlySaveToolbar} from '../components/bottom_toolbars';

const AdminUserChangePassword = () => {
  const notify = useNotify();
  const logout = useLogout();
  const dataProvider = useDataProvider();
  
  const save = async values => {
    try {
      await dataProvider.update('AdminUser', { data: values })
      notify('resources.actions.updated');
      logout();
    } catch {
      notify('admin.errors.default', {type: 'warning'});
    }
  };

  const validateConfirmation = (value, allValues) => {
    if (value !== allValues.newPassword) {
      return 'resources.ChangePassword.confirmationFailed';
    }
  };
  const validatePassword = [required(), minLength(8)];

  return (
  <Create resource="AdminUser">
      <SimpleForm
        onSubmit={save}
        toolbar={<OnlySaveToolbar />}
      >
        <PasswordInput source="password" autoComplete="off" validate={validatePassword} />
        <PasswordInput source="newPassword" autoComplete="off" validate={validatePassword} />
        <PasswordInput source="reNewPassword" autoComplete="off"  validate={[required(), minLength(8), validateConfirmation]} />
        <PasswordInput source="otp" autoComplete="off" validate={[required(), minLength(6), maxLength(6)]} />
      </SimpleForm>
  </Create>
  );
};
  
export {AdminUserChangePassword};