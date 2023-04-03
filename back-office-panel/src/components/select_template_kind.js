import { SelectInput } from 'react-admin';

const SelectTemplateKind = ({source}) => {
  return(
  <SelectInput source={source} choices={[
    { id: 'DIPLOMA', name: "resources.Template.fields.kinds.diploma" },
    { id: 'ATTENDANCE', name: "resources.Template.fields.kinds.attendance" },
    { id: 'BADGE', name: "resources.Template.fields.kinds.badge" },
  ]} />
)};

export default SelectTemplateKind;