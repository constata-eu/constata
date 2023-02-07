import { FunctionField, useTranslate } from 'react-admin';
import MyUrlField from './my_url_field';
import ParsedDateTextField from './parsed_date_textfield';

const AcceptedField = ({source, sortable}) => {
  const translate = useTranslate();
  return(
    <FunctionField source={source} sortable={sortable}
    render={record => {
    if (!record.accepted) {
      return <MyUrlField source='url' text={translate('resources.Person.fields.not_accepted_yet')} />;
    }
    return <ParsedDateTextField source='accepted' />
  }}
  />
)};

export default AcceptedField;