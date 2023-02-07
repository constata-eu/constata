import { FunctionField } from 'react-admin';
import {FieldCopyWithUrl} from './copy_to_clipboard';

const MyUrlField = ({source, sortable, text}) => {
  return(
    <FunctionField source={source} sortable={sortable}
      render={record => {
      if (!record[source]) return;
      return <FieldCopyWithUrl text={text} url={record[source]}/>;
    }}
    />
)};

export default MyUrlField;