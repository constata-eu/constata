import { SelectInput } from 'react-admin';

const SelectDeletionReason = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} choices={[
    { id: 'UserRequest', name: "resources.OrgDeletion.fields.reasons.USER_REQUEST" },
    { id: 'ConstataDecision', name: "resources.OrgDeletion.fields.reasons.CONSTATA_DECISION" },
    { id: 'Inactivity', name: "resources.OrgDeletion.fields.reasons.INACTIVITY" },
  ]} />
)};

export default SelectDeletionReason;