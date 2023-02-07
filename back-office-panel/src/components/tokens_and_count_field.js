import { FunctionField } from 'react-admin';

const TokensAndCountField = ({source, count, sortable}) => {
  return(
  <FunctionField source={source} sortable={sortable}
    render={record => {
      if (record[source] === 0) return '0 MB';
      if (record[source] === record[count]) return `${record[source]} MB/Documents`;
      return `${record[source]} MB - ${record[count]} Documents`;
    }}
  />
)};

export default TokensAndCountField;