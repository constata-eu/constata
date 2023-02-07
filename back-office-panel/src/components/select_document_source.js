import { SelectInput } from 'react-admin';

const SelectDocumentSources = ({source, validate}) => {
  return(
  <SelectInput source={source} validate={validate} choices={[
    { id: 'Email', name: 'resources.Document.fields.sourcedFroms.EMAIL' },
    { id: 'Api', name: 'resources.Document.fields.sourcedFroms.API' },
    { id: 'Telegram', name: 'resources.Document.fields.sourcedFroms.TELEGRAM' },
    { id: 'Internal', name: 'resources.Document.fields.sourcedFroms.INTERNAL' },
  ]} />
)};

export default SelectDocumentSources;