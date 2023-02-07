import { FunctionField } from 'react-admin';
import { parseDate } from './utils';

const ParsedDateTextField = ({source, label, sortable}) => {
  return(
  <FunctionField source={source} label={label} sortable={sortable}
    render={record => { 
      if (!record[source]) return;
      return parseDate(record[source]);
    }}
  />
)};

export default ParsedDateTextField;