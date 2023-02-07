import { DateInput } from 'react-admin';

const ParsedDateInput = ({source, label}) => {
  return(
  <DateInput source={source} label={label} parse={date => new Date(date + "T00:00:00")} />
)};

export default ParsedDateInput;