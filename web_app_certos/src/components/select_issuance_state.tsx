import { SelectInput } from 'react-admin';

const SelectIssuanceState = ({source, alwaysOn}) => {
  return(
  <SelectInput source={source} alwaysOn={alwaysOn} choices={[
    { id: 'created', name: "resources.Issuance.fields.states.created" },
    { id: 'signed', name: "resources.Issuance.fields.states.signed" },
    { id: 'completed', name: "resources.Issuance.fields.states.completed" },
  ]} />
)};

export default SelectIssuanceState;