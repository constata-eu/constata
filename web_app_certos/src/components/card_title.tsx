import { Box } from '@mui/material';
import { useTheme, useTranslate } from 'react-admin';
import { Head2 } from "../theme";

export default function CardTitle({text, ...props}) {
  const [theme] = useTheme();
  const translate = useTranslate();

  return(<Box {...props} sx={{ p: 2, borderBottom: "2px solid", borderColor: theme?.palette?.highlight?.main }}>
    <Head2>{ translate(text, { _: text }) }</Head2>
  </Box>)
}
