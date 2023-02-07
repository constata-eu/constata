import { SelectInput } from 'react-admin';

const SelectRequestState = ({source, alwaysOn}) => {
  return(
  <SelectInput source={source} alwaysOn={alwaysOn} choices={[
    { id: 'created', name: "resources.Request.fields.states.created" },
    { id: 'signed', name: "resources.Request.fields.states.signed" },
    { id: 'completed', name: "resources.Request.fields.states.completed" },
  ]} />
)};

export default SelectRequestState;