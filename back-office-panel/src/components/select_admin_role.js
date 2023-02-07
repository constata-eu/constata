import { SelectInput } from 'react-admin';

const SelectAdminRole = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} choices={[
    { id: 'Admin', name: "resources.AdminUser.fields.roles.admin" },
    { id: 'SuperAdmin', name: "resources.AdminUser.fields.roles.superadmin" },
  ]} />
)};

export default SelectAdminRole;