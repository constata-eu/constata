import { FunctionField } from 'react-admin';

const PersonTextField = ({source}) => {
  return(
  <FunctionField source={source}
    render={record => { 
      if (record.emailAddress) return `${record.id} - ${record.address}`;
      if (record.telegram) return `${record.id} - ${record.telegramFirstName}`;
      if (record.nickname) return `${record.id} - ${record.nickname}`;
      else return record.id;
    }}
  />
)};

export default PersonTextField;