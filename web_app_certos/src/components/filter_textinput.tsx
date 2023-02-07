import React from 'react';
import { TextInput } from 'react-admin';

interface FilterTextInputInterface {
  source: string,
  label?: string,
  alwaysOn: boolean,
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void
}

const FilterTextInput = ({source, label, alwaysOn, onChange}: FilterTextInputInterface) => {
  return(
  <TextInput
    source={source}
    label={label}
    alwaysOn={alwaysOn}
    onChange={onChange}
    autoComplete="off" />
)};

export default FilterTextInput;
