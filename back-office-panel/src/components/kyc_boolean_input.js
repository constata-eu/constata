import { BooleanInput, FunctionField, useTranslate } from 'react-admin';
import { parseDate } from './utils';

const KycBooleanInput = ({source}) => {
  const translate = useTranslate();
  return(
  <FunctionField source={source}
    render={record => {
      var sx = {};
      if (!record[source]) { sx = {display: "none"}; }
      let translatedResource = translate(`resources.KycRequest.fields.${source}`);
      let value = source === "birthdate" ? parseDate(record[source]) : record[source]; 
      return <BooleanInput
        source={"bool."+source}
        label={`${translatedResource}: ${value}`}
        defaultValue={true}
        sx={sx}
      />
    }}
  />

  
)};

export default KycBooleanInput;