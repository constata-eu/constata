import { SelectInput } from 'react-admin';

const SelectTemplateKind = ({source}) => {
  return(
  <SelectInput source={source} choices={[
    { id: 'DIPLOMA', name: "resources.Template.fields.kinds.diploma" },
    { id: 'ASSISTANCE', name: "resources.Template.fields.kinds.assistance" },
    { id: 'BADGE', name: "resources.Template.fields.kinds.badge" },
    { id: 'INVITATION', name: "resources.Template.fields.kinds.invitation" },
  ]} />
)};

export default SelectTemplateKind;