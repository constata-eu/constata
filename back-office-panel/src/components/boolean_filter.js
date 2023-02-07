import { NullableBooleanInput } from 'react-admin';

const MyBooleanFilter = ({source, resource, alwaysOn}) => {
  return(
  <NullableBooleanInput source={source} alwaysOn={alwaysOn}
    nullLabel={`resources.${resource}.${source}All`}
    falseLabel={`resources.${resource}.${source}False`}
    trueLabel={`resources.${resource}.${source}True`}
  />
)};

export default MyBooleanFilter;