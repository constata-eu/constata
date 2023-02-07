import { FunctionField, TextInput } from 'react-admin';

const PersonTextInput= ({source, disabled, validate}) => {
  return(
  <FunctionField source={source}
    render={record => { 
      if (record.nickname) return <TextInput source='nickname' disabled={disabled} validate={validate} />;
      if (record.emailAddress) return <TextInput source='address' disabled={disabled} validate={validate} />;
      else return <TextInput source='id' disabled={disabled} validate={validate} />;
    }}
  />
)};

export default PersonTextInput;