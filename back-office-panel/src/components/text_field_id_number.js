import { FunctionField } from 'react-admin';

const TextFieldIdNumber = ({source}) => {
  return(
    <FunctionField
    source={source}
    render={record => {
      return [record.idType, record.idNumber].filter((x) => x).join(" ");
    }}
  />
)};

export default TextFieldIdNumber;