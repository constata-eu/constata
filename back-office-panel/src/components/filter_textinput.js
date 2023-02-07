import { TextInput } from 'react-admin';

const FilterTextInput = ({source, label, alwaysOn, onChange}) => {
  return(
  <TextInput
    source={source}
    label={label}
    alwaysOn={alwaysOn}
    onChange={onChange}
    autoComplete="off" />
)};

export default FilterTextInput;