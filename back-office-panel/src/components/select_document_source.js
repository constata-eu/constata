import { SelectInput } from 'react-admin';

const SelectDocumentSources = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} choices={[
    { id: 'Email', name: 'resources.Document.fields.sourcedFroms.EMAIL' },
    { id: 'Api', name: 'resources.Document.fields.sourcedFroms.API' },
    { id: 'Internal', name: 'resources.Document.fields.sourcedFroms.INTERNAL' },
  ]} />
)};

export default SelectDocumentSources;
