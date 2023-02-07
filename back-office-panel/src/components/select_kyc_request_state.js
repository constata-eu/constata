import { SelectInput } from 'react-admin';

const SelectKycRequestState = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} defaultValue={'pending'} choices={[
    { id: 'pending', name: "resources.KycRequest.fields.states.pending" },
    { id: 'processed', name: "resources.KycRequest.fields.states.processed" },
  ]} />
)};

export default SelectKycRequestState;