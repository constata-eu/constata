import { FunctionField } from 'react-admin';
import { parseDate } from './utils';

interface ParsedDateTextFieldInterface {
  source: string,
  label?: string,
}

const ParsedDateTextField = ({source, label}: ParsedDateTextFieldInterface) => {
  return(
  <FunctionField source={source} label={label}
    render={record => { 
      if (!record[source]) return;
      return parseDate(record[source]);
    }}
  />
)};

export default ParsedDateTextField;