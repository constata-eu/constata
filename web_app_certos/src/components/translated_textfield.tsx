import { FunctionField, useTranslate } from 'react-admin';

interface TranslatedTextFieldInterface {
  source: string,
  label?: string,
  translation: string,
}

const TranslatedTextField = ({source, label, translation}: TranslatedTextFieldInterface) => {
  const translate = useTranslate();
  return <FunctionField source={source} label={label}
    render={record => { 
      if (!record[source]) return;
      return translate(`${translation}.${record[source]}`);
    }}
  />
};

export default TranslatedTextField;