import { FunctionField } from 'react-admin';

const TokensField = ({source, sortable}) => {
  let name = source === "missingTokens" ? "missing" : "token";
  return(
  <FunctionField source={source} sortable={sortable}
    render={record => {
      if (record[source] === 0) return '0';
      if (record[source] === 1) return `${record[source]} ${name}`;
      return `${record[source]} ${name}s`;
    }}
  />
)};

export default TokensField;