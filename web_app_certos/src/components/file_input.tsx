import type { InputHTMLAttributes } from "react";
import { useTranslate } from 'react-admin';
import { Box, IconButton, FormHelperText } from '@mui/material';
import { Login } from '@mui/icons-material';
import { readValuesAsText } from "./utils";
import { useSafeSetState } from 'ra-core';

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  disabled?: boolean,
  messageFile?: string | null,
  setConfFile: any
}

const FileInput = ({disabled, messageFile, setConfFile }: Props) => {
  const translate = useTranslate();
  const [label, setLabel] = useSafeSetState("");

  const onChange = async ({ target }) => {
    if (target.files[0]) {
      try{
        const text: any = await readValuesAsText(target.files[0]);
        const {public_key, encrypted_key, environment} = JSON.parse(text);
        if (!public_key || !encrypted_key || !environment) {
          throw new Error();
        }
        setConfFile(text);
        setLabel(`${target.files[0].name} ${translate("certos.actions.file_selected")}`);
      } catch {
        setLabel(translate("certos.errors.config_json"));
      }
    } else {
      setLabel("");
    }
  }

  return (
      <Box sx={{ display: 'flex', m: 'auto', flexDirection: 'column'}}>

        <Box sx={{alignSelf: 'center', justifySelf: 'center'}}>
          <label htmlFor="file_input">
          <IconButton color="primary" component="span">
            <Login />
            <Box sx={{fontSize: '13px', margin: '3px 0 0 5px'}}>
              {translate("certos.login.select_config_file")}
            </Box>
          </IconButton>
          </label>
        </Box>

        <input hidden type="file" onChange={onChange} id="file_input" accept=".json" disabled={disabled} />

        <Box sx={{alignSelf: 'center', justifySelf: 'center', fontSize: '12px', fontStyle: 'italic', marginLeft: '10px'}}>
          {label}
        </Box>
        <Box sx={{alignSelf: 'center', justifySelf: 'center', marginLeft: '10px'}}>
          <FormHelperText id="component-helper-text">{messageFile}</FormHelperText>
        </Box>

      </Box>
    )
};

export default FileInput;