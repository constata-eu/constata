import { FunctionField, useTranslate } from 'react-admin';

const TranslatedTextField = ({source, label, translation}) => {
  const translate = useTranslate();
  return(
  <FunctionField source={source} label={label}
    render={record => { 
      if (!record[source]) return;
      return translate(`${translation}.${record[source]}`);
    }}
  />
)};

export default TranslatedTextField;